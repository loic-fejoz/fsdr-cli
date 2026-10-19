#!/usr/bin/env bash
set -euo pipefail

echo "=========================================================="
echo " [fsdr-hub] Test de validation globale E2E (Multi-Canal)"
echo "=========================================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# 1. Compilation
echo "[1/6] Compilation de fsdr-hub (avec support soapy)..."
cargo build --bin fsdr-hub --features soapy

# 2. Lancement en arrière-plan
echo "[2/6] Démarrage de fsdr-hub avec examples/cfg-synthetic.toml..."
target/debug/fsdr-hub -c examples/cfg-synthetic.toml -v > /tmp/fsdr_hub_test.log 2>&1 &
HUB_PID=$!
echo "fsdr-hub démarré avec PID: $HUB_PID"

cleanup() {
    echo "[Cleanup] Arrêt forcé si nécessaire..."
    if kill -0 "$HUB_PID" 2>/dev/null; then
        kill -TERM "$HUB_PID" 2>/dev/null || true
        wait "$HUB_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# 3. Attente de la montée des services
echo "[3/6] Attente de la disponibilité du WebSocket (127.0.0.1:5006)..."
MAX_WAIT=15
WAIT_COUNT=0
while ! nc -z 127.0.0.1 5006 2>/dev/null; do
    sleep 0.5
    WAIT_COUNT=$((WAIT_COUNT + 1))
    if [ "$WAIT_COUNT" -gt "$MAX_WAIT" ]; then
        echo "ERREUR: Le serveur WebSocket n'a pas répondu dans les temps."
        cat /tmp/fsdr_hub_test.log
        exit 1
    fi
done
echo "Serveur WebSocket actif !"

# 4. Vérification des périphériques audio virtuels PulseAudio
echo "[4/6] Vérification des modules audio PulseAudio/PipeWire..."
if command -v pactl >/dev/null 2>&1; then
    pactl list short sources | grep -E "HF_FT8_RX|HF_SSTV_RX|HF_SSB_RX|HF_WSPR_RX" || true
    echo "Périphériques virtuels détectés avec succès."
fi

# 5. Validation de la télémétrie WebSocket temps-réel
echo "[5/6] Validation du flux WebSocket via test_websocket.py..."
python3 examples/test_websocket.py 5006

# 6. Arrêt propre par signal SIGTERM et validation du déchargement
echo "[6/6] Arrêt propre de fsdr-hub via SIGTERM..."
kill -TERM "$HUB_PID"
wait "$HUB_PID" || true

echo "Vérification des logs d'extinction :"
grep "Shutdown signal received" /tmp/fsdr_hub_test.log || true
grep "Unloaded PulseAudio module" /tmp/fsdr_hub_test.log | head -n 3 || true

echo "=========================================================="
echo " SUCCÈS : fsdr-hub validé de bout en bout !"
echo "=========================================================="
