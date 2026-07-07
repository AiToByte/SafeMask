#!/usr/bin/env bash
# ============================================================
# SafeMask Auth — Deployment Verification Script
# ============================================================
# Standalone — no dependency on deploy.sh or init-key.sh.
# Only requires the HMAC key file to test token generation.
#
# Usage:
#   bash verify.sh                                          # test http://127.0.0.1:35347
#   bash verify.sh --url https://www.xiao.org.cn            # test via Nginx
#   bash verify.sh --download-check                         # also follow 302 → OSS
#   bash verify.sh --url https://www.xiao.org.cn --download-check
#   bash verify.sh --key-file /custom/path/hmac.key
# ============================================================
set -euo pipefail

# ─── Defaults ──────────────────────────────────────────────
BASE_URL="http://127.0.0.1:35347"
KEY_FILE="/opt/safemask-auth/config/hmac.key"
DOWNLOAD_CHECK=false

# ─── Colors ────────────────────────────────────────────────
GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()    { echo -e "${GREEN}[INFO]${NC}  $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; }
pass()    { echo -e "  ${GREEN}✓${NC} $1"; }
fail()    { echo -e "  ${RED}✗${NC} $1"; }

# ─── Parse args ────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --url)          BASE_URL="$2";    shift 2 ;;
    --key-file)     KEY_FILE="$2";    shift 2 ;;
    --download-check) DOWNLOAD_CHECK=true; shift ;;
    -h|--help)
      cat <<'HELP'
Usage: verify.sh [options]

Options:
  --url URL           Base URL to test (default: http://127.0.0.1:35347)
  --key-file PATH     HMAC key file path (default: /opt/safemask-auth/config/hmac.key)
  --download-check    Also follow 302 redirect and verify OSS file reachable
  -h, --help          Show this help

Examples:
  bash verify.sh
  bash verify.sh --url https://www.xiao.org.cn
  bash verify.sh --url https://www.xiao.org.cn --download-check
HELP
      exit 0 ;;
    *) error "Unknown: $1"; exit 1 ;;
  esac
done

# ─── Prereqs ───────────────────────────────────────────────
section() { echo -e "\n${CYAN}═══════════════════════════════════════════════════${NC}"; echo -e "${CYAN}  $1${NC}"; echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"; }

section "SafeMask Auth — 部署验证"
echo -e "  目标: ${CYAN}$BASE_URL${NC}"

if [[ ! -f "$KEY_FILE" ]]; then
  warn "HMAC 密钥文件不存在: $KEY_FILE"
  warn "token 有效性测试 (5, 6) 将会跳过"
  echo ""
fi

# ─── Helpers ───────────────────────────────────────────────
verify() {
  local desc="$1" url="$2" expected="$3"
  local code
  code=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$url" 2>/dev/null || echo "000")
  if [[ "$code" == "$expected" ]]; then
    pass "$desc → HTTP $code"
    return 0
  else
    fail "$desc → 期望 HTTP $expected, 实际 $code"
    return 1
  fi
}

# ─── 1 - Health ────────────────────────────────────────────
echo ""
verify "GET /health (健康检查)" "$BASE_URL/health" 200 || ((FAIL++))
verify "GET /download (无 token)" "$BASE_URL/download" 400 || ((FAIL++))
verify "GET / (无效路径)" "$BASE_URL/" 404 || ((FAIL++))
verify "GET /download?token=invalid:device:0 (无效 token)" \
  "$BASE_URL/download?token=invalid:device:0" 403 || ((FAIL++))

# ─── 5-6 - Token tests ────────────────────────────────────
if [[ -f "$KEY_FILE" ]]; then
  HMAC_KEY=$(cat "$KEY_FILE")
  _device_id="verify-$(date +%s)"
  _ts=$(date +%s)
  _sig=$(echo -n "$_device_id:$_ts" | openssl dgst -sha256 -hmac "$HMAC_KEY" -binary | base64 -w0 | tr '+/' '-_' | tr -d '=')
  _token="${_sig}:${_device_id}:${_ts}"

  verify "GET /download?token=有效token (期望 302)" \
    "$BASE_URL/download?token=$_token" 302 || ((FAIL++))
  verify "同 token 再请求 (频率限制 → 429)" \
    "$BASE_URL/download?token=$_token" 429 || ((FAIL++))
else
  warn "跳过 token 测试 (5, 6) — 缺少密钥文件"
  warn "  提供密钥: --key-file /path/to/hmac.key"
fi

# ─── Download check ────────────────────────────────────────
if [[ "$DOWNLOAD_CHECK" == true && -f "$KEY_FILE" ]]; then
  echo ""
  info "跟随 302 检查 OSS 文件可达性 ..."

  _dl_device="dlcheck-$(date +%s)"
  _dl_ts=$(date +%s)
  _dl_sig=$(echo -n "$_dl_device:$_dl_ts" | openssl dgst -sha256 -hmac "$HMAC_KEY" -binary | base64 -w0 | tr '+/' '-_' | tr -d '=')
  _dl_token="${_dl_sig}:${_dl_device}:${_dl_ts}"

  set +e
  _http_code=$(curl -s -o /tmp/safemask-oss-check.bin -w "%{http_code}" \
    --max-time 15 -L "$BASE_URL/download?token=$_dl_token" 2>/dev/null)
  _size=0
  if [[ -f /tmp/safemask-oss-check.bin ]]; then
    _size=$(stat -c%s /tmp/safemask-oss-check.bin 2>/dev/null || echo 0)
    rm -f /tmp/safemask-oss-check.bin
  fi
  set -e

  if [[ "$_http_code" == "200" && "$_size" -gt 0 ]]; then
    pass "302 → OSS 直连 → HTTP $_http_code, 下载 $_size bytes"
  else
    fail "302 → OSS 直连失败 (HTTP $_http_code, size $_size bytes)"
    fail "检查 OSS 地址是否可达, 或 OSS 服务器防火墙"
  fi
elif [[ "$DOWNLOAD_CHECK" == true ]]; then
  warn "跳过 OSS 文件可达性检查 — 缺少密钥文件"
fi

# ─── Summary ───────────────────────────────────────────────
section "结果"
if [[ "${FAIL:-0}" -eq 0 ]]; then
  if [[ -f "$KEY_FILE" ]]; then
    echo -e "  ${GREEN}✅ 全部 6 项验证通过 — 服务正常${NC}"
  else
    echo -e "  ${GREEN}✅ 4 项基础验证通过 — token 测试已跳过${NC}"
  fi
  if [[ "$DOWNLOAD_CHECK" == true && -f "$KEY_FILE" ]]; then
    echo -e "  ${GREEN}✅ OSS 文件可达性验证通过${NC}"
  fi
  exit 0
else
  echo -e "  ${RED}❌ ${FAIL:-0} 项验证失败, 请检查服务日志${NC}"
  exit 1
fi
