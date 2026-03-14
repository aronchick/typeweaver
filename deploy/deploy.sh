#!/usr/bin/env bash
# Deploy TypeWeaver to the configured remote host.
# Run from repo root with DEPLOY_HOST and DEPLOY_USER set.
# The GitHub Actions workflow is the canonical deploy path; this script is the manual fallback.
set -euo pipefail

DEPLOY_HOST=${DEPLOY_HOST:-}
DEPLOY_USER=${DEPLOY_USER:-root}
REMOTE_DIR=${REMOTE_DIR:-/opt/typeweaver}
SERVICE=${SERVICE:-typeweaver}
SSH_OPTS=${SSH_OPTS:-}

if [ -z "$DEPLOY_HOST" ]; then
  cat >&2 <<'EOF'
DEPLOY_HOST is required.

Why: the old hardcoded public Hetzner IP drifted from the real deploy target.
GitHub Actions deploys successfully using repo secrets (often via Tailscale/private routing),
so this manual script must be told the current host explicitly.

Example:
  DEPLOY_HOST=100.x.y.z DEPLOY_USER=root ./deploy/deploy.sh
EOF
  exit 1
fi

echo "==> Building release binary"
cargo build --release -p typeweaver-cli

echo "==> Checking SSH reachability for ${DEPLOY_USER}@${DEPLOY_HOST}:22"
python3 -c 'import socket, sys; host = sys.argv[1]; s = socket.create_connection((host, 22), timeout=10); s.close(); print(f"SSH reachable on {host}:22")' "$DEPLOY_HOST"

echo "==> Syncing binary"
ssh $SSH_OPTS "${DEPLOY_USER}@${DEPLOY_HOST}" "mkdir -p ${REMOTE_DIR}"
rsync -avz --progress \
  -e "ssh $SSH_OPTS" \
  target/release/typeweaver-cli \
  "${DEPLOY_USER}@${DEPLOY_HOST}:${REMOTE_DIR}/typeweaver"

echo "==> Installing systemd service"
rsync -avz \
  -e "ssh $SSH_OPTS" \
  deploy/typeweaver.service \
  "${DEPLOY_USER}@${DEPLOY_HOST}:~/typeweaver.service"
ssh $SSH_OPTS "${DEPLOY_USER}@${DEPLOY_HOST}" "install -m 0644 ~/typeweaver.service /etc/systemd/system/${SERVICE}.service && rm -f ~/typeweaver.service"

echo "==> Installing Caddy config"
rsync -avz \
  -e "ssh $SSH_OPTS" \
  deploy/caddy-typeweaver.caddy \
  "${DEPLOY_USER}@${DEPLOY_HOST}:~/typeweaver.caddy"
ssh $SSH_OPTS "${DEPLOY_USER}@${DEPLOY_HOST}" "mkdir -p /etc/caddy/sites && install -m 0644 ~/typeweaver.caddy /etc/caddy/sites/${SERVICE}.caddy && rm -f ~/typeweaver.caddy"

echo "==> Reloading services"
ssh $SSH_OPTS "${DEPLOY_USER}@${DEPLOY_HOST}" bash -euo pipefail <<REMOTE
  useradd --system --shell /usr/sbin/nologin typeweaver 2>/dev/null || true
  mkdir -p ${REMOTE_DIR}/.typeweaver
  chown -R typeweaver:typeweaver ${REMOTE_DIR}
  chmod +x ${REMOTE_DIR}/typeweaver
  systemctl daemon-reload
  systemctl enable ${SERVICE}
  systemctl restart ${SERVICE}
  caddy validate --config /etc/caddy/Caddyfile
  systemctl reload caddy
  sleep 3
  curl -sf --max-time 10 http://127.0.0.1:3500/okz && echo ' /okz OK'
  curl -sf --max-time 10 http://127.0.0.1:3500/healthz && echo ' /healthz OK'
  curl -sf --max-time 10 http://127.0.0.1:3500/varz | head -5 && echo ' /varz OK'
REMOTE

echo ""
echo "Deployed to typeweaver.org via ${DEPLOY_USER}@${DEPLOY_HOST}"
