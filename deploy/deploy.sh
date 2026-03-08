#!/usr/bin/env bash
# Deploy TypeWeaver to Hetzner (136.243.132.145)
# Run from repo root on the DO droplet.
set -euo pipefail

HETZNER=136.243.132.145
REMOTE_USER=root
REMOTE_DIR=/opt/typeweaver
SERVICE=typeweaver

echo "==> Building release binary (no OCR feature for now)"
cargo build --release -p typeweaver-cli

echo "==> Syncing binary to Hetzner"
ssh "${REMOTE_USER}@${HETZNER}" "mkdir -p ${REMOTE_DIR}"
rsync -avz --progress \
  target/release/typeweaver-cli \
  "${REMOTE_USER}@${HETZNER}:${REMOTE_DIR}/typeweaver"

echo "==> Installing systemd service"
scp deploy/typeweaver.service \
  "${REMOTE_USER}@${HETZNER}:/etc/systemd/system/${SERVICE}.service"

echo "==> Installing Caddy config"
scp deploy/caddy-typeweaver.caddy \
  "${REMOTE_USER}@${HETZNER}:/etc/caddy/sites/${SERVICE}.caddy"

echo "==> Reloading services on Hetzner"
ssh "${REMOTE_USER}@${HETZNER}" bash -euo pipefail << 'REMOTE'
  useradd --system --shell /usr/sbin/nologin typeweaver 2>/dev/null || true
  mkdir -p /opt/typeweaver/.typeweaver
  chown -R typeweaver:typeweaver /opt/typeweaver
  chmod +x /opt/typeweaver/typeweaver
  systemctl daemon-reload
  systemctl enable typeweaver
  systemctl restart typeweaver
  caddy validate --config /etc/caddy/Caddyfile 2>/dev/null && caddy reload --config /etc/caddy/Caddyfile || true
REMOTE

echo "==> Verifying deployment"
sleep 3
curl -sf "http://${HETZNER}:3500/okz" && echo " /okz OK"
curl -sf "http://${HETZNER}:3500/healthz" && echo " /healthz OK"
curl -sf "http://${HETZNER}:3500/varz" | head -5 && echo " /varz OK"

echo ""
echo "Deployed to typeweaver.org (${HETZNER}:3500)"
echo "Point typeweaver.org DNS A record → ${HETZNER} in Cloudflare if not already done."
