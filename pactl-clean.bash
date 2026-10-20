#!/bin/bash
if command -v pactl >/dev/null 2>&1; then
    pactl list modules short 2>/dev/null | grep "module-null-sink" | awk '{print $1}' | while read -r id; do
        if [ -n "$id" ]; then
            pactl unload-module "$id" 2>/dev/null && echo "Unloaded $id" || true
        fi
    done
fi
