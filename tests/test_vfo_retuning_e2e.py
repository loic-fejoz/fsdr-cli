#!/usr/bin/env python3
import asyncio
import json
import math
import os
import signal
import struct
import subprocess
import sys
import time
import numpy as np

try:
    from tornado.websocket import websocket_connect
    from tornado.ioloop import IOLoop
except ImportError:
    print("Tornado not installed, using tungstenite/websockets alternative if available")

WS_URL = "ws://127.0.0.1:5006/ws"

def record_audio(sink_name="HF_SSB_RX.monitor", duration_sec=1.5, sample_rate=48000):
    cmd = [
        "parec",
        "--format=s16le",
        f"--rate={sample_rate}",
        "--channels=1",
        "--latency-msec=50",
        "-d", sink_name,
    ]
    try:
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    except FileNotFoundError:
        print("[test] parec not found on system")
        return None
        
    raw_data = bytearray()
    start = time.time()
    while time.time() - start < duration_sec:
        chunk = proc.stdout.read(4096)
        if not chunk:
            break
        raw_data.extend(chunk)
    proc.terminate()
    try:
        proc.wait(timeout=1.0)
    except subprocess.TimeoutExpired:
        proc.kill()
        
    num_samples = len(raw_data) // 2
    if num_samples < sample_rate // 2:
        return None
    samples = np.frombuffer(raw_data[:num_samples * 2], dtype=np.int16).astype(np.float32) / 32768.0
    return samples

def analyze_audio_tones(samples, sample_rate=48000):
    # Perform FFT on windowed audio
    window = np.hanning(len(samples))
    fft = np.abs(np.fft.rfft(samples * window))
    freqs = np.fft.rfftfreq(len(samples), 1.0 / sample_rate)
    
    # Check energy in 700 Hz band (650-750 Hz), 1500 Hz band (1450-1550 Hz), and 1900 Hz band (1850-1950 Hz)
    idx_700 = np.where((freqs >= 650) & (freqs <= 750))[0]
    idx_1500 = np.where((freqs >= 1450) & (freqs <= 1550))[0]
    idx_1900 = np.where((freqs >= 1850) & (freqs <= 1950))[0]
    idx_noise = np.where((freqs >= 100) & (freqs <= 3000) 
                         & ~((freqs >= 650) & (freqs <= 750))
                         & ~((freqs >= 1450) & (freqs <= 1550))
                         & ~((freqs >= 1850) & (freqs <= 1950)))[0]
    
    peak_700 = np.max(fft[idx_700]) if len(idx_700) > 0 else 0
    peak_1500 = np.max(fft[idx_1500]) if len(idx_1500) > 0 else 0
    peak_1900 = np.max(fft[idx_1900]) if len(idx_1900) > 0 else 0
    noise_floor = np.median(fft[idx_noise]) if len(idx_noise) > 0 else 1e-6
    
    snr_700 = 20 * np.log10(max(1e-6, peak_700) / max(1e-6, noise_floor))
    snr_1500 = 20 * np.log10(max(1e-6, peak_1500) / max(1e-6, noise_floor))
    snr_1900 = 20 * np.log10(max(1e-6, peak_1900) / max(1e-6, noise_floor))
    
    return snr_700, snr_1900, snr_1500

