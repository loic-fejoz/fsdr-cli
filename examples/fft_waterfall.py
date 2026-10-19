#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""fsdr-hub : FFT Globale + Waterfall + Zoom SSB + Contrôle VFO bidirectionnel (Bokeh).

Consomme le serveur WebSocket unifié de télémétrie de fsdr-hub (ws://127.0.0.1:5006/ws) :
- Flux binaire float32 : sondes "panoramic_20m" (4096 bins) et "ssb_voice_zoom" (1024 bins).
- Flux de contrôle JSON bidirectionnel : synchronisation automatique du VFO
  (depuis CAT rigctld / WSJT-X ou depuis l'IHM web par clic/sliders/boutons).
- Indicateurs visuels sur la FFT globale et le waterfall montrant l'emplacement
  et la bande passante de chaque canal configuré dans fsdr-hub.

Lancement :
    .venv/bin/bokeh serve examples/fft_waterfall.py --allow-websocket-origin='*' --port 5010
puis ouvrir http://localhost:5010/fft_waterfall
"""
import asyncio
import json
import os

import numpy as np
import bokeh.palettes as bp
from tornado.ioloop import IOLoop
from tornado.websocket import websocket_connect

from bokeh.events import Tap
from bokeh.io import curdoc
from bokeh.layouts import column, row
from bokeh.models import (
    BoxAnnotation,
    Button,
    ColumnDataSource,
    Div,
    Label,
    NumeralTickFormatter,
    Range1d,
    Select,
    Slider,
    Span,
    TextInput,
)
from bokeh.plotting import figure

WS_URL = os.environ.get("FSDR_WS_URL", "ws://127.0.0.1:5006/ws")

# Configuration des fréquences absolues initiales (en Hz)
CENTER_FREQ = float(os.environ.get("FSDR_CENTER_FREQ", 14_200_000.0))  # LO 14.200 MHz
FULL_BW = float(os.environ.get("FSDR_FULL_BW", 2_000_000.0))          # Bande 2 MHz
SSB_INITIAL_FREQ = float(os.environ.get("FSDR_SSB_FREQ", 14_250_000.0))# Canal SSB initial
SSB_BW = float(os.environ.get("FSDR_SSB_BW", 48_000.0))                # Zoom 48 kHz

# Calcul des bornes panoramiques en MHz
FREQ_MIN_MHZ = (CENTER_FREQ - FULL_BW / 2.0) / 1e6
FREQ_MAX_MHZ = (CENTER_FREQ + FULL_BW / 2.0) / 1e6
DW_MHZ = FULL_BW / 1e6

current_ssb_freq = SSB_INITIAL_FREQ
ssb_min_mhz = (current_ssb_freq - SSB_BW / 2.0) / 1e6
ssb_max_mhz = (current_ssb_freq + SSB_BW / 2.0) / 1e6

WIDE_BINS = 4096      # sonde panoramique brute
WIDE_SPEC_PTS = 1024  # points affichés sur la courbe FFT panoramique (décimation max)
SSB_BINS = 1024       # sonde zoom SSB
WF_COLS = 640         # largeur waterfall (downsampling 4096 -> 640)
WF_ROWS = 256         # profondeur du waterfall

# Configuration des canaux fsdr-hub avec nom, mode, bande passante et couleur
CHANNELS_CFG = [
    {
        "id": "ft8_20m",
        "name": "FT8",
        "freq": 14_074_000,
        "mode": "USB",
        "bw": 3_000,
        "color": "#ff5722",
        "desc": "Digital WSJT-X",
    },
    {
        "id": "wspr_20m",
        "name": "WSPR",
        "freq": 14_095_600,
        "mode": "USB",
        "bw": 2_000,
        "color": "#e91e63",
        "desc": "Balises WSPR",
    },
    {
        "id": "aprs_hf",
        "name": "APRS",
        "freq": 14_105_000,
        "mode": "LSB",
        "bw": 2_400,
        "color": "#ab47bc",
        "desc": "Paquets HF AFSK / KISS",
    },
    {
        "id": "sstv_20m",
        "name": "SSTV",
        "freq": 14_230_000,
        "mode": "USB",
        "bw": 3_000,
        "color": "#26a69a",
        "desc": "Images analogiques",
    },
    {
        "id": "ssb_voice_20m",
        "name": "SSB VFO",
        "freq": SSB_INITIAL_FREQ,
        "mode": "USB",
        "bw": 3_000,
        "color": "#ffca28",
        "desc": "Phonie USB mobile (VFO)",
        "is_vfo": True,
    },
]

def get_channel_bounds(ch_dict, carrier_freq):
    bw = ch_dict["bw"]
    mode = ch_dict["mode"]
    if mode == "USB":
        return carrier_freq / 1e6, (carrier_freq + bw) / 1e6
    elif mode == "LSB":
        return (carrier_freq - bw) / 1e6, carrier_freq / 1e6
    else:
        return (carrier_freq - bw / 2.0) / 1e6, (carrier_freq + bw / 2.0) / 1e6

# ---------- Gestion des palettes de couleurs ----------
def make_classic_palette():
    stops = np.array([
        [0.00, 0, 0, 0],
        [0.15, 12, 12, 64],
        [0.35, 0, 90, 200],
        [0.55, 0, 220, 255],
        [0.75, 60, 255, 60],
        [0.90, 255, 220, 40],
        [1.00, 255, 80, 40],
    ], dtype=float)
    x = np.arange(256) / 255.0
    r = np.interp(x, stops[:, 0], stops[:, 1]).astype(np.uint32)
    g = np.interp(x, stops[:, 0], stops[:, 2]).astype(np.uint32)
    b = np.interp(x, stops[:, 0], stops[:, 3]).astype(np.uint32)
    return r | (g << 8) | (b << 16) | (np.uint32(255) << 24)

def hex_palette_to_uint32(hex_list):
    arr = np.zeros(len(hex_list), dtype=np.uint32)
    for i, h in enumerate(hex_list):
        h = h.lstrip("#")
        r = int(h[0:2], 16)
        g = int(h[2:4], 16)
        b = int(h[4:6], 16)
        arr[i] = r | (g << 8) | (b << 16) | (np.uint32(255) << 24)
    return arr

def make_monochrome_palette():
    x = np.arange(256, dtype=np.uint32)
    return x | (x << 8) | (x << 16) | (np.uint32(255) << 24)

PALETTES = {
    "Classique SDR": make_classic_palette(),
    "Viridis": hex_palette_to_uint32(bp.Viridis256),
    "Plasma": hex_palette_to_uint32(bp.Plasma256),
    "Inferno": hex_palette_to_uint32(bp.Inferno256),
    "Turbo": hex_palette_to_uint32(bp.Turbo256),
    "Monochrome": make_monochrome_palette(),
}

current_db_min = float(os.environ.get("FSDR_DB_MIN", -70.0))
current_db_max = float(os.environ.get("FSDR_DB_MAX", 30.0))
current_gamma = float(os.environ.get("FSDR_GAMMA", 1.0))
current_palette = PALETTES["Classique SDR"]

def to_rgba(row_db):
    denom = max(1e-3, current_db_max - current_db_min)
    norm = np.clip((row_db - current_db_min) / denom, 0.0, 1.0)
    if current_gamma != 1.0:
        norm = np.power(norm, current_gamma)
    idx = (norm * 255.0).astype(np.int32)
    return current_palette[idx]

def to_rgba_matrix(matrix_db):
    denom = max(1e-3, current_db_max - current_db_min)
    norm = np.clip((matrix_db - current_db_min) / denom, 0.0, 1.0)
    if current_gamma != 1.0:
        norm = np.power(norm, current_gamma)
    idx = (norm * 255.0).astype(np.int32)
    return current_palette[idx]

col_start_wf = (np.arange(WF_COLS + 1) * WIDE_BINS // WF_COLS).astype(np.intp)
col_start_spec = (np.arange(WIDE_SPEC_PTS + 1) * WIDE_BINS // WIDE_SPEC_PTS).astype(np.intp)

def downsample_max_wf(frame):
    return np.maximum.reduceat(frame, col_start_wf[:-1])

def downsample_max_spec(frame):
    return np.maximum.reduceat(frame, col_start_spec[:-1])

# ---------- Sources de données ----------
# 1. Waterfall panoramique
wf_raw_db = np.full((WF_ROWS, WF_COLS), current_db_min, dtype=np.float32)
wf_rgba = to_rgba_matrix(wf_raw_db)
wf_src = ColumnDataSource(
    data=dict(image=[wf_rgba], x=[FREQ_MIN_MHZ], y=[0], dw=[DW_MHZ], dh=[WF_ROWS])
)

# 2. Spectre FFT panoramique global (4096 bins -> 1024 points décimés max)
x_wide_mhz = np.linspace(FREQ_MIN_MHZ, FREQ_MAX_MHZ, WIDE_SPEC_PTS)
wide_sp_src = ColumnDataSource(
    data=dict(x=x_wide_mhz, y=np.full(WIDE_SPEC_PTS, -100.0, dtype=np.float32))
)

# 3. Spectre zoom SSB (1024 bins sur 48 kHz)
x_ssb_mhz = np.linspace(ssb_min_mhz, ssb_max_mhz, SSB_BINS)
sp_src = ColumnDataSource(data=dict(x=x_ssb_mhz, y=np.zeros(SSB_BINS)))

# ---------- Widgets de contrôle VFO & Couleurs ----------
slider_vfo = Slider(
    title="Molette VFO SSB (MHz) :",
    value=current_ssb_freq / 1e6,
    start=FREQ_MIN_MHZ,
    end=FREQ_MAX_MHZ,
    step=0.0005,
    width=380,
)
vfo_input = TextInput(
    title="Fréquence exacte (MHz) :",
    value=f"{current_ssb_freq / 1e6:.4f}",
    width=160,
)

btn_m5k = Button(label="-5 kHz", width=65, button_type="primary")
btn_m1k = Button(label="-1 kHz", width=65)
btn_m500 = Button(label="-500 Hz", width=65)
btn_p500 = Button(label="+500 Hz", width=65)
btn_p1k = Button(label="+1 kHz", width=65)
btn_p5k = Button(label="+5 kHz", width=65, button_type="primary")

slider_db_min = Slider(
    title="Plancher Bruit (dB min)",
    value=current_db_min,
    start=-120.0,
    end=0.0,
    step=1.0,
    width=280,
)
slider_db_max = Slider(
    title="Plafond Signal (dB max)",
    value=current_db_max,
    start=-40.0,
    end=60.0,
    step=1.0,
    width=280,
)
slider_gamma = Slider(
    title="Contraste / Gamma",
    value=current_gamma,
    start=0.4,
    end=2.5,
    step=0.05,
    width=220,
)
select_palette = Select(
    title="Palette Waterfall",
    value="Classique SDR",
    options=list(PALETTES.keys()),
    width=200,
)

def on_color_settings_changed(attr, old, new):
    global current_db_min, current_db_max, current_gamma, current_palette, wf_rgba
    d_min = slider_db_min.value
    d_max = slider_db_max.value
    if d_min >= d_max:
        return
    current_db_min = d_min
    current_db_max = d_max
    current_gamma = slider_gamma.value
    current_palette = PALETTES.get(select_palette.value, PALETTES["Classique SDR"])
    wf_rgba = to_rgba_matrix(wf_raw_db)
    wf_src.data.update(image=[wf_rgba])

slider_db_min.on_change("value", on_color_settings_changed)
slider_db_max.on_change("value", on_color_settings_changed)
slider_gamma.on_change("value", on_color_settings_changed)
select_palette.on_change("value", on_color_settings_changed)

# ---------- Figures Bokeh ----------

# 1. Figure FFT PANORAMIQUE GLOBALE (Spectre 1D de toute la bande 2 MHz)
p_wide_fft = figure(
    title=f"Spectre FFT Panoramique 20m — (FFT 4096 bins, {FULL_BW/1e6:.1f} MHz, {FREQ_MIN_MHZ:.3f} ➔ {FREQ_MAX_MHZ:.3f} MHz) [Vue d'ensemble des canaux fsdr-hub]",
    width=1200, height=210,
    x_range=Range1d(FREQ_MIN_MHZ, FREQ_MAX_MHZ), y_range=Range1d(-105, 35),
    x_axis_label="Fréquence absolue (MHz)", y_axis_label="Puissance relative (dB)",
    toolbar_location="above",
)
p_wide_fft.xaxis.formatter = NumeralTickFormatter(format="0.000")
p_wide_fft.varea(x="x", y1=-105, y2="y", source=wide_sp_src, fill_color="#00e5ff", fill_alpha=0.10)
p_wide_fft.line(x="x", y="y", source=wide_sp_src, line_width=1.2, line_color="#00e5ff", line_alpha=0.9)

# 2. Figure WATERFALL PANORAMIQUE (Partage l'axe x_range avec le spectre ci-dessus)
p_wf = figure(
    title=f"Waterfall Panoramique 20m — (Historique temps-fréquence) [Clic pour syntoniser le VFO]",
    width=1200, height=260,
    x_range=p_wide_fft.x_range, y_range=Range1d(0, WF_ROWS),
    x_axis_label="Fréquence absolue (MHz)", y_axis_label="Temps (lignes)",
    toolbar_location="above",
)
p_wf.xaxis.formatter = NumeralTickFormatter(format="0.000")
p_wf.image_rgba(image="image", x="x", y="y", dw="dw", dh="dh", source=wf_src)

# 3. Figure ZOOM CANAL SSB (Canal déménagé et filtré centré sur le VFO)
p_sp = figure(
    title=f"Zoom Canal SSB — (FFT 1024 bins, {SSB_BW/1e3:.0f} kHz, {ssb_min_mhz:.4f} ➔ {ssb_max_mhz:.4f} MHz) [Clic pour accord fin]",
    width=1200, height=240,
    x_range=Range1d(ssb_min_mhz, ssb_max_mhz), y_range=Range1d(-90, 40),
    x_axis_label="Fréquence absolue (MHz)", y_axis_label="Puissance relative (dB)",
    toolbar_location="above",
)
p_sp.xaxis.formatter = NumeralTickFormatter(format="0.0000")
p_sp.varea(x="x", y1=-90, y2="y", source=sp_src, fill_color="#ffb300", fill_alpha=0.12)
p_sp.line(x="x", y="y", source=sp_src, line_width=1.5, line_color="#ffb300")

# ---------- Ajout des indicateurs de canaux fsdr-hub sur la FFT globale et le Waterfall ----------

# Indicateur de l'oscillateur local (LO SDR matériel)
for p in (p_wide_fft, p_wf):
    p.add_layout(Span(location=CENTER_FREQ / 1e6, dimension="height",
                      line_color="#00e5ff", line_alpha=0.8, line_width=2.0, line_dash="solid"))

p_wide_fft.add_layout(Label(x=CENTER_FREQ / 1e6, y=24, text=f"LO {CENTER_FREQ/1e6:.3f}M",
                            text_font_size="8pt", text_font_style="bold", text_color="#00e5ff",
                            background_fill_color="#101824", background_fill_alpha=0.9,
                            border_line_color="#00e5ff", border_line_alpha=0.7,
                            x_offset=-22, y_offset=0))

p_wf.add_layout(Label(x=CENTER_FREQ / 1e6, y=WF_ROWS - 20, text=f"LO {CENTER_FREQ/1e6:.3f}M",
                      text_font_size="8pt", text_font_style="bold", text_color="#00e5ff",
                      background_fill_color="#101824", background_fill_alpha=0.9,
                      border_line_color="#00e5ff", border_line_alpha=0.7,
                      x_offset=-22, y_offset=0))

# Indicateurs pour chaque canal configuré
channel_boxes_wide = {}
channel_boxes_wf = {}
channel_spans_wide = {}
channel_spans_wf = {}
channel_labels_wide = {}
channel_labels_wf = {}

for ch in CHANNELS_CFG:
    cid = ch["id"]
    col = ch["color"]
    f = ch["freq"] if not ch.get("is_vfo") else current_ssb_freq
    f_left, f_right = get_channel_bounds(ch, f)
    f_carrier = f / 1e6

    # 1. Zone de bande passante (BoxAnnotation) sur FFT panoramique
    box_wide = BoxAnnotation(left=f_left, right=f_right,
                             fill_color=col, fill_alpha=0.22,
                             line_color=col, line_alpha=0.6, line_width=1.2)
    p_wide_fft.add_layout(box_wide)
    channel_boxes_wide[cid] = box_wide

    # 2. Zone de bande passante (BoxAnnotation) sur Waterfall
    box_wf = BoxAnnotation(left=f_left, right=f_right,
                           fill_color=col, fill_alpha=0.18,
                           line_color=col, line_alpha=0.5, line_width=1.0)
    p_wf.add_layout(box_wf)
    channel_boxes_wf[cid] = box_wf

    # 3. Ligne de porteuse
    dash = "solid" if ch.get("is_vfo") else "dashed"
    span_w = Span(location=f_carrier, dimension="height",
                  line_color=col, line_alpha=0.85, line_width=1.6, line_dash=dash)
    p_wide_fft.add_layout(span_w)
    channel_spans_wide[cid] = span_w

    span_wf = Span(location=f_carrier, dimension="height",
                   line_color=col, line_alpha=0.85, line_width=1.6, line_dash=dash)
    p_wf.add_layout(span_wf)
    channel_spans_wf[cid] = span_wf

    # 4. Badge / Libellé d'identification du canal
    badge_text = f"[{ch['name']}] {f_carrier:.4f} ({ch['mode']})"
    lbl_w = Label(x=f_carrier, y=24, text=badge_text,
                  text_font_size="8pt", text_font_style="bold", text_color=col,
                  background_fill_color="#101824", background_fill_alpha=0.9,
                  border_line_color=col, border_line_alpha=0.7,
                  x_offset=-26, y_offset=0)
    p_wide_fft.add_layout(lbl_w)
    channel_labels_wide[cid] = lbl_w

    lbl_wf = Label(x=f_carrier, y=WF_ROWS - 20, text=f"{ch['name']} ({f_carrier:.4f})",
                   text_font_size="8pt", text_font_style="bold", text_color=col,
                   background_fill_color="#101824", background_fill_alpha=0.9,
                   border_line_color=col, border_line_alpha=0.7,
                   x_offset=-24, y_offset=0)
    p_wf.add_layout(lbl_wf)
    channel_labels_wf[cid] = lbl_wf

# Repères de la porteuse et de la bande audio sur la figure ZOOM SSB
span_ssb_carrier = Span(location=current_ssb_freq / 1e6, dimension="height",
                        line_color="#ffb300", line_alpha=0.9, line_width=1.8, line_dash="solid")
label_ssb_carrier = Label(x=current_ssb_freq / 1e6, y=28,
                          text=f"Porteuse SSB ({current_ssb_freq/1e6:.4f} MHz)",
                          text_font_size="9pt", text_font_style="bold", text_color="#ffb300",
                          background_fill_color="#101824", background_fill_alpha=0.9,
                          border_line_color="#ffb300", border_line_alpha=0.6,
                          x_offset=8, y_offset=0)
box_ssb_audio = BoxAnnotation(left=(current_ssb_freq + 300) / 1e6,
                              right=(current_ssb_freq + 2700) / 1e6,
                              fill_color="#4caf50", fill_alpha=0.15,
                              line_color="#4caf50", line_alpha=0.5, line_width=1.0)
span_ssb_audio_low = Span(location=(current_ssb_freq + 300) / 1e6, dimension="height",
                          line_color="#4caf50", line_alpha=0.6, line_dash="dotted")
span_ssb_audio_high = Span(location=(current_ssb_freq + 2700) / 1e6, dimension="height",
                           line_color="#4caf50", line_alpha=0.6, line_dash="dotted")
p_sp.add_layout(box_ssb_audio)
p_sp.add_layout(span_ssb_carrier)
p_sp.add_layout(label_ssb_carrier)
p_sp.add_layout(span_ssb_audio_low)
p_sp.add_layout(span_ssb_audio_high)

status = Div(text="<b>État : Déconnecté</b>", width=500)
header = Div(
    text="""
    <div style="padding: 4px 0 8px 0; border-bottom: 1px solid #273548; margin-bottom: 6px;">
        <h2 style="margin: 0; color: #00e5ff; font-family: sans-serif;">fsdr-hub — Télémétrie Spectre & Contrôle VFO Unifié</h2>
        <p style="margin: 3px 0 0 0; color: #7e8b9b; font-size: 0.85rem; font-family: sans-serif;">
            Fréquences absolues synchronisées avec CAT (Hamlib rigctld) • Indicateurs visuels des canaux fsdr-hub • Syntonisation point-and-click
        </p>
    </div>
    """,
    width=1200,
)

# Boutons de présélection rapide de canaux
quick_channel_buttons = []
for c in CHANNELS_CFG:
    btn_ch = Button(label=f"{c['name']} ({c['freq']/1e6:.3f}M)", width=130)
    btn_ch.on_click(lambda target_f=c["freq"]: request_vfo_frequency(target_f))
    quick_channel_buttons.append(btn_ch)

channel_bar = column(
    Div(text="<b style='color: #00e5ff; font-family: sans-serif;'>Accès direct aux canaux fsdr-hub (clic pour syntoniser) :</b>", width=1200),
    row(*quick_channel_buttons, spacing=8),
    spacing=3,
)

vfo_controls_box = column(
    Div(text="<b style='color: #ffca28; font-family: sans-serif;'>Contrôle VFO SSB (Bande 20m) :</b>", width=1200),
    row(vfo_input, btn_m5k, btn_m1k, btn_m500, btn_p500, btn_p1k, btn_p5k, slider_vfo, spacing=6),
    spacing=4,
)

color_controls_box = column(
    Div(text="<b style='color: #e0e6ed; font-family: sans-serif;'>Ajustement des couleurs Waterfall :</b>", width=1200),
    row(slider_db_min, slider_db_max, slider_gamma, select_palette),
    spacing=4,
)

doc = curdoc()
doc.title = "fsdr-hub FFT / Waterfall"
doc.add_root(column(header, status, channel_bar, vfo_controls_box, color_controls_box, p_wide_fft, p_wf, p_sp, spacing=10))

# ---------- Logique de mise à jour dynamique du VFO ----------
active_ws = None
is_internal_update = False

def apply_vfo_frequency(freq_hz, from_server=False):
    global current_ssb_freq, is_internal_update, x_ssb_mhz
    current_ssb_freq = float(freq_hz)
    center_mhz = current_ssb_freq / 1e6
    min_mhz = (current_ssb_freq - SSB_BW / 2.0) / 1e6
    max_mhz = (current_ssb_freq + SSB_BW / 2.0) / 1e6

    # 1. Mise à jour de l'axe et du titre du zoom SSB
    p_sp.x_range.start = min_mhz
    p_sp.x_range.end = max_mhz
    p_sp.title.text = f"Zoom Canal SSB — (FFT {SSB_BINS} bins, {SSB_BW/1e3:.0f} kHz, {min_mhz:.4f} ➔ {max_mhz:.4f} MHz) [Clic pour accord fin]"

    # 2. Recalcul des abscisses du tracé de spectre zoom
    x_ssb_mhz = np.linspace(min_mhz, max_mhz, SSB_BINS)
    sp_src.data.update(x=x_ssb_mhz)

    # 3. Déplacement des repères sur le zoom SSB
    span_ssb_carrier.location = center_mhz
    label_ssb_carrier.x = center_mhz
    label_ssb_carrier.text = f"Porteuse SSB ({center_mhz:.4f} MHz)"
    box_ssb_audio.left = (current_ssb_freq + 300) / 1e6
    box_ssb_audio.right = (current_ssb_freq + 2700) / 1e6
    span_ssb_audio_low.location = (current_ssb_freq + 300) / 1e6
    span_ssb_audio_high.location = (current_ssb_freq + 2700) / 1e6

    # 4. Déplacement de l'indicateur SSB VFO sur la FFT panoramique et le Waterfall
    ch_ssb = next(c for c in CHANNELS_CFG if c.get("is_vfo"))
    b_left, b_right = get_channel_bounds(ch_ssb, current_ssb_freq)

    channel_boxes_wide["ssb_voice_20m"].left = b_left
    channel_boxes_wide["ssb_voice_20m"].right = b_right
    channel_boxes_wf["ssb_voice_20m"].left = b_left
    channel_boxes_wf["ssb_voice_20m"].right = b_right

    channel_spans_wide["ssb_voice_20m"].location = center_mhz
    channel_spans_wf["ssb_voice_20m"].location = center_mhz

    channel_labels_wide["ssb_voice_20m"].x = center_mhz
    channel_labels_wide["ssb_voice_20m"].text = f"[SSB VFO] {center_mhz:.4f} (USB)"
    channel_labels_wf["ssb_voice_20m"].x = center_mhz
    channel_labels_wf["ssb_voice_20m"].text = f"SSB VFO ({center_mhz:.4f})"

    # 5. Mise à jour des widgets de contrôle sans redéclencher d'envoi réseau
    is_internal_update = True
    try:
        slider_vfo.value = center_mhz
        vfo_input.value = f"{center_mhz:.4f}"
    finally:
        is_internal_update = False

main_loop = None

def send_ws_command(cmd_dict):
    global active_ws, main_loop
    if active_ws is None:
        print("[bokeh] Warning: active_ws is None! Command not sent.")
        return
    if main_loop is None:
        print("[bokeh] Warning: main_loop is None! Command not sent.")
        return
    cmd_str = json.dumps(cmd_dict)
    def _do_write():
        try:
            print(f"[bokeh] Writing message to WebSocket on main loop: {cmd_str}")
            active_ws.write_message(cmd_str)
        except Exception as err:
            print(f"[bokeh] Error in write_message on main loop: {err}")
    main_loop.call_soon_threadsafe(_do_write)

def request_vfo_frequency(freq_hz):
    # Borner à la bande passante SDR disponible
    f_min_hz = CENTER_FREQ - FULL_BW / 2.0
    f_max_hz = CENTER_FREQ + FULL_BW / 2.0
    freq_hz = float(np.clip(freq_hz, f_min_hz, f_max_hz))

    print(f"[bokeh] request_vfo_frequency called: {freq_hz} Hz")
    apply_vfo_frequency(freq_hz)

    # Envoyer la commande JSON sur le WebSocket unifié vers fsdr-hub de façon thread-safe
    send_ws_command({
        "cmd": "set_frequency",
        "channel": "ssb_voice_20m",
        "frequency": int(freq_hz),
    })

def on_slider_vfo_change(attr, old, new):
    print(f"[bokeh] on_slider_vfo_change triggered: {old} -> {new}")
    if is_internal_update:
        return
    request_vfo_frequency(new * 1e6)

def on_vfo_input_change(attr, old, new):
    if is_internal_update:
        return
    try:
        val = float(new)
        if val > 1000:
            val = val / 1e6 if val > 1e6 else val / 1e3
        request_vfo_frequency(val * 1e6)
    except ValueError:
        pass

slider_vfo.on_change("value", on_slider_vfo_change)
vfo_input.on_change("value", on_vfo_input_change)

def step_vfo(delta_hz):
    request_vfo_frequency(current_ssb_freq + delta_hz)

btn_m5k.on_click(lambda: step_vfo(-5000))
btn_m1k.on_click(lambda: step_vfo(-1000))
btn_m500.on_click(lambda: step_vfo(-500))
btn_p500.on_click(lambda: step_vfo(500))
btn_p1k.on_click(lambda: step_vfo(1000))
btn_p5k.on_click(lambda: step_vfo(5000))

# Point-and-click tuning : Clic direct sur le spectre panoramique, le waterfall ou le zoom
def on_panoramic_tap(event):
    target_freq = round(event.x * 1e6 / 500) * 500
    request_vfo_frequency(target_freq)

def on_zoom_tap(event):
    target_freq = round(event.x * 1e6 / 100) * 100
    request_vfo_frequency(target_freq)

p_wide_fft.on_event(Tap, on_panoramic_tap)
p_wf.on_event(Tap, on_panoramic_tap)
p_sp.on_event(Tap, on_zoom_tap)

# ---------- Télémétrie en direct ----------
def update_waterfall(frame):
    global wf_raw_db, wf_rgba
    # Défilement temporel du waterfall
    wf_raw_db = np.roll(wf_raw_db, 1, axis=0)
    row_data = downsample_max_wf(frame)
    wf_raw_db[0] = row_data
    wf_rgba = np.roll(wf_rgba, 1, axis=0)
    wf_rgba[0] = to_rgba(row_data)
    wf_src.data.update(image=[wf_rgba])

    # Tracé instantané du spectre panoramique 1D
    spec_data = downsample_max_spec(frame)
    wide_sp_src.data.update(y=spec_data)

def update_spectrum(frame):
    sp_src.data.update(y=frame)

FRAMES = {WIDE_BINS: update_waterfall, SSB_BINS: update_spectrum}

session_doc = curdoc()
is_session_active = [True]

def on_session_destroyed(session_context):
    is_session_active[0] = False
    global active_ws
    if active_ws:
        try:
            active_ws.close()
        except Exception:
            pass

session_doc.on_session_destroyed(on_session_destroyed)

async def ws_loop():
    global active_ws, main_loop
    main_loop = asyncio.get_running_loop()
    while is_session_active[0]:
        try:
            ws = await websocket_connect(WS_URL, connect_timeout=5)
            active_ws = ws
            print(f"[bokeh] ws_loop successfully connected to {WS_URL}")
        except Exception as e:
            active_ws = None
            if is_session_active[0]:
                try:
                    session_doc.add_next_tick_callback(
                        lambda: setattr(status, "text", f"<b>Serveur fsdr-hub non joignable</b> ({e.__class__.__name__})")
                    )
                except Exception:
                    pass
            await asyncio.sleep(2)
            continue

        if is_session_active[0]:
            try:
                session_doc.add_next_tick_callback(
                    lambda: setattr(status, "text", "<b>Connecté — streaming actif (WebSocket unifié)</b>")
                )
            except Exception:
                pass

        try:
            while is_session_active[0]:
                msg = await ws.read_message()
                if msg is None:
                    break
                if isinstance(msg, str):
                    try:
                        data = json.loads(msg)
                        evt = data.get("event")
                        if evt == "state":
                            channels = data.get("channels", {})
                            if "ssb_voice_20m" in channels:
                                f = channels["ssb_voice_20m"].get("frequency")
                                if f and is_session_active[0]:
                                    session_doc.add_next_tick_callback(
                                        lambda f=float(f): apply_vfo_frequency(f, from_server=True)
                                    )
                        elif evt == "frequency_changed":
                            f = data.get("frequency")
                            ch = data.get("channel")
                            if f and (not ch or ch == "ssb_voice_20m") and is_session_active[0]:
                                session_doc.add_next_tick_callback(
                                    lambda f=float(f): apply_vfo_frequency(f, from_server=True)
                                )
                    except Exception as e:
                        print(f"[bokeh] JSON handling error: {e}")
                    continue

                a = np.frombuffer(msg, dtype="<f4")
                fn = FRAMES.get(a.size)
                if fn is not None and is_session_active[0]:
                    try:
                        session_doc.add_next_tick_callback(lambda f=fn, d=a: f(d))
                    except Exception:
                        pass
        except Exception as e:
            if is_session_active[0]:
                print(f"[bokeh] ws_loop stream notice: {e}")
        finally:
            active_ws = None
            try:
                ws.close()
            except Exception:
                pass

        if is_session_active[0]:
            try:
                session_doc.add_next_tick_callback(lambda: setattr(status, "text", "<b>Déconnecté — reconnexion...</b>"))
            except Exception:
                pass
            await asyncio.sleep(2)

IOLoop.current().add_callback(ws_loop)
