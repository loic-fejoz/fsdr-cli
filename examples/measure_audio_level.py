#!/usr/bin/env python3
import subprocess
import struct
import math
import sys
import time

def measure_sink(sink_name="HF_SSB_RX.monitor", duration_sec=2.0):
    print(f"[measure] Recording {duration_sec}s from {sink_name} via parec...")
    cmd = [
        "parec",
        "-d", sink_name,
    ]
    
    try:
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    except FileNotFoundError:
        print(f"[measure] ERROR: parec executable not found on system!")
        return False
    raw_data = bytearray()
    start_time = time.time()
    
    # Read samples incrementally over duration
    while time.time() - start_time < duration_sec:
        chunk = proc.stdout.read(4096)
        if not chunk:
            break
        raw_data.extend(chunk)
    proc.terminate()
    try:
        proc.wait(timeout=1.0)
    except subprocess.TimeoutExpired:
        proc.kill()
        
    # Discard lead-in silence (first 4KB / ~1000 samples) if stream has enough data
    if len(raw_data) > 8192:
        raw_data = raw_data[4096:]
        
    num_samples = len(raw_data) // 2
    if num_samples == 0:
        print(f"[measure] ERROR: No audio samples captured from {sink_name}!")
        return False
        
    samples = [struct.unpack("<h", raw_data[i*2:(i+1)*2])[0] / 32768.0 for i in range(num_samples)]
    
    # Calculate statistics
    peak = max(abs(s) for s in samples)
    rms = math.sqrt(sum(s*s for s in samples) / len(samples))
    db_peak = 20 * math.log10(peak) if peak > 1e-6 else -120.0
    db_rms = 20 * math.log10(rms) if rms > 1e-6 else -120.0
    
    print(f"[measure] Captured {num_samples} samples ({num_samples/48000.0:.2f}s)")
    print(f"[measure] Peak Amplitude: {peak:.4f} ({db_peak:.1f} dBFS)")
    print(f"[measure] RMS Level:      {rms:.4f} ({db_rms:.1f} dBFS)")
    
    if rms < 0.001:
        print(f"[measure] WARNING: Signal level is too low (silence detected)")
        return False
        
    print(f"[measure] SUCCESS: Active audio stream detected on {sink_name}!")
    return True

if __name__ == "__main__":
    sink = sys.argv[1] if len(sys.argv) > 1 else "HF_SSB_RX.monitor"
    dur = float(sys.argv[2]) if len(sys.argv) > 2 else 2.0
    ok = measure_sink(sink, dur)
    sys.exit(0 if ok else 1)
