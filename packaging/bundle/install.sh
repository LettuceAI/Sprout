#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$DIR/sprout"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"

if [ ! -f "$BIN" ]; then
  echo "error: sprout binary not found next to this script" >&2
  exit 1
fi

mkdir -p "$BIN_DIR"
install -m 755 "$BIN" "$BIN_DIR/sprout"
echo "Installed sprout to $BIN_DIR/sprout"

if [ "$(uname -s)" = "Linux" ] && command -v systemctl >/dev/null 2>&1 && [ -t 0 ]; then
  printf 'Run sprout in the background as a systemd user service? [y/N] '
  read -r reply
  case "$reply" in
    y | Y)
      UNIT_DIR="$HOME/.config/systemd/user"
      mkdir -p "$UNIT_DIR"
      cat >"$UNIT_DIR/sprout.service" <<EOF
[Unit]
Description=Sprout hardware-info probe for LettuceAI
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=$BIN_DIR/sprout
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF
      systemctl --user daemon-reload
      systemctl --user enable --now sprout.service
      echo "Service started. Logs: journalctl --user -u sprout -f"
      ;;
  esac
fi

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "Add $BIN_DIR to your PATH to run 'sprout' directly." ;;
esac

echo "Start it with: sprout"