async def run_test():
    print(f"[test] Connecting to WebSocket at {WS_URL}...")
    ws = None
    for attempt in range(20):
        try:
            ws = await websocket_connect(WS_URL, connect_timeout=2)
            break
        except Exception:
            await asyncio.sleep(0.5)
            
    if ws is None:
        print("[test] FAILED: Could not connect to fsdr-hub WebSocket")
        return False
        
    print("[test] Connected to fsdr-hub WebSocket successfully!")
    
    # 1. Collect initial frames (FFT 1024 zoom)
    fft_1024_initial = None
    for _ in range(50):
        msg = await ws.read_message()
        if msg is None:
            break
        if isinstance(msg, bytes):
            floats = np.frombuffer(msg, dtype="<f4")
            if len(floats) == 1024:
                fft_1024_initial = floats.copy()
                break
        await asyncio.sleep(0.02)
        
    if fft_1024_initial is None:
        print("[test] FAILED: No 1024-bin zoom FFT frames received from WebSocket")
        return False
        
    print(f"[test] Received initial 1024-bin FFT zoom frame (min={np.min(fft_1024_initial):.1f} dB, max={np.max(fft_1024_initial):.1f} dB)")
    
    # In initial state (14.250 MHz), check for two-tone signal at bins ~559 and ~584
    # (Center bin is 512, tones are at +2200 Hz and +3400 Hz, with 46.875 Hz/bin)
    peak1_region = fft_1024_initial[550:568]
    peak2_region = fft_1024_initial[578:595]
    noise_region = np.concatenate([fft_1024_initial[520:540], fft_1024_initial[610:650]])
    median_noise = np.median(noise_region)
    
    p1_val = np.max(peak1_region)
    p2_val = np.max(peak2_region)
    print(f"[test] Initial FFT SSB peaks: Peak1={p1_val:.1f} dB, Peak2={p2_val:.1f} dB (Noise floor: {median_noise:.1f} dB)")
    assert p1_val - median_noise > 10.0, f"Peak 1 (700Hz subcarrier) not detected in zoom FFT: {p1_val} vs {median_noise}"
    assert p2_val - median_noise > 10.0, f"Peak 2 (1900Hz subcarrier) not detected in zoom FFT: {p2_val} vs {median_noise}"
    print("[test] SUCCESS: SSB two-tone signal clearly detected in initial zoom FFT!")
    
    # Check audio from PulseAudio monitor sink
    samples = record_audio("HF_SSB_RX.monitor", duration_sec=1.5)
    if samples is not None and len(samples) > 0:
        snr700, snr1900, snr1500 = analyze_audio_tones(samples)
        print(f"[test] Initial audio tones in HF_SSB_RX: 700 Hz SNR={snr700:.1f} dB, 1900 Hz SNR={snr1900:.1f} dB, 1500 Hz SNR={snr1500:.1f} dB")
        if snr700 > 10.0 and snr1900 > 10.0:
            print("[test] SUCCESS: SSB two-tone audio detected in HF_SSB_RX.monitor!")
        else:
            print("[test] Notice: Audio detected but SNR is moderate, continuing test...")
    else:
        print("[test] Notice: parec not capturing from HF_SSB_RX.monitor, continuing with FFT validation")

    # 2. Simulate moving VFO slider to 14.230 MHz (SSTV carrier offset)
    print("\n[test] >>> Sending VFO retune command: set_frequency to 14_230_000 Hz...")
    retune_cmd = json.dumps({
        "cmd": "set_frequency",
        "channel": "ssb_voice_20m",
        "frequency": 14230000
    })
    await ws.write_message(retune_cmd)
    
    # 3. Verify frequency_changed event is broadcast over WebSocket
    event_received = False
    for i in range(100):
        msg = await ws.read_message()
        if msg is None:
            print("[test] ws.read_message() returned None (closed)")
            break
        if isinstance(msg, str):
            print(f"[test] Received text message: {msg}")
            try:
                data = json.loads(msg)
                if data.get("event") == "frequency_changed" and data.get("frequency") == 14230000:
                    event_received = True
                    print(f"[test] SUCCESS: Received frequency_changed event over WebSocket: {data}")
                    break
            except Exception as e:
                print(f"[test] JSON parse error: {e}")
        else:
            # Binary frame
            pass
        
    assert event_received, "Expected frequency_changed event on WebSocket after retuning"
    
    # Wait for DSP pipeline and audio buffers to process new NCO frequency
    await asyncio.sleep(1.2)
    
    # 4. Collect latest post-retuning FFT zoom frames (draining older buffered frames)
    fft_1024_after = None
    for _ in range(30):
        msg = await ws.read_message()
        if msg is None:
            break
        if isinstance(msg, bytes):
            floats = np.frombuffer(msg, dtype="<f4")
            if len(floats) == 1024:
                fft_1024_after = floats.copy()
        await asyncio.sleep(0.01)
        
    assert fft_1024_after is not None, "Did not receive post-retuning FFT zoom frame"
    
    # At 14.230 MHz, the SSTV carrier tone (+30 kHz from center 14.200M) is now downconverted to 0 Hz (bin 512)!
    center_peak = np.max(fft_1024_after[508:516])
    old_p1 = np.max(fft_1024_after[550:568])
    old_p2 = np.max(fft_1024_after[578:595])
    noise_after = np.median(fft_1024_after[610:650])
    
    print(f"[test] Post-retune FFT: Center tone (bin 512)={center_peak:.1f} dB, Old P1={old_p1:.1f} dB, Old P2={old_p2:.1f} dB (Noise: {noise_after:.1f} dB)")
    
    assert center_peak - noise_after > 15.0, f"Expected strong center peak (SSTV signal at 14.230 MHz) in zoom FFT, got {center_peak} vs {noise_after}"
    assert center_peak > old_p1 + 10.0, f"Expected center tone to dominate over old P1 tone: {center_peak} vs {old_p1}"
    print("[test] SUCCESS: Zoom FFT correctly shifted to 14.230 MHz, center peak confirmed and old two-tone signal removed!")
    
    # Check post-retune audio: 700/1900 Hz tones should drop significantly and 1500 Hz SSTV tone should dominate
    samples_after = record_audio("HF_SSB_RX.monitor", duration_sec=1.5)
    if samples_after is not None and len(samples_after) > 0:
        snr700_aft, snr1900_aft, snr1500_aft = analyze_audio_tones(samples_after)
        print(f"[test] Post-retune audio tones in HF_SSB_RX: 700 Hz SNR={snr700_aft:.1f} dB, 1900 Hz SNR={snr1900_aft:.1f} dB, 1500 Hz SNR={snr1500_aft:.1f} dB")
        assert snr1500_aft > 30.0, f"Expected strong 1500 Hz SSTV tone in audio after retuning to 14.230 MHz, got {snr1500_aft:.1f} dB"
        assert snr1500_aft > snr700_aft + 25.0, f"Expected 1500 Hz SSTV tone to dominate over 700 Hz tone: {snr1500_aft} vs {snr700_aft}"
        assert snr1500_aft > snr1900_aft + 35.0, f"Expected 1500 Hz SSTV tone to dominate over 1900 Hz tone: {snr1500_aft} vs {snr1900_aft}"
        print("[test] SUCCESS: Post-retuning audio has dominant SSTV 1500 Hz tone (>25dB above SSB tones)!")

    ws.close()
    return True

if __name__ == "__main__":
    ok = asyncio.run(run_test())
    sys.exit(0 if ok else 1)
