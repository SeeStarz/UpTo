#!/bin/bash
set -e

ssh vps bash << EOF
  set -e
  git clone https://github.com/SeeStarz/UpTo.git || true
EOF

scp .env-secret vps:UpTo/.env-secret
scp Caddyfile vps:/etc/caddy/fahri/upto.caddy

ssh vps bash << EOF
  set -e
  cd UpTo
  git fetch origin
  git reset --hard origin/deploy
  cargo build --release

  sed -i 's|value="http://localhost:8000"|value="https://upto.seestarz.my.id"|' server/index.html

  sudo mkdir -p /var/www/upto
  sudo chown fahri:caddy /var/www/upto
  cp server/index.html /var/www/upto
  sudo systemctl restart caddy

  set -a
  source .env-secret
  set +a
  target/release/server
EOF
