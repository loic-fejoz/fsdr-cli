#!/bin/bash
set -euo pipefail

SIGMF_META="${1:-}"
if [ -z "$SIGMF_META" ]; then
    echo "Usage: $0 <path_to_sigmf_meta>" >&2
    exit 1
fi

SIGMF_DATA="${SIGMF_META%.*}.sigmf-data"
if [ ! -f "$SIGMF_DATA" ]; then
    echo "Error: SigMF data file not found: $SIGMF_DATA" >&2
    exit 1
fi

TIMESTAMP=$(date +"%Y%m%d_%H%M%S_%N" | cut -c1-19)
OPUS_FILE="/tmp/radio_${TIMESTAMP}.opus"

echo "[nfm2dtn] Converting $SIGMF_DATA (PCM f32le 48kHz) to $OPUS_FILE..."
ffmpeg -v error -y -f f32le -ar 48000 -ac 1 -i "$SIGMF_DATA" -c:a libopus -b:a 24k "$OPUS_FILE"

echo "[nfm2dtn] Sending $OPUS_FILE to dtn://f4jxq-9/files..."
if dtnsend --receiver "dtn://f4jxq-9/files" "$OPUS_FILE"; then
    echo "[nfm2dtn] Sent successfully to DTN: $OPUS_FILE"
else
    echo "[nfm2dtn] Warning: dtnsend returned an error (is DTN daemon running?)" >&2
fi
rm -f ${OPUS_FILE}
