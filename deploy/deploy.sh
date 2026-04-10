#!/usr/bin/env bash
# Deploy TypeWeaver to Hetzner.
# Run from repo root with DEPLOY_HOST set (for example: DEPLOY_HOST=hetzner-main ./deploy/deploy.sh)
set -euo pipefail

DEPLOY_HOST=${DEPLOY_HOST:-}
DEPLOY_USER=${DEPLOY_USER:-daaronch}
REMOTE_DIR=${REMOTE_DIR:-/opt/typeweaver}
SERVICE=${SERVICE:-typeweaver}
SSH_OPTS=${SSH_OPTS:-}
CADDY_SITE_DIR=${CADDY_SITE_DIR:-/etc/caddy/sites}
CADDY_SITE=${CADDY_SITE:-typeweaver.caddy}

if [ -z "$DEPLOY_HOST" ]; then
  cat >&2 <<'MSG'
DEPLOY_HOST is required.

Example:
  DEPLOY_HOST=hetzner-main DEPLOY_USER=daaronch ./deploy/deploy.sh
MSG
  exit 1
fi

echo "==> Building release binary"
cargo build --release -p typeweaver-cli

echo "==> Checking SSH reachability for ${DEPLOY_USER}@${DEPLOY_HOST}:22"
python3 -c 'import socket, sys; host = sys.argv[1]; s = socket.create_connection((host, 22), timeout=10); s.close(); print(f"SSH reachable on {host}:22")' "$DEPLOY_HOST"

echo "==> Syncing binary and static site"
ssh $SSH_OPTS "${DEPLOY_USER}@${DEPLOY_HOST}" "mkdir -p /tmp/typeweaver-frontend"
rsync -avz --progress \
  -e "ssh $SSH_OPTS" \
  target/release/typeweaver-cli \
  "${DEPLOY_USER}@${DEPLOY_HOST}:/tmp/typeweaver-cli"
rsync -avz --delete \
  -e "ssh $SSH_OPTS" \
  frontend/ \
  "${DEPLOY_USER}@${DEPLOY_HOST}:/tmp/typeweaver-frontend/"

echo "==> Installing binary, static site, and Caddy config"
rsync -avz \
  -e "ssh $SSH_OPTS" \
  deploy/caddy-typeweaver.caddy \
  "${DEPLOY_USER}@${DEPLOY_HOST}:/tmp/typeweaver.caddy"
ssh $SSH_OPTS "${DEPLOY_USER}@${DEPLOY_HOST}" bash -s <<'EOF'
set -euo pipefail
sudo mkdir -p /opt/typeweaver/frontend /etc/caddy/sites
sudo rsync -av --delete /tmp/typeweaver-frontend/ /opt/typeweaver/frontend/
sudo install -m 0755 /tmp/typeweaver-cli /opt/typeweaver/typeweaver
sudo install -m 0644 /tmp/typeweaver.caddy /etc/caddy/sites/typeweaver.caddy
sudo systemctl daemon-reload
sudo systemctl enable typeweaver
sudo systemctl restart typeweaver
sudo caddy validate --config /etc/caddy/Caddyfile
sudo systemctl reload caddy
curl -sf --max-time 10 http://127.0.0.1:3500/okz >/dev/null
curl -sf --max-time 10 http://127.0.0.1:3500/api/public-fonts?limit=3 >/dev/null
EOF

echo ""
echo "Deployed to typeweaver.org via ${DEPLOY_USER}@${DEPLOY_HOST}"
