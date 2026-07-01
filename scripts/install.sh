#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
UNIT_DIR="$HOME/.config/systemd/user"
CONFIG="${SPROUT_CONFIG:-$HOME/.config/sprout/config.toml}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found. Install the Rust toolchain from https://rustup.rs" >&2
  exit 1
fi

if ! command -v systemctl >/dev/null 2>&1; then
  echo "error: systemctl not found. This installer targets systemd." >&2
  echo "Run sprout manually instead: cargo build --release && ./target/release/sprout" >&2
  exit 1
fi

echo "==> Building release binary"
cargo build --release --manifest-path "$ROOT/Cargo.toml"

echo "==> Installing binary to $BIN_DIR/sprout"
mkdir -p "$BIN_DIR"
install -m 755 "$ROOT/target/release/sprout" "$BIN_DIR/sprout"

echo "==> Installing systemd user service"
mkdir -p "$UNIT_DIR"
sed "s|@BIN@|$BIN_DIR/sprout|g" "$ROOT/packaging/sprout.service" > "$UNIT_DIR/sprout.service"
systemctl --user daemon-reload
systemctl --user enable --now sprout.service

for _ in $(seq 1 20); do
  [ -f "$CONFIG" ] && break
  sleep 0.3
done

echo
echo "Sprout is installed and running as a user service."
echo "  Status:  systemctl --user status sprout"
echo "  Logs:    journalctl --user -u sprout -f"
echo "  Restart: systemctl --user restart sprout"
echo "  Config:  $CONFIG"
if [ -f "$CONFIG" ]; then
  KEY="$(sed -n 's/^api_key *= *"\(.*\)"/\1/p' "$CONFIG")"
  echo "  API key: $KEY"
fi
echo
echo "To accept requests from other machines, set host = \"0.0.0.0\" in the config, then:"
echo "  systemctl --user restart sprout"
echo "To keep the service running while logged out:"
echo "  sudo loginctl enable-linger $USER"
if ! printf '%s' ":$PATH:" | grep -q ":$BIN_DIR:"; then
  echo
  echo "Note: $BIN_DIR is not on your PATH. Add it to run 'sprout' directly."
fi
