#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
UNIT_DIR="$HOME/.config/systemd/user"

if command -v systemctl >/dev/null 2>&1; then
  systemctl --user disable --now sprout.service 2>/dev/null || true
  rm -f "$UNIT_DIR/sprout.service"
  systemctl --user daemon-reload
fi

rm -f "$BIN_DIR/sprout"

echo "Sprout binary and service removed."
echo "Config was left in place. Remove it manually if you want:"
echo "  rm -rf ~/.config/sprout"
