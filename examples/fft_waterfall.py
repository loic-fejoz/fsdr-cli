#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""fsdr-hub : FFT + Waterfall (application Bokeh).

Consomme le serveur WebSocket de télémétrie de fsdr-hub
(ws://127.0.0.1:5006/ws, sondes "panoramic_20m" 4096 bins et
"ssb_voice_zoom" 1024 bins, format binaire float32).

Lancement :
    .venv/bin/bokeh serve examples/fft_waterfall.py --allow-websocket-origin='*' --port 5010
puis ouvrir http://localhost:5010/fft_waterfall
"""
import asyncio
import os

import numpy as np
from tornado.ioloop import IOLoop
from tornado.websocket import websocket_connect

from bokeh.io import curdoc
from bokeh.models import ColumnDataSource, Div, Range1d, Span
from bokeh.plotting import figure

WS_URL = os.environ.get("FSDR_WS_URL", "ws://127.0.0.1:5006/ws")

WIDE_BINS = 4096   # sonde panoramique
SSB_BINS = 1024    # sonde zoom SSB
WF_COLS = 640      # largeur waterfall (downsampling 4096 -> 640)
WF_ROWS = 256      # profondeur du waterfall
DB_MIN, DB_MAX = -70.0, 30.0   # plage dB -> palette
FULL_BW = 2_000_000.0          # bande panoramique (Hz)
SSB_BW = 48_000.0              # bande zoom SSB (Hz)
CHANNELS = [(-126_000, "FT8"), (-104_400, "WSPR"), (-95_000, "APRS"),
            (30_000, "SSTV"), (50_000, "SSB")]


def make_palette():
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

PAL = make_palette()


def to_rgba(row_db):
    idx = np.clip(((row_db - DB_MIN) / (DB_MAX - DB_MIN) * 255.0).astype(np.int32), 0, 255)
    return PAL[idx]


col_start = (np.arange(WF_COLS + 1) * WIDE_BINS // WF_COLS).astype(np.intp)


def downsample_max(frame):
    return np.maximum.reduceat(frame, col_start[:-1])


# ---------- sources de données ----------
wf = np.zeros((WF_ROWS, WF_COLS), dtype=np.uint32)
wf_src = ColumnDataSource(
    data=dict(image=[wf], x=[-FULL_BW / 2], y=[0], dw=[FULL_BW], dh=[WF_ROWS])
)
x_ssb = np.linspace(0, SSB_BW, SSB_BINS)
sp_src = ColumnDataSource(data=dict(x=x_ssb, y=np.zeros(SSB_BINS)))

# ---------- figures ----------
p_wf = figure(
    title="Waterfall 20m — Panoramique (FFT 4096, 2 MHz)",
    width=1200, height=320,
    x_range=Range1d(-FULL_BW / 2, FULL_BW / 2), y_range=Range1d(0, WF_ROWS),
    x_axis_label="Fréquence relative au centre (Hz)",
    y_axis_label="Temps (lignes)", toolbar_location="above",
)
p_wf.image_rgba(image="image", x="x", y="y", dw="dw", dh="dh", source=wf_src)
for off, _name in CHANNELS:
    p_wf.add_layout(Span(location=off, dimension="height",
                         line_color="yellow", line_alpha=0.35, line_dash="dashed"))

p_sp = figure(
    title="Zoom canal SSB — (FFT 1024, 48 kHz)",
    width=1200, height=260,
    x_range=Range1d(0, SSB_BW), y_range=Range1d(-90, 40),
    x_axis_label="Fréquence (Hz)", y_axis_label="dB",
    toolbar_location="above",
)
p_sp.line(x="x", y="y", source=sp_src, line_width=1.5, line_color="#ffb300")

status = Div(text="<b>Déconnecté</b>", width=400)

doc = curdoc()
doc.title = "fsdr-hub FFT / Waterfall"
doc.add_root(status)
doc.add_root(p_wf)
doc.add_root(p_sp)


# ---------- mises à jour ----------
def update_waterfall(frame):
    global wf
    wf = np.roll(wf, 1, axis=0)
    wf[0] = to_rgba(downsample_max(frame))
    wf_src.data.update(image=[wf])


def update_spectrum(frame):
    sp_src.data.update(y=frame)


FRAMES = {WIDE_BINS: update_waterfall, SSB_BINS: update_spectrum}


async def ws_loop():
    while True:
        try:
            ws = await websocket_connect(WS_URL, connect_timeout=5)
        except Exception as e:
            doc.add_next_tick_callback(
                lambda: setattr(status, "text", f"<b>Serveur fsdr-hub non joignable</b> ({e.__class__.__name__})")
            )
            await asyncio.sleep(2)
            continue
        doc.add_next_tick_callback(lambda: setattr(status, "text", "<b>Connecté — streaming actif</b>"))
        try:
            while True:
                msg = await ws.read_message()
                if msg is None:
                    break
                if isinstance(msg, str):
                    continue
                a = np.frombuffer(msg, dtype="<f4")
                fn = FRAMES.get(a.size)
                if fn is None:
                    continue
                doc.add_next_tick_callback(lambda f=fn, d=a: f(d))
        except Exception:
            pass
        finally:
            ws.close()
        doc.add_next_tick_callback(lambda: setattr(status, "text", "<b>Déconnecté — reconnexion...</b>"))
        await asyncio.sleep(2)


IOLoop.current().add_callback(ws_loop)
