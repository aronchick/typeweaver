#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

echo "==> Building release binary"
cd "$REPO_DIR"
cargo build --release -p typeweaver-cli

echo "==> Installing binary"
sudo cp target/release/typeweaver-cli /usr/local/bin/typeweaver

echo "==> Creating service user (if needed)"
sudo useradd --system --shell /usr/sbin/nologin typeweaver 2>/dev/null || true

echo "==> Creating data directory"
sudo mkdir -p /var/lib/typeweaver
sudo chown typeweaver:typeweaver /var/lib/typeweaver

echo "==> Installing systemd unit"
sudo cp deploy/typeweaver.service /etc/systemd/system/typeweaver.service
sudo systemctl daemon-reload
sudo systemctl enable typeweaver
sudo systemctl restart typeweaver

echo "==> Done. Check status:"
echo "    sudo systemctl status typeweaver"
echo "    curl http://localhost:3000/healthz"
