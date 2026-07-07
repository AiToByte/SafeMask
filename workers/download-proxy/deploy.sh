#!/usr/bin/env bash
# ============================================================
# SafeMask Auth — Auto Deployment Script
# ============================================================
# Idempotent — safe to re-run.  Links with init-key.sh.
#
# Usage:
#   sudo bash deploy.sh \
#     --domain www.xiao.org.cn \
#     --download-url https://obs.be.com:9004/gxzh/2026/07/06/privacy-filter.zip
#
#   sudo bash deploy.sh --restart-only
#   bash deploy.sh --dry-run --domain www.xiao.org.cn --download-url ...
# ============================================================
set -euo pipefail

# ─── Paths ──────────────────────────────────────────────────
KEY_DIR="/opt/safemask-auth/config"
CONFIG_DIR="/opt/safemask-auth"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INIT_KEY_SCRIPT="$SCRIPT_DIR/init-key.sh"
KEY_FILE="$KEY_DIR/hmac.key"

PM2_NAME="safemask-dl-auth"
PROXY_PORT=35347

# ─── Params ──────────────────────────────────────────────────
DOMAIN=""
DOWNLOAD_URL=""
DRY_RUN=false
RESTART_ONLY=false
SKIP_VERIFY=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --domain)        DOMAIN="$2";        shift 2 ;;
    --download-url)  DOWNLOAD_URL="$2";  shift 2 ;;
    --key-dir)       KEY_DIR="$2";       shift 2 ;;
    --config-dir)    CONFIG_DIR="$2";    shift 2 ;;
    --port)          PROXY_PORT="$2";    shift 2 ;;
    --dry-run)       DRY_RUN=true;       shift   ;;
    --restart-only)  RESTART_ONLY=true;  shift   ;;
    --skip-verify)   SKIP_VERIFY=true;   shift   ;;
    -h|--help)
      cat <<'HELP'
Usage: deploy.sh [options]

Options:
  --domain DOMAIN         Auth server domain (e.g. www.xiao.org.cn)
  --download-url URL      OSS download URL
  --key-dir DIR           HMAC key storage (default: /opt/safemask-auth/config)
  --config-dir DIR        Server.js directory (default: /opt/safemask-auth)
  --port PORT             Node.js listen port (default: 35347)
  --dry-run               Print actions without executing
  --restart-only          Only restart PM2, skip verification
  --skip-verify           Skip post-deployment curl tests
  -h, --help              Show this help

Examples:
  sudo bash deploy.sh --domain www.xiao.org.cn --download-url https://obs.be.com:9004/...
  sudo bash deploy.sh --restart-only
HELP
      exit 0 ;;
    *) echo "[FATAL] Unknown: $1"; exit 1 ;;
  esac
done

