#!/usr/bin/env bash
# ============================================================
# SafeMask Auth — HMAC Key Initializer
# ============================================================
# Usage:
#   sudo bash init-key.sh
#   sudo bash init-key.sh --dir /etc/safemask-auth --force
#   bash init-key.sh --dry-run
# ============================================================
set -euo pipefail

CONFIG_DIR="/opt/safemask-auth/config"
FORCE=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dir)    CONFIG_DIR="$2"; shift 2 ;;
    --force)  FORCE=true;       shift   ;;
    --dry-run) DRY_RUN=true;    shift   ;;
    -h|--help)
      echo "Usage: init-key.sh [--dir PATH] [--force] [--dry-run]"
      exit 0 ;;
    *) echo "Unknown: $1"; exit 1 ;;
  esac
done

KEY_FILE="$CONFIG_DIR/hmac.key"
ENV_FILE="$CONFIG_DIR/safemask.env"

if ! command -v openssl &>/dev/null; then
  echo "[FATAL] openssl not found — cannot generate key"
  exit 1
fi

# ----- Already exists ? -----
if [[ -f "$KEY_FILE" && "$FORCE" != true ]]; then
  echo "[INIT] HMAC key exists at $KEY_FILE (use --force to regenerate)"
  cat "$KEY_FILE"
  exit 0
fi

echo "[INIT] Generating 256-bit HMAC key ..."

if [[ "$DRY_RUN" == true ]]; then
  echo "[DRY]  mkdir -p $CONFIG_DIR"
  echo "[DRY]  openssl rand -base64 32 → $KEY_FILE"
  echo "[DRY]  chmod 600 $KEY_FILE"
  echo "[DRY]  write $ENV_FILE"
  exit 0
fi

mkdir -p "$CONFIG_DIR"

HMAC_KEY=$(openssl rand -base64 32)

echo -n "$HMAC_KEY" > "$KEY_FILE"
chmod 600 "$KEY_FILE"

cat > "$ENV_FILE" <<EOF
# SafeMask Auth — Environment (auto-generated, keep secure)
HMAC_KEY=$HMAC_KEY
EOF
chmod 600 "$ENV_FILE"

echo "[INIT] Key saved to $KEY_FILE (chmod 600)"
echo "[INIT] Env  saved to $ENV_FILE"
echo "$HMAC_KEY"
