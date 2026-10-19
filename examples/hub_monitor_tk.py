#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""fsdr-hub : Moniteur Spectre & Contrôle VFO Unifié (Application locale Tkinter).

Application graphique autonome interagissant directement avec le serveur WebSocket
unifié de télémétrie de fsdr-hub (ws://127.0.0.1:5006/ws) :
- Chargement dynamique du fichier de configuration TOML (--config <path.toml>)
  garantissant la correspondance parfaite des fréquences LO, bandes passantes, canaux et sondes FFT.
- Spectre Panoramique avec repères absolus en MHz, LO, VFO et badges des canaux configurés.
- Waterfall Panoramique temps-réel avec réglage de palette, contrastes et dynamique dB.
- Spectre Zoom haute résolution / FFT précise avec marqueur de porteuse, filtre audio (USB/LSB/FM/CW),
  accord fin par simple clic ou glisser-déposer (<Button-1>, <B1-Motion>) et molette de souris.
- Waterfall Zoom haute résolution temps-réel synchronisé avec la FFT fine et le VFO.
- Les réglages Min dB, Max dB, Gamma et Palette agissent en direct sur les waterfalls ET sur
  les échelles Y des graphes FFT du haut et du bas.
- Contrôle bidirectionnel du VFO : Molette, accord fin (+/-5k, 1k, 500, 100), saisie exacte,
  accès direct aux canaux configurés et syntonisation fluide point-and-click.

Usage :
    .venv/bin/python3 examples/hub_monitor_tk.py --config examples/cfg-synthetic.toml
    .venv/bin/python3 examples/hub_monitor_tk.py --config examples/cfg-20m.toml
    .venv/bin/python3 examples/hub_monitor_tk.py --config examples/cfg-2m.toml
"""

import argparse
import asyncio
import json
import os
import queue
import re
import shutil
import subprocess
import sys
import threading
import time
import numpy as np
from PIL import Image, ImageTk

try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib
    except ImportError:
        tomllib = None

try:
    import tkinter as tk
    from tkinter import ttk
except ImportError:
    print("Erreur: Le module tkinter n'est pas disponible dans cet environnement Python.")
    sys.exit(1)

try:
    from tornado.websocket import websocket_connect
except ImportError:
    print("Erreur: Le module tornado est requis pour la connexion WebSocket.")
    sys.exit(1)


# =============================================================================
# Palettes de couleurs pour les Waterfalls
# =============================================================================

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
    r = np.interp(x, stops[:, 0], stops[:, 1]).astype(np.uint8)
    g = np.interp(x, stops[:, 0], stops[:, 2]).astype(np.uint8)
    b = np.interp(x, stops[:, 0], stops[:, 3]).astype(np.uint8)
    return np.stack([r, g, b], axis=-1)

def make_viridis_palette():
    x = np.linspace(0, 1, 256)
    r = np.clip(np.sin(x * np.pi) * 200 + 40, 0, 255).astype(np.uint8)
    g = (x * 230 + 20).astype(np.uint8)
    b = np.clip(np.cos(x * np.pi * 0.5) * 220 + 35, 0, 255).astype(np.uint8)
    return np.stack([r, g, b], axis=-1)

def make_plasma_palette():
    x = np.linspace(0, 1, 256)
    r = (x * 240 + 15).astype(np.uint8)
    g = (np.sin(x * np.pi) * 200 + 10).astype(np.uint8)
    b = ((1.0 - x) * 220 + 35).astype(np.uint8)
    return np.stack([r, g, b], axis=-1)

def make_greyscale_palette():
    x = np.arange(256, dtype=np.uint8)
    return np.stack([x, x, x], axis=-1)

PALETTES = {
    "Classique SDR": make_classic_palette(),
    "Viridis": make_viridis_palette(),
    "Plasma": make_plasma_palette(),
    "Nuances de gris": make_greyscale_palette(),
}

DEFAULT_CHANNEL_COLORS = [
    "#ff5722", "#e91e63", "#ab47bc", "#26a69a",
    "#ffca28", "#42a5f5", "#26c6da", "#66bb6a",
    "#ffa726", "#8d6e63"
]


# =============================================================================
# Gestionnaire d'Écoute Audio Directe (parec | pacat)
# =============================================================================

class AudioPlayer:
    """Gère l'écoute audio en direct du pipeline sélectionné via parec | pacat."""

    def __init__(self):
        self.p_rec = None
        self.p_play = None
        self.current_device = None
        self.current_channel_name = None
        self.enabled = True
        self.volume_pct = 80
        self.is_muted = False
        self._lock = threading.Lock()
        self._last_retry_time = 0.0

    @staticmethod
    def is_available():
        return shutil.which("parec") is not None and shutil.which("pacat") is not None

    def is_active(self):
        with self._lock:
            return (
                self.enabled
                and not self.is_muted
                and self.p_rec is not None
                and self.p_rec.poll() is None
                and self.p_play is not None
                and self.p_play.poll() is None
            )

    def set_enabled(self, enabled):
        with self._lock:
            self.enabled = enabled
            if not self.enabled:
                self._stop_process()
            else:
                self._start_process()

    def toggle_mute(self):
        with self._lock:
            self.is_muted = not self.is_muted
            if self.is_muted:
                self._stop_process()
            else:
                if self.enabled:
                    self._start_process()
            return self.is_muted

    def set_volume(self, volume_pct):
        with self._lock:
            self.volume_pct = max(0, min(150, int(volume_pct)))
            if self.enabled and not self.is_muted and self.current_device:
                self._start_process()

    def switch_device(self, device_name, channel_name=None):
        with self._lock:
            if self.current_device == device_name and self.p_rec and self.p_rec.poll() is None:
                self.current_channel_name = channel_name
                return
            self.current_device = device_name
            self.current_channel_name = channel_name
            if self.enabled and not self.is_muted:
                self._start_process()
            else:
                self._stop_process()

    def tick(self):
        """Relance automatiquement le flux si fsdr-hub a créé le sink après coup."""
        with self._lock:
            if not self.enabled or self.is_muted or not self.current_device:
                return
            now = time.time()
            if self.p_rec is None or self.p_rec.poll() is not None:
                if now - self._last_retry_time > 1.5:
                    self._last_retry_time = now
                    self._start_process()

    def _start_process(self):
        self._stop_process()
        if not self.current_device or not self.is_available() or self.is_muted or not self.enabled:
            return
        try:
            vol_int = int(65536 * (self.volume_pct / 100.0))
            self.p_rec = subprocess.Popen(
                ["parec", "-d", self.current_device, "--latency-msec=40"],
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL
            )
            self.p_play = subprocess.Popen(
                ["pacat", "--playback", "--latency-msec=40", f"--volume={vol_int}"],
                stdin=self.p_rec.stdout,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL
            )
            self.p_rec.stdout.close()
        except Exception:
            self._stop_process()

    def _stop_process(self):
        for p in (self.p_play, self.p_rec):
            if p:
                try:
                    p.terminate()
                    p.wait(timeout=0.08)
                except Exception:
                    try:
                        p.kill()
                    except Exception:
                        pass
        self.p_rec = None
        self.p_play = None

    def close(self):
        with self._lock:
            self.enabled = False
            self._stop_process()


# =============================================================================
# Analyseur de Configuration TOML fsdr-hub
# =============================================================================

def load_hub_config(config_path):
    """Charge et analyse le fichier de configuration TOML de fsdr-hub."""
    if not os.path.exists(config_path):
        raise FileNotFoundError(f"Fichier de configuration introuvable : {config_path}")

    if tomllib is None:
        raise RuntimeError("Aucun parseur TOML disponible (installer tomllib ou tomli).")

    with open(config_path, "rb") as f:
        data = tomllib.load(f)

    # 1. Matériel SDR / Fréquences de base
    sdr = data.get("hardware", {}).get("sdr", {}) or data.get("hardware", {}).get("pluto", {}) or {}
    center_freq = float(sdr.get("rx_freq", 14_200_000.0))
    sample_rate = float(sdr.get("sample_rate", 2_000_000.0))
    freq_min_hz = center_freq - sample_rate / 2.0
    freq_max_hz = center_freq + sample_rate / 2.0

    # 2. Télémétrie Bokeh / Probes
    tel = data.get("telemetry", {}).get("bokeh", {}) or {}
    listen_addr = tel.get("listen_addr", "127.0.0.1:5006")
    probes = tel.get("probes", [])

    panoramic_probe = None
    zoom_probe = None

    for p in probes:
        p_name = p.get("name", "")
        p_type = p.get("type", "")
        p_src = p.get("source_node", "")
        p_size = int(p.get("fft_size", 1024))
        p_avg = int(p.get("averaging", 1))

        if p_type == "waterfall" or p_src in ("sdr_raw_bus", "rf_corrected_bus") or p_size >= 2048:
            if panoramic_probe is None:
                panoramic_probe = {
                    "name": p_name,
                    "type": p_type,
                    "source_node": p_src,
                    "fft_size": p_size,
                    "averaging": p_avg,
                }
        else:
            if zoom_probe is None:
                ch_id = None
                if p_src.startswith("channels."):
                    parts = p_src.split(".")
                    if len(parts) >= 2:
                        ch_id = parts[1]
                zoom_probe = {
                    "name": p_name,
                    "type": p_type,
                    "source_node": p_src,
                    "fft_size": p_size,
                    "averaging": p_avg,
                    "channel_id": ch_id,
                }

    # Si une seule sonde existe et n'a pas été classée
    if not panoramic_probe and not zoom_probe and probes:
        p0 = probes[0]
        panoramic_probe = {
            "name": p0.get("name", "spectrum"),
            "type": p0.get("type", "waterfall"),
            "source_node": p0.get("source_node", "sdr_raw_bus"),
            "fft_size": int(p0.get("fft_size", 1024)),
        }

    # 3. Canaux
    raw_channels = data.get("channels", [])
    channels = []

    # Canal VFO cible par défaut (canal sondé par la sonde zoom)
    zoom_channel_id = None
    if zoom_probe and zoom_probe.get("channel_id"):
        zoom_channel_id = zoom_probe["channel_id"]

    for idx, ch in enumerate(raw_channels):
        cid = ch.get("id", f"ch_{idx}")
        f = ch.get("frequency")
        if f is None:
            fo = ch.get("frequency_offset", 0)
            f = int(center_freq + fo)
        else:
            f = int(f)

        audio_sr = (ch.get("audio") or {}).get("sample_rate", 48000)

        # Détection du sink audio RX pour l'écoute directe
        audio_cfg = ch.get("audio", {}) or {}
        raw_rx_pipe = (ch.get("rx") or {}).get("pipeline", "")
        audio_sink = None
        m = re.search(r'pulse_sink\s+["\']?([^"\'\s!]+)["\']?', raw_rx_pipe)
        if m:
            audio_sink = m.group(1).strip()
        if not audio_sink:
            audio_sink = audio_cfg.get("source_name") or audio_cfg.get("sink_name")

        audio_monitor = None
        if audio_sink:
            audio_monitor = audio_sink if audio_sink.endswith(".monitor") else f"{audio_sink}.monitor"

        # Heuristique mode
        mode = "USB"
        rx_pipe = (ch.get("rx") or {}).get("pipeline", "").lower()
        if "weaver_lsb" in rx_pipe or "lsb" in rx_pipe:
            mode = "LSB"
        elif "fm" in rx_pipe or "nbfm" in rx_pipe or "wfm" in rx_pipe or "fm" in cid.lower():
            mode = "FM"
        elif "cw" in rx_pipe or "cw" in cid.lower():
            mode = "CW"

        # Nom lisible
        name = cid
        for suffix in ["_20m", "_2m", "_6m", "_hf", "_vhf"]:
            name = name.replace(suffix, "")
        name = name.replace("_", " ").upper()

        col = DEFAULT_CHANNEL_COLORS[idx % len(DEFAULT_CHANNEL_COLORS)]
        is_vfo = (cid == zoom_channel_id) if zoom_channel_id else False

        channels.append({
            "id": cid,
            "name": name,
            "freq": f,
            "mode": mode,
            "audio_sr": audio_sr,
            "audio_sink": audio_sink,
            "audio_monitor": audio_monitor,
            "color": col,
            "is_vfo": is_vfo,
        })

    # Sélection VFO si non définie par la sonde zoom
    vfo_channel_id = zoom_channel_id
    if not vfo_channel_id and channels:
        for ch in channels:
            if any(k in ch["name"] for k in ["SSB", "VOICE", "CALL", "PHONIE"]):
                vfo_channel_id = ch["id"]
                ch["is_vfo"] = True
                break
        if not vfo_channel_id:
            vfo_channel_id = channels[0]["id"]
            channels[0]["is_vfo"] = True
    else:
        for ch in channels:
            if ch["id"] == vfo_channel_id:
                ch["is_vfo"] = True
                break

    # Fréquence initiale du VFO et bande passante zoom
    initial_vfo_freq = center_freq
    zoom_bw = 48000.0
    vfo_mode = "USB"
    for ch in channels:
        if ch["id"] == vfo_channel_id:
            initial_vfo_freq = float(ch["freq"])
            zoom_bw = float(ch.get("audio_sr", 48000))
            vfo_mode = ch.get("mode", "USB")
            break

    # Monitor audio initial
    initial_audio_monitor = None
    initial_audio_name = None
    target_id = zoom_channel_id or vfo_channel_id
    for ch in channels:
        if ch["id"] == target_id and ch.get("audio_monitor"):
            initial_audio_monitor = ch["audio_monitor"]
            initial_audio_name = ch["name"]
            break
    if not initial_audio_monitor:
        for ch in channels:
            if ch.get("audio_monitor"):
                initial_audio_monitor = ch["audio_monitor"]
                initial_audio_name = ch["name"]
                break

    return {
        "config_path": config_path,
        "center_freq": center_freq,
        "sample_rate": sample_rate,
        "freq_min_hz": freq_min_hz,
        "freq_max_hz": freq_max_hz,
        "listen_addr": listen_addr,
        "panoramic_probe": panoramic_probe,
        "zoom_probe": zoom_probe,
        "channels": channels,
        "vfo_channel_id": vfo_channel_id,
        "zoom_channel_id": zoom_channel_id or vfo_channel_id,
        "initial_vfo_freq": initial_vfo_freq,
        "zoom_bw": zoom_bw,
        "vfo_mode": vfo_mode,
        "initial_audio_monitor": initial_audio_monitor,
        "initial_audio_name": initial_audio_name,
    }


# =============================================================================
# Client Réseau WebSocket fsdr-hub
# =============================================================================

class FsdrHubClient(threading.Thread):
    def __init__(self, ws_url, inbound_queue):
        super().__init__(daemon=True)
        self.ws_url = ws_url
        self.inbound_queue = inbound_queue
        self.send_queue = queue.Queue()
        self.running = True
        self.loop = None
        self.active_ws = None

    def send_command(self, cmd_dict):
        self.send_queue.put(cmd_dict)
        if self.loop and self.loop.is_running():
            self.loop.call_soon_threadsafe(lambda: None)

    def stop(self):
        self.running = False
        if self.active_ws:
            try:
                self.active_ws.close()
            except Exception:
                pass

    def run(self):
        self.loop = asyncio.new_event_loop()
        asyncio.set_event_loop(self.loop)
        self.loop.run_until_complete(self._main_ws_task())

    async def _main_ws_task(self):
        while self.running:
            self.inbound_queue.put(("status", f"Connexion à {self.ws_url}..."))
            try:
                self.active_ws = await websocket_connect(self.ws_url, connect_timeout=4)
                self.inbound_queue.put(("status", "Connecté à fsdr-hub (WebSocket unifié)"))
            except Exception as e:
                self.active_ws = None
                self.inbound_queue.put(("status", f"Déconnecté (reconnexion...) : {e.__class__.__name__}"))
                await asyncio.sleep(2)
                continue

            recv_task = asyncio.create_task(self._recv_loop())
            send_task = asyncio.create_task(self._send_loop())
            done, pending = await asyncio.wait([recv_task, send_task], return_when=asyncio.FIRST_COMPLETED)
            for p in pending:
                p.cancel()
            self.active_ws = None
            await asyncio.sleep(1)

    async def _recv_loop(self):
        while self.running and self.active_ws:
            msg = await self.active_ws.read_message()
            if msg is None:
                break
            if isinstance(msg, bytes):
                floats = np.frombuffer(msg, dtype="<f4")
                self.inbound_queue.put(("frame", floats))
            elif isinstance(msg, str):
                try:
                    data = json.loads(msg)
                    self.inbound_queue.put(("event", data))
                except Exception:
                    pass

    async def _send_loop(self):
        while self.running and self.active_ws:
            while not self.send_queue.empty():
                cmd = self.send_queue.get_nowait()
                try:
                    cmd_str = json.dumps(cmd)
                    await self.active_ws.write_message(cmd_str)
                except Exception:
                    pass
            await asyncio.sleep(0.02)


# =============================================================================
# Application Tkinter Principale
# =============================================================================

class FsdrTkApp:
    def __init__(self, root, config_data, custom_ws_url=None):
        self.root = root
        self.cfg = config_data

        # Paramètres SDR & Fréquences
        self.center_freq = self.cfg["center_freq"]
        self.sample_rate = self.cfg["sample_rate"]
        self.freq_min_hz = self.cfg["freq_min_hz"]
        self.freq_max_hz = self.cfg["freq_max_hz"]
        self.freq_min_mhz = self.freq_min_hz / 1e6
        self.freq_max_mhz = self.freq_max_hz / 1e6

        # Sondes & Canaux
        self.panoramic_probe = self.cfg["panoramic_probe"]
        self.zoom_probe = self.cfg["zoom_probe"]
        self.channels = self.cfg["channels"]
        self.zoom_channel_id = self.cfg.get("zoom_channel_id") or self.cfg["vfo_channel_id"]
        self.vfo_channel_id = self.zoom_channel_id
        self.current_vfo_hz = self.cfg["initial_vfo_freq"]
        self.zoom_bw = self.cfg["zoom_bw"]
        self.vfo_mode = self.cfg["vfo_mode"]

        self.panoramic_fft_size = self.panoramic_probe["fft_size"] if self.panoramic_probe else None
        self.zoom_fft_size = self.zoom_probe["fft_size"] if self.zoom_probe else None

        # Configuration de l'affichage
        cfg_name = os.path.basename(self.cfg["config_path"])
        self.root.title(f"fsdr-hub — Moniteur Spectre & Contrôle VFO [{cfg_name}]")
        self.root.geometry("1100x880")
        self.root.minsize(950, 720)
        self.root.configure(bg="#0b1118")

        # Dynamique & Couleurs (Agissent sur Waterfalls ET Graphes FFT)
        self.db_min = -75.0
        self.db_max = 25.0
        self.gamma = 1.0
        self.palette = PALETTES["Classique SDR"]

        # Waterfall Panoramique
        self.wf_cols = 1000
        self.wf_rows = 160
        self.wf_matrix_db = np.full((self.wf_rows, self.wf_cols), self.db_min, dtype=np.float32)
        self.wf_photo = None

        # Waterfall Zoom FFT Fine
        self.zoom_wf_cols = 1000
        self.zoom_wf_rows = 160
        self.zoom_wf_matrix_db = np.full((self.zoom_wf_rows, self.zoom_wf_cols), self.db_min, dtype=np.float32)
        self.zoom_wf_photo = None

        # Trames de spectre
        self.latest_wide_frame = None
        self.latest_zoom_frame = None
        self.cached_wide_frame = None
        self.cached_zoom_frame = None

        # Widgets
        self.canvas_wide = None
        self.canvas_wf = None
        self.canvas_zoom = None
        self.canvas_zoom_wf = None
        self.scale_vfo = None
        self.scale_min = None
        self.scale_max = None
        self.scale_gamma = None

        self.inbound_queue = queue.Queue()

        # Écoute Audio Directe
        self.audio_player = AudioPlayer()
        init_dev = self.cfg.get("initial_audio_monitor")
        init_name = self.cfg.get("initial_audio_name")
        if init_dev:
            self.audio_player.switch_device(init_dev, channel_name=init_name)

        self._build_ui()

        # Démarrage WebSocket
        ws_url = custom_ws_url or os.environ.get("FSDR_WS_URL") or f"ws://{self.cfg['listen_addr']}/ws"
        self.ws_client = FsdrHubClient(ws_url, self.inbound_queue)
        self.ws_client.start()

        # Nettoyage à la fermeture de la fenêtre
        self.root.protocol("WM_DELETE_WINDOW", self._on_close)

        # Boucle d'animation fluide à ~25 FPS (40 ms)
        self.root.after(40, self._render_loop)

    def _build_ui(self):
        style = ttk.Style()
        style.theme_use("clam")
        style.configure(".", background="#0b1118", foreground="#e0e6ed", font=("Sans", 9))
        style.configure("Header.TLabel", font=("Sans", 11, "bold"), foreground="#00e5ff")
        style.configure("Status.TLabel", font=("Sans", 9), foreground="#7e8b9b")
        style.configure("Vfo.TLabel", font=("Sans", 12, "bold"), foreground="#ffca28")
        style.configure("TButton", font=("Sans", 9), background="#1b2533", foreground="#ffffff")
        style.map("TButton", background=[("active", "#2a3b50")])

        # 1. En-tête : Titre & Statut de connexion
        hdr_frame = ttk.Frame(self.root, padding="8 4 8 4")
        hdr_frame.pack(fill=tk.X)
        cfg_name = os.path.basename(self.cfg["config_path"])
        lo_str = f"LO: {self.center_freq/1e6:.3f} MHz | BW: {self.sample_rate/1e6:.1f} MHz"
        self.lbl_title = ttk.Label(hdr_frame, text=f"fsdr-hub — {cfg_name} ({lo_str})", style="Header.TLabel")
        self.lbl_title.pack(side=tk.LEFT)
        self.lbl_status = ttk.Label(hdr_frame, text="Initialisation...", style="Status.TLabel")
        self.lbl_status.pack(side=tk.RIGHT)

        # 2. Boutons de présélection des canaux
        if self.channels:
            ch_frame = ttk.Frame(self.root, padding="8 2 8 4")
            ch_frame.pack(fill=tk.X)
            ttk.Label(ch_frame, text="Canaux :", font=("Sans", 9, "bold"), foreground="#00e5ff").pack(side=tk.LEFT, padx=(0, 6))
            for c in self.channels:
                btn = tk.Button(
                    ch_frame, text=f"{c['name']} ({c['freq']/1e6:.3f}M)",
                    bg=c["color"], fg="#000000", font=("Sans", 8, "bold"),
                    activebackground="#ffffff", relief=tk.FLAT, padx=6, pady=2,
                    command=lambda ch=c: self._on_channel_btn(ch)
                )
                btn.pack(side=tk.LEFT, padx=3)

        # 3. Contrôle VFO (Boutons + Slider + Saisie)
        vfo_box = ttk.Frame(self.root, padding="8 4 8 4")
        vfo_box.pack(fill=tk.X)

        self.lbl_vfo = ttk.Label(
            vfo_box,
            text=f"VFO [{self.vfo_channel_id}] : {self.current_vfo_hz/1e6:.4f} MHz",
            style="Vfo.TLabel"
        )
        self.lbl_vfo.pack(side=tk.LEFT, padx=(0, 10))

        self.entry_vfo = ttk.Entry(vfo_box, width=12, font=("Sans", 10))
        self.entry_vfo.insert(0, f"{self.current_vfo_hz/1e6:.4f}")
        self.entry_vfo.pack(side=tk.LEFT, padx=(0, 6))
        self.entry_vfo.bind("<Return>", self._on_entry_vfo)

        for delta in [-5000, -1000, -500, 500, 1000, 5000]:
            label = f"{delta:+d} Hz" if abs(delta) < 1000 else f"{delta//1000:+d}k"
            btn = ttk.Button(vfo_box, text=label, width=5, command=lambda d=delta: self.step_vfo(d))
            btn.pack(side=tk.LEFT, padx=1)

        self.scale_vfo = ttk.Scale(
            vfo_box, from_=self.freq_min_mhz, to=self.freq_max_mhz, orient=tk.HORIZONTAL,
            command=self._on_scale_vfo
        )
        self.scale_vfo.set(self.current_vfo_hz / 1e6)
        self.scale_vfo.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=8)

        # 4. Écoute Audio Directe (parec | pacat)
        audio_box = ttk.Frame(self.root, padding="8 2 8 4")
        audio_box.pack(fill=tk.X)

        self.audio_available = AudioPlayer.is_available()
        self.audio_enabled_var = tk.BooleanVar(value=self.audio_available)

        self.chk_audio = ttk.Checkbutton(
            audio_box,
            text="🔊 Écoute Audio",
            variable=self.audio_enabled_var,
            command=self._on_toggle_audio
        )
        if not self.audio_available:
            self.chk_audio.config(state="disabled")
        self.chk_audio.pack(side=tk.LEFT, padx=(0, 8))

        ttk.Label(audio_box, text="Canal :", font=("Sans", 9, "bold")).pack(side=tk.LEFT, padx=(0, 4))

        self.audio_channels = [c for c in self.channels if c.get("audio_monitor")]
        self.audio_channel_labels = [f"{c['name']} [{c['audio_monitor']}]" for c in self.audio_channels]
        self.audio_channel_map = {f"{c['name']} [{c['audio_monitor']}]": c for c in self.audio_channels}

        self.combo_audio = ttk.Combobox(
            audio_box,
            values=self.audio_channel_labels,
            state="readonly" if self.audio_available else "disabled",
            width=28
        )
        init_audio_idx = 0
        target_audio_ch = self.cfg.get("initial_audio_monitor")
        for i, c in enumerate(self.audio_channels):
            if c.get("audio_monitor") == target_audio_ch:
                init_audio_idx = i
                break
        if self.audio_channel_labels:
            self.combo_audio.current(init_audio_idx)
        self.combo_audio.bind("<<ComboboxSelected>>", self._on_combo_audio_select)
        self.combo_audio.pack(side=tk.LEFT, padx=(0, 10))

        # Indicateur d'état du flux audio
        self.lbl_audio_status = ttk.Label(
            audio_box,
            text="Connexion...",
            font=("Sans", 9),
            foreground="#26a69a" if self.audio_available else "#e53935"
        )
        self.lbl_audio_status.pack(side=tk.LEFT, padx=(0, 12))

        # Volume
        ttk.Label(audio_box, text="Vol :").pack(side=tk.LEFT, padx=(0, 2))
        self.lbl_volume = ttk.Label(audio_box, text="80%", width=5)
        self.scale_volume = ttk.Scale(
            audio_box,
            from_=0,
            to=150,
            orient=tk.HORIZONTAL,
            length=100,
            command=self._on_volume_change
        )
        self.scale_volume.pack(side=tk.LEFT, padx=(0, 4))
        self.lbl_volume.pack(side=tk.LEFT, padx=(0, 6))
        self.scale_volume.set(80)

        # Bouton Mute
        self.btn_mute = ttk.Button(
            audio_box,
            text="🔇 Mute",
            width=7,
            command=self._on_btn_mute
        )
        self.btn_mute.pack(side=tk.LEFT)

        # 5. Ajustements Dynamique & Couleurs (FFT + Waterfalls)
        col_box = ttk.Frame(self.root, padding="8 2 8 4")
        col_box.pack(fill=tk.X)
        ttk.Label(col_box, text="Min (dB):").pack(side=tk.LEFT, padx=(0, 2))
        self.scale_min = ttk.Scale(col_box, from_=-120, to=0, orient=tk.HORIZONTAL, length=110, command=self._on_col_scale)
        self.scale_min.set(self.db_min)
        self.scale_min.pack(side=tk.LEFT, padx=(0, 8))

        ttk.Label(col_box, text="Max (dB):").pack(side=tk.LEFT, padx=(0, 2))
        self.scale_max = ttk.Scale(col_box, from_=-30, to=60, orient=tk.HORIZONTAL, length=110, command=self._on_col_scale)
        self.scale_max.set(self.db_max)
        self.scale_max.pack(side=tk.LEFT, padx=(0, 8))

        ttk.Label(col_box, text="Gamma:").pack(side=tk.LEFT, padx=(0, 2))
        self.scale_gamma = ttk.Scale(col_box, from_=0.4, to=2.5, orient=tk.HORIZONTAL, length=80, command=self._on_col_scale)
        self.scale_gamma.set(self.gamma)
        self.scale_gamma.pack(side=tk.LEFT, padx=(0, 8))

        ttk.Label(col_box, text="Palette:").pack(side=tk.LEFT, padx=(0, 2))
        self.palette_combo = ttk.Combobox(col_box, values=list(PALETTES.keys()), state="readonly", width=14)
        self.palette_combo.current(0)
        self.palette_combo.bind("<<ComboboxSelected>>", self._on_palette_change)
        self.palette_combo.pack(side=tk.LEFT)

        # 5. Canevas FFT Panoramique
        if self.panoramic_probe:
            self.canvas_wide = tk.Canvas(self.root, bg="#101824", highlightthickness=1, highlightbackground="#273548", height=110)
            self.canvas_wide.pack(fill=tk.X, padx=8, pady=(4, 1))
            self.canvas_wide.bind("<Button-1>", self._on_wide_click)
            self.canvas_wide.bind("<B1-Motion>", self._on_wide_drag)
            self.canvas_wide.bind("<Motion>", self._on_wide_motion)
            self.canvas_wide.bind("<Leave>", self._on_wide_leave)
            self.canvas_wide.bind("<MouseWheel>", self._on_wide_wheel)
            self.canvas_wide.bind("<Button-4>", self._on_wide_wheel)
            self.canvas_wide.bind("<Button-5>", self._on_wide_wheel)

            # 6. Canevas Waterfall Panoramique
            self.canvas_wf = tk.Canvas(self.root, bg="#000000", highlightthickness=1, highlightbackground="#273548", height=130)
            self.canvas_wf.pack(fill=tk.X, padx=8, pady=1)
            self.canvas_wf.bind("<Button-1>", self._on_wide_click)
            self.canvas_wf.bind("<B1-Motion>", self._on_wide_drag)
            self.canvas_wf.bind("<Motion>", self._on_wide_motion)
            self.canvas_wf.bind("<Leave>", self._on_wide_leave)
            self.canvas_wf.bind("<MouseWheel>", self._on_wide_wheel)
            self.canvas_wf.bind("<Button-4>", self._on_wide_wheel)
            self.canvas_wf.bind("<Button-5>", self._on_wide_wheel)

        # 7. Canevas Zoom FFT Haute Résolution & Waterfall Fin
        if self.zoom_probe:
            self.canvas_zoom = tk.Canvas(self.root, bg="#101824", highlightthickness=1, highlightbackground="#273548", height=110)
            self.canvas_zoom.pack(fill=tk.X, padx=8, pady=(3, 1))
            self.canvas_zoom.bind("<Button-1>", self._on_zoom_click)
            self.canvas_zoom.bind("<B1-Motion>", self._on_zoom_drag)
            self.canvas_zoom.bind("<Motion>", self._on_zoom_motion)
            self.canvas_zoom.bind("<Leave>", self._on_zoom_leave)
            self.canvas_zoom.bind("<MouseWheel>", self._on_zoom_wheel)
            self.canvas_zoom.bind("<Button-4>", self._on_zoom_wheel)
            self.canvas_zoom.bind("<Button-5>", self._on_zoom_wheel)

            # 8. Canevas Waterfall Zoom FFT Fin
            self.canvas_zoom_wf = tk.Canvas(self.root, bg="#000000", highlightthickness=1, highlightbackground="#273548", height=130)
            self.canvas_zoom_wf.pack(fill=tk.X, padx=8, pady=(1, 4))
            self.canvas_zoom_wf.bind("<Button-1>", self._on_zoom_click)
            self.canvas_zoom_wf.bind("<B1-Motion>", self._on_zoom_drag)
            self.canvas_zoom_wf.bind("<Motion>", self._on_zoom_motion_wf)
            self.canvas_zoom_wf.bind("<Leave>", self._on_zoom_leave_wf)
            self.canvas_zoom_wf.bind("<MouseWheel>", self._on_zoom_wheel)
            self.canvas_zoom_wf.bind("<Button-4>", self._on_zoom_wheel)
            self.canvas_zoom_wf.bind("<Button-5>", self._on_zoom_wheel)
        else:
            note_frame = ttk.Frame(self.root, padding="10")
            note_frame.pack(fill=tk.X)
            ttk.Label(
                note_frame,
                text=f"(Note: Aucune sonde FFT zoom définie dans '{cfg_name}' - Affichage panoramique seul actif)",
                style="Status.TLabel"
            ).pack()

    # ---------- Gestion des interactions utilisateur ----------

    def _on_channel_btn(self, ch_cfg):
        """Sélectionne un canal, règle le VFO sur sa fréquence et bascule l'écoute audio."""
        self.vfo_channel_id = ch_cfg["id"]
        self.vfo_mode = ch_cfg.get("mode", "USB")
        if ch_cfg.get("audio_sr"):
            self.zoom_bw = float(ch_cfg["audio_sr"])
        # Toujours propager le changement au canal zoom s'il correspond au mode audio
        target_ch = self.zoom_channel_id or ch_cfg["id"]
        self.set_vfo_frequency(ch_cfg["freq"], channel_id=target_ch)

        # Basculer automatiquement l'écoute audio sur ce canal
        self._switch_audio_to_channel(ch_cfg)

    # ---------- Gestion de l'écoute audio directe ----------

    def _on_toggle_audio(self):
        enabled = self.audio_enabled_var.get()
        self.audio_player.set_enabled(enabled)
        self._update_audio_status_ui()

    def _on_combo_audio_select(self, _=None):
        selected_text = self.combo_audio.get()
        ch = self.audio_channel_map.get(selected_text)
        if ch:
            self.audio_player.switch_device(ch.get("audio_monitor"), channel_name=ch.get("name"))
            self._update_audio_status_ui()

    def _on_volume_change(self, val):
        if not hasattr(self, "lbl_volume") or self.lbl_volume is None:
            return
        vol = int(float(val))
        self.lbl_volume.config(text=f"{vol}%")
        if hasattr(self, "audio_player"):
            self.audio_player.set_volume(vol)

    def _on_btn_mute(self):
        is_muted = self.audio_player.toggle_mute()
        if is_muted:
            self.btn_mute.config(text="🔊 Écouter")
        else:
            self.btn_mute.config(text="🔇 Mute")
        self._update_audio_status_ui()

    def _switch_audio_to_channel(self, ch_cfg):
        if not ch_cfg:
            return
        mon = ch_cfg.get("audio_monitor")
        name = ch_cfg.get("name")
        if mon:
            self.audio_player.switch_device(mon, channel_name=name)
            for idx, label in enumerate(self.audio_channel_labels):
                c = self.audio_channel_map.get(label)
                if c and c.get("id") == ch_cfg["id"]:
                    self.combo_audio.current(idx)
                    break
            self._update_audio_status_ui()

    def _update_audio_status_ui(self):
        if not self.audio_available:
            self.lbl_audio_status.config(text="(parec/pacat non disponibles)", foreground="#e53935")
            return
        if not self.audio_player.enabled:
            self.lbl_audio_status.config(text="○ Audio inactif", foreground="#7e8b9b")
        elif self.audio_player.is_muted:
            self.lbl_audio_status.config(text="🔇 Muet", foreground="#ffa726")
        elif self.audio_player.is_active():
            dev = self.audio_player.current_device or ""
            ch_name = self.audio_player.current_channel_name or ""
            txt = f"● Écoute : {ch_name} ({dev})" if ch_name else f"● Écoute : {dev}"
            self.lbl_audio_status.config(text=txt, foreground="#26a69a")
        else:
            dev = self.audio_player.current_device or ""
            self.lbl_audio_status.config(text=f"◌ En attente ({dev})...", foreground="#ffb74d")

    def set_vfo_frequency(self, freq_hz, channel_id=None):
        """Met à jour la fréquence du VFO et transmet l'ordre au canal cible dans fsdr-hub."""
        target_ch = channel_id or self.zoom_channel_id or self.vfo_channel_id
        self.current_vfo_hz = float(np.clip(freq_hz, self.freq_min_hz, self.freq_max_hz))

        # Envoi de la commande sur le WebSocket unifié vers fsdr-hub
        self.ws_client.send_command({
            "cmd": "set_frequency",
            "channel": target_ch,
            "frequency": int(self.current_vfo_hz),
        })

        self._update_vfo_widgets()

        # Rafraîchissement visuel instantané avec les trames en cache
        if self.cached_wide_frame is not None and self.canvas_wide is not None:
            self._draw_wide_fft(self.cached_wide_frame)
            self._draw_waterfall()
        if self.cached_zoom_frame is not None and self.canvas_zoom is not None:
            self._draw_zoom_fft(self.cached_zoom_frame)
        if self.canvas_zoom_wf is not None:
            self._draw_zoom_waterfall()

    def step_vfo(self, delta_hz):
        self.set_vfo_frequency(self.current_vfo_hz + delta_hz, channel_id=self.zoom_channel_id)

    def _on_entry_vfo(self, event):
        try:
            val = float(self.entry_vfo.get().strip().replace(",", "."))
            if val < 1000:
                val *= 1e6
            self.set_vfo_frequency(val, channel_id=self.zoom_channel_id)
        except ValueError:
            pass

    def _on_scale_vfo(self, val):
        f_mhz = float(val)
        f_hz = f_mhz * 1e6
        if abs(f_hz - self.current_vfo_hz) > 100:
            self.set_vfo_frequency(f_hz, channel_id=self.zoom_channel_id)

    def _on_col_scale(self, _=None):
        """Ajustement dynamique Min/Max/Gamma : répercute sur waterfalls ET graphes FFT."""
        if self.scale_min is None or self.scale_max is None or self.scale_gamma is None:
            return
        self.db_min = float(self.scale_min.get())
        self.db_max = max(self.db_min + 5.0, float(self.scale_max.get()))
        self.gamma = float(self.scale_gamma.get())

        # Rafraîchissement immédiat de tous les graphes FFT et waterfalls
        if self.cached_wide_frame is not None and self.canvas_wide:
            self._draw_wide_fft(self.cached_wide_frame)
            self._draw_waterfall()
        if self.cached_zoom_frame is not None and self.canvas_zoom:
            self._draw_zoom_fft(self.cached_zoom_frame)
        if self.canvas_zoom_wf:
            self._draw_zoom_waterfall()

    def _on_palette_change(self, _=None):
        pal_name = self.palette_combo.get()
        self.palette = PALETTES.get(pal_name, PALETTES["Classique SDR"])
        if self.canvas_wf:
            self._draw_waterfall()
        if self.canvas_zoom_wf:
            self._draw_zoom_waterfall()

    # --- Interactions Panoramiques (Clic, Glisser, Molette, Survol) ---

    def _on_wide_click(self, event):
        widget = getattr(event, "widget", self.canvas_wide)
        width = widget.winfo_width() if widget else (self.canvas_wide.winfo_width() if self.canvas_wide else 0)
        if width <= 0:
            return
        ratio = max(0.0, min(1.0, event.x / float(width)))
        target_f = self.freq_min_hz + ratio * (self.freq_max_hz - self.freq_min_hz)
        target_f = round(target_f / 500.0) * 500.0
        self.set_vfo_frequency(target_f, channel_id=self.zoom_channel_id)

    def _on_wide_drag(self, event):
        self._on_wide_click(event)

    def _on_wide_wheel(self, event):
        delta = 0
        if event.num == 4 or (hasattr(event, "delta") and event.delta > 0):
            delta = 1000.0
        elif event.num == 5 or (hasattr(event, "delta") and event.delta < 0):
            delta = -1000.0
        if delta != 0:
            self.step_vfo(delta)

    def _on_wide_motion(self, event):
        if not self.canvas_wide:
            return
        width = self.canvas_wide.winfo_width()
        height = self.canvas_wide.winfo_height()
        if width <= 0:
            return
        x = max(0, min(width, event.x))
        ratio = x / float(width)
        f_hz = self.freq_min_hz + ratio * (self.freq_max_hz - self.freq_min_hz)

        self.canvas_wide.delete("crosshair")
        self.canvas_wide.create_line(x, 0, x, height, fill="#ffffff", dash=(2, 2), tags="crosshair")
        tip_text = f"{f_hz/1e6:.4f} MHz"
        anchor = tk.NE if x > width - 130 else tk.NW
        tx = x - 6 if x > width - 130 else x + 6
        self.canvas_wide.create_text(tx, 14, text=tip_text, fill="#ffffff", font=("Sans", 8, "bold"), anchor=anchor, tags="crosshair")

    def _on_wide_leave(self, event):
        if self.canvas_wide:
            self.canvas_wide.delete("crosshair")

    # --- Interactions Zoom / FFT Précise (Clic, Glisser, Molette, Survol) ---

    def _on_zoom_click(self, event):
        """Accord fin par clic ou glisser sur la FFT précise ou son waterfall."""
        if not self.canvas_zoom or self.zoom_bw <= 0:
            return
        widget = getattr(event, "widget", self.canvas_zoom)
        width = widget.winfo_width() if widget else self.canvas_zoom.winfo_width()
        if width <= 0:
            return
        ratio = max(0.0, min(1.0, event.x / float(width)))
        # La FFT précise est centrée sur DC = current_vfo_hz
        zoom_min_hz = self.current_vfo_hz - self.zoom_bw / 2.0
        target_f = zoom_min_hz + ratio * self.zoom_bw
        # Accord fin au pas de 50 Hz
        target_f = round(target_f / 50.0) * 50.0
        # Toujours appliquer le changement de fréquence sur le canal écouté par la FFT zoom
        self.set_vfo_frequency(target_f, channel_id=self.zoom_channel_id)

    def _on_zoom_drag(self, event):
        self._on_zoom_click(event)

    def _on_zoom_wheel(self, event):
        delta = 0
        if event.num == 4 or (hasattr(event, "delta") and event.delta > 0):
            delta = 100.0
        elif event.num == 5 or (hasattr(event, "delta") and event.delta < 0):
            delta = -100.0
        if delta != 0:
            self.step_vfo(delta)

    def _on_zoom_motion(self, event):
        if not self.canvas_zoom or self.zoom_bw <= 0:
            return
        width = self.canvas_zoom.winfo_width()
        height = self.canvas_zoom.winfo_height()
        if width <= 0:
            return
        x = max(0, min(width, event.x))
        ratio = x / float(width)
        f_hz = (self.current_vfo_hz - self.zoom_bw / 2.0) + ratio * self.zoom_bw
        offset_hz = (ratio - 0.5) * self.zoom_bw

        self.canvas_zoom.delete("crosshair")
        self.canvas_zoom.create_line(x, 0, x, height, fill="#ffffff", dash=(2, 2), tags="crosshair")
        tip_text = f"{f_hz/1e6:.4f} MHz ({offset_hz/1e3:+.2f} kHz)"
        anchor = tk.NE if x > width - 180 else tk.NW
        tx = x - 6 if x > width - 180 else x + 6
        self.canvas_zoom.create_text(tx, 14, text=tip_text, fill="#ffffff", font=("Sans", 8, "bold"), anchor=anchor, tags="crosshair")

    def _on_zoom_leave(self, event):
        if self.canvas_zoom:
            self.canvas_zoom.delete("crosshair")

    def _on_zoom_motion_wf(self, event):
        if not self.canvas_zoom_wf or self.zoom_bw <= 0:
            return
        width = self.canvas_zoom_wf.winfo_width()
        height = self.canvas_zoom_wf.winfo_height()
        if width <= 0:
            return
        x = max(0, min(width, event.x))
        ratio = x / float(width)
        f_hz = (self.current_vfo_hz - self.zoom_bw / 2.0) + ratio * self.zoom_bw
        offset_hz = (ratio - 0.5) * self.zoom_bw

        self.canvas_zoom_wf.delete("crosshair")
        self.canvas_zoom_wf.create_line(x, 0, x, height, fill="#ffffff", dash=(2, 2), tags="crosshair")
        tip_text = f"{f_hz/1e6:.4f} MHz ({offset_hz/1e3:+.2f} kHz)"
        anchor = tk.NE if x > width - 180 else tk.NW
        tx = x - 6 if x > width - 180 else x + 6
        self.canvas_zoom_wf.create_text(tx, 14, text=tip_text, fill="#ffffff", font=("Sans", 8, "bold"), anchor=anchor, tags="crosshair")

    def _on_zoom_leave_wf(self, event):
        if self.canvas_zoom_wf:
            self.canvas_zoom_wf.delete("crosshair")

    def _update_vfo_widgets(self):
        mhz = self.current_vfo_hz / 1e6
        ch_id = self.zoom_channel_id or self.vfo_channel_id
        self.lbl_vfo.config(text=f"VFO [{ch_id}] : {mhz:.4f} MHz")
        self.entry_vfo.delete(0, tk.END)
        self.entry_vfo.insert(0, f"{mhz:.4f}")
        if self.scale_vfo:
            self.scale_vfo.set(mhz)

    def _handle_event(self, data):
        evt = data.get("event")
        if evt == "state":
            channels_state = data.get("channels", {})
            for ch in self.channels:
                cid = ch["id"]
                if cid in channels_state:
                    f = channels_state[cid].get("frequency")
                    if f:
                        ch["freq"] = float(f)
            target_ch = self.zoom_channel_id or self.vfo_channel_id
            if target_ch in channels_state:
                f = channels_state[target_ch].get("frequency")
                if f:
                    self._apply_external_vfo(float(f))
        elif evt == "frequency_changed":
            f = data.get("frequency")
            ch_id = data.get("channel")
            if f:
                for ch in self.channels:
                    if ch["id"] == ch_id:
                        ch["freq"] = float(f)
                target_ch = self.zoom_channel_id or self.vfo_channel_id
                if ch_id == target_ch or not ch_id:
                    self._apply_external_vfo(float(f))

    def _apply_external_vfo(self, freq_hz):
        self.current_vfo_hz = freq_hz
        self._update_vfo_widgets()
        if self.cached_wide_frame is not None and self.canvas_wide:
            self._draw_wide_fft(self.cached_wide_frame)
            self._draw_waterfall()
        if self.cached_zoom_frame is not None and self.canvas_zoom:
            self._draw_zoom_fft(self.cached_zoom_frame)
        if self.canvas_zoom_wf:
            self._draw_zoom_waterfall()

    # ---------- Boucle de rendu graphique Tkinter ----------

    def _render_loop(self):
        # 1. Traitement de la file de messages réseau
        while not self.inbound_queue.empty():
            try:
                kind, payload = self.inbound_queue.get_nowait()
                if kind == "frame":
                    n_bins = len(payload)
                    if self.panoramic_fft_size and n_bins == self.panoramic_fft_size:
                        self.latest_wide_frame = payload
                        self.cached_wide_frame = payload
                    elif self.zoom_fft_size and n_bins == self.zoom_fft_size:
                        self.latest_zoom_frame = payload
                        self.cached_zoom_frame = payload
                    elif not self.zoom_fft_size:
                        self.latest_wide_frame = payload
                        self.cached_wide_frame = payload
                elif kind == "event":
                    self._handle_event(payload)
                elif kind == "status":
                    self.lbl_status.config(text=payload)
            except queue.Empty:
                break

        # 2. Rendu FFT Panoramique & Waterfall
        if self.latest_wide_frame is not None and self.canvas_wide is not None:
            wide_f = self.latest_wide_frame
            self.latest_wide_frame = None

            n_bins = len(wide_f)
            decimated_wf = np.maximum.reduceat(
                wide_f, (np.arange(self.wf_cols + 1) * n_bins // self.wf_cols).astype(np.intp)[:-1]
            )
            self.wf_matrix_db = np.roll(self.wf_matrix_db, 1, axis=0)
            self.wf_matrix_db[0] = decimated_wf

            self._draw_wide_fft(wide_f)
            self._draw_waterfall()

        # 3. Rendu Zoom FFT Haute Résolution & Waterfall Fin
        if self.latest_zoom_frame is not None and self.canvas_zoom is not None:
            zoom_f = self.latest_zoom_frame
            self.latest_zoom_frame = None

            n_bins = len(zoom_f)
            decimated_zoom_wf = np.maximum.reduceat(
                zoom_f, (np.arange(self.zoom_wf_cols + 1) * n_bins // self.zoom_wf_cols).astype(np.intp)[:-1]
            )
            self.zoom_wf_matrix_db = np.roll(self.zoom_wf_matrix_db, 1, axis=0)
            self.zoom_wf_matrix_db[0] = decimated_zoom_wf

            self._draw_zoom_fft(zoom_f)
            if self.canvas_zoom_wf is not None:
                self._draw_zoom_waterfall()

        # 4. Rafraîchissement périodique du flux audio
        self.audio_player.tick()
        self._update_audio_status_ui()

        self.root.after(40, self._render_loop)

    def _on_close(self):
        """Ferme proprement le lecteur audio et le client WebSocket avant de quitter."""
        self.audio_player.close()
        try:
            self.ws_client.stop()
        except Exception:
            pass
        self.root.destroy()

    def _draw_wide_fft(self, frame):
        c = self.canvas_wide
        c.delete("spectrum", "markers", "labels", "grid")
        w = c.winfo_width()
        h = c.winfo_height()
        if w < 100 or h < 50:
            return

        n_bins = len(frame)
        pts_count = min(w, 800)
        idx = (np.arange(pts_count + 1) * n_bins // pts_count).astype(np.intp)
        sub = np.maximum.reduceat(frame, idx[:-1])

        # Normalisation Y dynamique pilotée par db_min et db_max
        y_min, y_max = self.db_min, self.db_max
        coords = []
        for i, val in enumerate(sub):
            x = (i / float(pts_count - 1)) * w
            norm_y = (val - y_min) / max(1.0, (y_max - y_min))
            norm_y = min(1.0, max(0.0, norm_y))
            y = h - (norm_y * (h - 22) + 11)
            coords.extend([x, y])

        # Lignes de grille horizontales en dB
        db_range = y_max - y_min
        step_db = 20 if db_range > 50 else (10 if db_range > 20 else 5)
        first_tick = int(np.ceil(y_min / step_db)) * step_db
        for tick in range(first_tick, int(y_max), step_db):
            norm_tick = (tick - y_min) / max(1.0, y_max - y_min)
            ty = h - (norm_tick * (h - 22) + 11)
            c.create_line(0, ty, w, ty, fill="#1c2838", dash=(2, 4), tags="grid")
            c.create_text(w - 4, ty - 2, text=f"{tick:+d} dB", fill="#4f637b", font=("Sans", 7), anchor=tk.SE, tags="grid")

        # Tracé de la courbe
        c.create_line(*coords, fill="#00e5ff", width=1.5, tags="spectrum")

        # Indicateur LO
        lo_ratio = (self.center_freq - self.freq_min_hz) / (self.freq_max_hz - self.freq_min_hz)
        lo_x = lo_ratio * w
        c.create_line(lo_x, 0, lo_x, h, fill="#00e5ff", width=1, dash=(4, 4), tags="markers")
        c.create_text(lo_x + 4, 12, text=f"LO {self.center_freq/1e6:.3f}M", fill="#00e5ff", font=("Sans", 8), anchor=tk.NW, tags="labels")

        # Repères des canaux
        target_ch = self.zoom_channel_id or self.vfo_channel_id
        for ch in self.channels:
            cf = ch["freq"] if ch["id"] != target_ch else self.current_vfo_hz
            ratio = (cf - self.freq_min_hz) / (self.freq_max_hz - self.freq_min_hz)
            cx = ratio * w
            col = ch["color"]
            dash = () if ch["id"] == target_ch else (3, 3)
            c.create_line(cx, 0, cx, h, fill=col, width=1.5, dash=dash, tags="markers")
            c.create_text(
                cx + 3,
                38 if ch["id"] == target_ch else 24,
                text=f"{ch['name']} ({cf/1e6:.3f})",
                fill=col,
                font=("Sans", 8, "bold"),
                anchor=tk.NW,
                tags="labels"
            )

        probe_name = self.panoramic_probe["name"] if self.panoramic_probe else "panoramic"
        c.create_text(
            8, 6,
            text=f"Spectre Panoramique [{probe_name}] ({self.freq_min_mhz:.3f} ➔ {self.freq_max_mhz:.3f} MHz) [{self.db_min:.0f} à {self.db_max:.0f} dB]",
            fill="#7e8b9b",
            font=("Sans", 8),
            anchor=tk.NW,
            tags="labels"
        )

    def _draw_waterfall(self):
        c = self.canvas_wf
        w = c.winfo_width()
        h = c.winfo_height()
        if w < 100 or h < 50:
            return

        denom = max(1e-3, self.db_max - self.db_min)
        norm = np.clip((self.wf_matrix_db - self.db_min) / denom, 0.0, 1.0)
        if self.gamma != 1.0:
            norm = np.power(norm, self.gamma)
        indices = (norm * 255.0).astype(np.uint8)
        rgb_arr = self.palette[indices]

        img = Image.fromarray(rgb_arr, mode="RGB")
        img_resized = img.resize((w, h), Image.NEAREST)
        self.wf_photo = ImageTk.PhotoImage(img_resized)

        c.delete("image", "vfo_line")
        c.create_image(0, 0, anchor=tk.NW, image=self.wf_photo, tags="image")

        # Repère VFO sur le waterfall
        vfo_ratio = (self.current_vfo_hz - self.freq_min_hz) / (self.freq_max_hz - self.freq_min_hz)
        vx = vfo_ratio * w
        c.create_line(vx, 0, vx, h, fill="#ffca28", width=2, tags="vfo_line")
        c.create_text(
            vx + 4, h - 14,
            text=f"VFO {self.current_vfo_hz/1e6:.4f}M",
            fill="#ffca28",
            font=("Sans", 8, "bold"),
            anchor=tk.SW,
            tags="vfo_line"
        )

    def _draw_zoom_fft(self, frame):
        c = self.canvas_zoom
        c.delete("spectrum", "filter", "markers", "labels", "grid")
        w = c.winfo_width()
        h = c.winfo_height()
        if w < 100 or h < 50:
            return

        # Normalisation Y dynamique pilotée par db_min et db_max
        y_min, y_max = self.db_min, self.db_max
        n_bins = len(frame)
        pts_count = min(w, n_bins)
        step = n_bins // pts_count
        coords = []
        for i in range(pts_count):
            x = (i / float(pts_count - 1)) * w
            val = frame[min(i * step, n_bins - 1)]
            norm_y = (val - y_min) / max(1.0, (y_max - y_min))
            norm_y = min(1.0, max(0.0, norm_y))
            y = h - (norm_y * (h - 22) + 11)
            coords.extend([x, y])

        # Lignes de grille horizontales en dB
        db_range = y_max - y_min
        step_db = 20 if db_range > 50 else (10 if db_range > 20 else 5)
        first_tick = int(np.ceil(y_min / step_db)) * step_db
        for tick in range(first_tick, int(y_max), step_db):
            norm_tick = (tick - y_min) / max(1.0, y_max - y_min)
            ty = h - (norm_tick * (h - 22) + 11)
            c.create_line(0, ty, w, ty, fill="#1c2838", dash=(2, 4), tags="grid")
            c.create_text(w - 4, ty - 2, text=f"{tick:+d} dB", fill="#4f637b", font=("Sans", 7), anchor=tk.SE, tags="grid")

        # Zone bande passante audio selon le mode (USB / LSB / FM / CW)
        px_per_hz = w / self.zoom_bw
        if self.vfo_mode == "LSB":
            fl, fr = -2700.0, -300.0
        elif self.vfo_mode == "FM":
            fl, fr = -5000.0, 5000.0
        elif self.vfo_mode == "CW":
            fl, fr = 500.0, 900.0
        else:  # USB par défaut
            fl, fr = 300.0, 2700.0

        pb_left = w / 2.0 + fl * px_per_hz
        pb_right = w / 2.0 + fr * px_per_hz
        c.create_rectangle(pb_left, 0, pb_right, h, fill="#133820", outline="#2e7d32", tags="filter")

        # Tracé de la courbe
        c.create_line(*coords, fill="#ffb300", width=1.5, tags="spectrum")

        # Marqueur de porteuse (DC = 0 Hz relatif = centre écran)
        cx = w / 2.0
        c.create_line(cx, 0, cx, h, fill="#ffca28", width=1.5, dash=(4, 4), tags="markers")
        target_ch = self.zoom_channel_id or self.vfo_channel_id
        c.create_text(
            cx + 4, 10,
            text=f"Porteuse [{target_ch}] : {self.current_vfo_hz/1e6:.4f} MHz ({self.vfo_mode})",
            fill="#ffca28",
            font=("Sans", 8, "bold"),
            anchor=tk.NW,
            tags="labels"
        )

        # Bornes en MHz
        z_min = (self.current_vfo_hz - self.zoom_bw / 2.0) / 1e6
        z_max = (self.current_vfo_hz + self.zoom_bw / 2.0) / 1e6
        probe_name = self.zoom_probe["name"] if self.zoom_probe else "zoom"
        c.create_text(
            8, h - 8,
            text=f"Zoom FFT [{probe_name}] ({z_min:.4f} MHz) [Clic / Glisser pour accord fin]",
            fill="#7e8b9b",
            font=("Sans", 8),
            anchor=tk.SW,
            tags="labels"
        )
        c.create_text(
            w - 8, h - 8,
            text=f"{z_max:.4f} MHz (Largeur {self.zoom_bw/1e3:.1f} kHz)",
            fill="#7e8b9b",
            font=("Sans", 8),
            anchor=tk.SE,
            tags="labels"
        )

    def _draw_zoom_waterfall(self):
        """Affiche la cascade défilante haute résolution centrée sur le canal VFO fin."""
        c = self.canvas_zoom_wf
        w = c.winfo_width()
        h = c.winfo_height()
        if w < 100 or h < 50:
            return

        denom = max(1e-3, self.db_max - self.db_min)
        norm = np.clip((self.zoom_wf_matrix_db - self.db_min) / denom, 0.0, 1.0)
        if self.gamma != 1.0:
            norm = np.power(norm, self.gamma)
        indices = (norm * 255.0).astype(np.uint8)
        rgb_arr = self.palette[indices]

        img = Image.fromarray(rgb_arr, mode="RGB")
        img_resized = img.resize((w, h), Image.NEAREST)
        self.zoom_wf_photo = ImageTk.PhotoImage(img_resized)

        c.delete("image", "vfo_line", "filter_line", "labels")
        c.create_image(0, 0, anchor=tk.NW, image=self.zoom_wf_photo, tags="image")

        # Marqueur de porteuse VFO au centre (DC = 0 Hz relatif = w / 2)
        cx = w / 2.0
        c.create_line(cx, 0, cx, h, fill="#ffca28", width=1.5, dash=(4, 4), tags="vfo_line")

        # Repères de la bande passante audio (USB / LSB) sur le waterfall zoom
        px_per_hz = w / self.zoom_bw
        if self.vfo_mode == "LSB":
            fl, fr = -2700.0, -300.0
        elif self.vfo_mode == "FM":
            fl, fr = -5000.0, 5000.0
        elif self.vfo_mode == "CW":
            fl, fr = 500.0, 900.0
        else:
            fl, fr = 300.0, 2700.0

        pb_left = w / 2.0 + fl * px_per_hz
        pb_right = w / 2.0 + fr * px_per_hz
        c.create_line(pb_left, 0, pb_left, h, fill="#2e7d32", width=1, dash=(2, 2), tags="filter_line")
        c.create_line(pb_right, 0, pb_right, h, fill="#2e7d32", width=1, dash=(2, 2), tags="filter_line")

        c.create_text(
            cx + 4, h - 14,
            text=f"Porteuse {self.current_vfo_hz/1e6:.4f}M",
            fill="#ffca28",
            font=("Sans", 8, "bold"),
            anchor=tk.SW,
            tags="labels"
        )
        c.create_text(
            8, 6,
            text=f"Cascade Zoom Haute Résolution ({self.zoom_bw/1e3:.1f} kHz) [Clic / Glisser pour accord fin]",
            fill="#7e8b9b",
            font=("Sans", 8),
            anchor=tk.NW,
            tags="labels"
        )


# =============================================================================
# Entrée principale CLI
# =============================================================================

def parse_args():
    parser = argparse.ArgumentParser(
        description="fsdr-hub : Moniteur Spectre & Contrôle VFO Unifié (Application Tkinter)"
    )
    parser.add_argument(
        "--config", "-c",
        default=None,
        help="Chemin vers le fichier de configuration TOML (ex: examples/cfg-synthetic.toml, examples/cfg-20m.toml)"
    )
    parser.add_argument(
        "--ws-url",
        default=None,
        help="URL WebSocket du serveur télémétrie fsdr-hub (défaut: extrait du TOML ou ws://127.0.0.1:5006/ws)"
    )
    return parser.parse_args()


def find_default_config():
    candidates = [
        "examples/cfg-synthetic.toml",
        "examples/cfg-20m.toml",
        "examples/cfg-2m.toml",
        "cfg.toml",
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return "examples/cfg-synthetic.toml"


def main():
    args = parse_args()
    config_path = args.config or find_default_config()

    try:
        config_data = load_hub_config(config_path)
    except Exception as e:
        print(f"Erreur lors du chargement de la configuration '{config_path}' : {e}")
        sys.exit(1)

    print(f"[hub_monitor_tk] Fichier de configuration : {config_path}")
    print(f"[hub_monitor_tk] LO : {config_data['center_freq']/1e6:.3f} MHz, Largeur : {config_data['sample_rate']/1e6:.1f} MHz")
    print(f"[hub_monitor_tk] Canal sondé par zoom FFT : '{config_data['zoom_channel_id']}' ({config_data['initial_vfo_freq']/1e6:.4f} MHz)")
    if config_data['panoramic_probe']:
        print(f"[hub_monitor_tk] Sonde panoramique : {config_data['panoramic_probe']['name']} ({config_data['panoramic_probe']['fft_size']} bins)")
    if config_data['zoom_probe']:
        print(f"[hub_monitor_tk] Sonde zoom : {config_data['zoom_probe']['name']} ({config_data['zoom_probe']['fft_size']} bins, {config_data['zoom_bw']/1e3:.1f} kHz)")

    root = tk.Tk()
    app = FsdrTkApp(root, config_data, custom_ws_url=args.ws_url)
    root.mainloop()


if __name__ == "__main__":
    main()