# ─── Helpers ─────────────────────────────────────────────────
GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; CYAN='\033[0;36m'; NC='\033[0m'
info()    { echo -e "${GREEN}[INFO]${NC}  $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; }
section() { echo -e "\n${CYAN}═══════════════════════════════════════════════════${NC}"; echo -e "${CYAN}  $1${NC}"; echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"; }

_run() {
  if [[ "$DRY_RUN" == true ]]; then
    echo -e "  ${YELLOW}▶${NC} $*"
  else
    echo -e "  ${GREEN}▶${NC} $*"
    "$@"
  fi
}

guard_cmd() {
  if ! command -v "$1" &>/dev/null; then
    error "$1 未安装. $2"
    exit 1
  fi
}

# Node.js 查找（兼容 sudo 的 secure_path 隔离）
find_node_or_exit() {
  local node_bin

  node_bin=$(command -v node 2>/dev/null || true)
  if [[ -z "$node_bin" ]]; then
    node_bin=$(which node 2>/dev/null || true)
  fi

  if [[ -z "$node_bin" ]]; then
    # sudo 环境 common 路径回退
    for candidate in \
      /usr/local/bin/node \
      /usr/bin/node \
      /opt/nodejs/bin/node \
      /opt/node/bin/node \
      /usr/local/nvm/versions/node/*/bin/node \
      "$HOME"/.nvm/versions/node/*/bin/node \
      /root/.nvm/versions/node/*/bin/node; do
      # glob 展开 (处理 v*/bin/node)
      shopt -s nullglob 2>/dev/null || true
      for match in $candidate; do
        if [[ -x "$match" ]]; then
          node_bin="$match"
          break 2
        fi
      done
      shopt -u nullglob 2>/dev/null || true
    done
  fi

  if [[ -z "$node_bin" ]]; then
    error "找不到 node 可执行文件"
    error "当前 PATH: $PATH"
    error "请安装 Node.js ≥ 18 或手动将 node 加入 PATH"
    exit 1
  fi

  # 将 node 所在目录加入 PATH，确保后续 node 调用成功
  local node_dir
  node_dir=$(dirname "$node_bin")
  if [[ ":$PATH:" != *":$node_dir:"* ]]; then
    export PATH="$node_dir:$PATH"
  fi

  local ver
  ver=$("$node_bin" -v | sed 's/v//' | cut -d. -f1)
  if [[ "$ver" -lt 18 ]]; then
    error "Node.js ≥ 18 需要, 当前: $("$node_bin" -v)"
    exit 1
  fi
  info "Node.js $("$node_bin" -v) ✓"
}

verify_curl() {
  local desc="$1" url="$2" expected="$3"
  if [[ "$DRY_RUN" == true ]]; then
    echo -e "  ${YELLOW}▶${NC} [VERIFY] $desc → 期望 HTTP $expected"
    return 0
  fi
  local code
  code=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$url" 2>/dev/null || echo "000")
  if [[ "$code" == "$expected" ]]; then
    echo -e "  ${GREEN}✓${NC} $desc → HTTP $code"
  else
    echo -e "  ${RED}✗${NC} $desc → 期望 HTTP $expected, 实际 $code"
    return 1
  fi
}

# ─── Prereq checks ──────────────────────────────────────────
section "前置检查"

if [[ "$(id -u)" -ne 0 && "$DRY_RUN" != true ]]; then
  warn "部分操作需要 root 权限（写入 /opt）"
  warn "建议使用: sudo bash $0 ..."
fi

find_node_or_exit
guard_cmd openssl "请安装 openssl"
guard_cmd pm2    "请执行: npm install -g pm2"
info "openssl ✓"
info "PM2 ✓"

if [[ "$RESTART_ONLY" != true && -z "$DOMAIN" ]]; then
  error "--domain 是必填参数"
  exit 1
fi
if [[ "$RESTART_ONLY" != true && -z "$DOWNLOAD_URL" ]]; then
  error "--download-url 是必填参数"
  exit 1
fi

# ─── 1. HMAC Key ─────────────────────────────────────────
section "1/4 · HMAC 密钥"

if [[ ! -f "$KEY_FILE" ]]; then
  info "未发现 HMAC 密钥, 正在生成 ..."
  if [[ "$DRY_RUN" == true ]]; then
    _run bash "$INIT_KEY_SCRIPT" --dir "$KEY_DIR" --dry-run
  else
    bash "$INIT_KEY_SCRIPT" --dir "$KEY_DIR"
  fi
else
  info "使用现有密钥: $KEY_FILE"
fi

if [[ "$DRY_RUN" != true ]]; then
  HMAC_KEY=$(cat "$KEY_FILE")
else
  HMAC_KEY="<dry-run-masked>"
fi

# ─── 2. Deploy server.js ────────────────────────────────
section "2/4 · 部署 server.js"

if [[ ! -f "$CONFIG_DIR/server.js" ]]; then
  if [[ -f "$SCRIPT_DIR/server.js" ]]; then
    _run cp "$SCRIPT_DIR/server.js" "$CONFIG_DIR/server.js"
  else
    error "server.js 不存在于 $SCRIPT_DIR/server.js"
    error "请先将 server.js 上传到服务器"
    exit 1
  fi
else
  info "server.js 已存在: $CONFIG_DIR/server.js"
fi

# ─── 3. PM2 Service ─────────────────────────────────────
section "3/4 · PM2 服务管理"

# Stop existing if any
if pm2 list 2>/dev/null | grep -q "$PM2_NAME"; then
  _run pm2 delete "$PM2_NAME"
fi

# Start with env vars (export to pass to PM2 child process)
if [[ "$DRY_RUN" == true ]]; then
  echo -e "  ${YELLOW}▶${NC} export HMAC_KEY='$HMAC_KEY'"
  echo -e "  ${YELLOW}▶${NC} export DOWNLOAD_URL='$DOWNLOAD_URL'"
  echo -e "  ${YELLOW}▶${NC} export PORT=$PROXY_PORT"
  echo -e "  ${YELLOW}▶${NC} pm2 start $CONFIG_DIR/server.js --name $PM2_NAME --update-env"
else
  export HMAC_KEY
  export DOWNLOAD_URL
  export PORT
  pm2 start "$CONFIG_DIR/server.js" --name "$PM2_NAME" --update-env
fi

_run pm2 save
_run pm2 status

# ─── 4. Verification ────────────────────────────────────
if [[ "$SKIP_VERIFY" != true && "$RESTART_ONLY" != true ]]; then
  section "4/4 · 部署验证"
  info "验证中 (域名: https://$DOMAIN) ...\n"

  BASE="https://$DOMAIN"
  FAIL_COUNT=0

  verify_curl "GET /health (健康检查)"        "$BASE/health" "200" || ((FAIL_COUNT++))
  verify_curl "GET /download (无 token)"       "$BASE/download" "400" || ((FAIL_COUNT++))
  verify_curl "GET / (无效路径)"                "$BASE/" "404" || ((FAIL_COUNT++))
  verify_curl "GET /download?token=invalid:device:0 (无效 token)" \
    "$BASE/download?token=invalid:device:0" "403" || ((FAIL_COUNT++))

  # Generate a valid token and test
  if [[ "$DRY_RUN" != true ]]; then
    _verify_device_id="test-verify-$(date +%s)"
    _verify_ts=$(date +%s)
    _verify_sig=$(echo -n "$_verify_device_id:$_verify_ts" | openssl dgst -sha256 -hmac "$HMAC_KEY" -binary | base64 -w0 | tr '+/' '-_' | tr -d '=')
    _verify_token="${_verify_sig}:${_verify_device_id}:${_verify_ts}"
    verify_curl "GET /download?token=有效token (应 302)" \
      "$BASE/download?token=$_verify_token" "302" || ((FAIL_COUNT++))
    verify_curl "同 token 再请求 (频率限制 → 429)" \
      "$BASE/download?token=$_verify_token" "429" || ((FAIL_COUNT++))

    echo ""
    if [[ "$FAIL_COUNT" -eq 0 ]]; then
      echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
      echo -e "${GREEN}  ✅ 全部 6 项验证通过 — 部署成功${NC}"
      echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
    else
      echo -e "${RED}  ❌ $FAIL_COUNT 项验证失败, 请检查日志${NC}"
    fi
  else
    echo -e "\n  ${YELLOW}▶${NC} [DRY] 跳过 token 生成和 curl 验证"
    echo -e "  ${YELLOW}▶${NC} 正常执行时会自动测试 6 项验证"
  fi
elif [[ "$DRY_RUN" == true ]]; then
  echo -e "\n  ${YELLOW}▶${NC} [DRY] --skip-verify 或 --restart-only, 跳过验证"
fi

# ─── Summary ────────────────────────────────────────────
section "部署摘要"
echo -e "  鉴权域名:    ${CYAN}https://$DOMAIN${NC}"
echo -e "  OSS 下载源:   ${CYAN}$DOWNLOAD_URL${NC}"
echo -e "  Node.js 端口: ${CYAN}$PROXY_PORT${NC}"
echo -e "  PM2 进程名:   ${CYAN}$PM2_NAME${NC}"
echo -e "  密钥位置:     ${CYAN}$KEY_FILE${NC}"
echo ""
info "查看实时日志: pm2 logs $PM2_NAME"
info "下一步: 在本地修改 WORKER_BASE_URL=https://$DOMAIN 并构建 Rust 二进制"
