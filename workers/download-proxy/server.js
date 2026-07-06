// SafeMask AI 模型下载鉴权代理 — 自建服务器版
// 无需任何第三方依赖，仅使用 Node.js 内置模块
//
// 启动方式:
//   HMAC_KEY="your-secret" DOWNLOAD_URL="https://oss/..." node server.js
//
// 生产推荐（PM2）:
//   pm2 start server.js --name safemask-dl-auth --update-env

const http = require('http');
const crypto = require('crypto');

// ── 配置（通过环境变量覆盖） ────────────────────────

const HMAC_KEY = process.env.HMAC_KEY || 'dev-fallback-key';
const DOWNLOAD_URL = process.env.DOWNLOAD_URL;
const PORT = parseInt(process.env.PORT, 10) || 3000;
const TOKEN_TTL_SECS = 3600;            // 令牌有效期 1 小时
const MAX_PER_TOKEN = 1;                // 每令牌最多 1 次
const MAX_PER_IP_DAILY = 2;             // 每 IP 每天最多 2 次

// ── 计数器存储 ─────────────────────────────────────
// 内存 Map（进程重启清零，生产建议用 Redis）
// 如需 Redis，将 get/set 替换为 Redis 调用即可（接口兼容）
const tokenCounters = new Map();   // token:{sigPart} → count
const ipCounters = new Map();      // ip:{ip}:{YYYY-MM-DD} → count

// 定时清理过期计数器（每分钟一次，防内存泄露）
setInterval(() => {
  const now = Date.now();
  for (const [key, entry] of tokenCounters) {
    if (entry.expiresAt <= now) tokenCounters.delete(key);
  }
  for (const [key, entry] of ipCounters) {
    if (entry.expiresAt <= now) ipCounters.delete(key);
  }
}, 60_000);

// ── 核心函数 ────────────────────────────────────────

/**
 * 验证 HMAC 令牌
 * 令牌格式: base64url(HMAC-SHA256(key, deviceId:timestamp)) : deviceId : timestamp
 * 返回 { deviceId, sigPart } 或 null
 */
function verifyToken(token) {
  const parts = token.split(':');
  if (parts.length !== 3) return null;

  const [sig, deviceId, ts] = parts;
  const timestamp = parseInt(ts, 10);
  if (isNaN(timestamp)) return null;

  // 1. TTL 检查
  const ageSec = (Date.now() - timestamp * 1000) / 1000;
  if (ageSec > TOKEN_TTL_SECS) return null;

  // 2. HMAC 签名验证
  const expectedSig = crypto
    .createHmac('sha256', HMAC_KEY)
    .update(`${deviceId}:${ts}`)
    .digest('base64url');

  // 使用 timingSafeEqual 防止时序攻击
  const sigBuf = Buffer.from(sig);
  const expectedBuf = Buffer.from(expectedSig);
  if (sigBuf.length !== expectedBuf.length) return null;
  if (!crypto.timingSafeEqual(sigBuf, expectedBuf)) return null;

  return { deviceId, sigPart: sig };
}

/**
 * 频率限制检查（每令牌 N 次 + 每 IP 每天 M 次）
 */
function checkRateLimit(sigPart, ip) {
  const today = new Date().toISOString().slice(0, 10);

  // Token 级计数器
  const tokenKey = `token:${sigPart}`;
  const tokenEntry = tokenCounters.get(tokenKey) || { count: 0, expiresAt: Date.now() + TOKEN_TTL_SECS * 1000 };
  if (tokenEntry.count >= MAX_PER_TOKEN) {
    tokenCounters.set(tokenKey, tokenEntry);
    return false;
  }
  tokenEntry.count += 1;
  tokenCounters.set(tokenKey, tokenEntry);

  // IP 级每日计数器
  const ipKey = `ip:${ip}:${today}`;
  const ipEntry = ipCounters.get(ipKey) || { count: 0, expiresAt: Date.now() + 86400_000 };
  if (ipEntry.count >= MAX_PER_IP_DAILY) {
    ipCounters.set(ipKey, ipEntry);
    return false;
  }
  ipEntry.count += 1;
  ipCounters.set(ipKey, ipEntry);

  return true;
}

// ── HTTP 服务 ───────────────────────────────────────

if (!DOWNLOAD_URL) {
  console.error('❌ 必须设置 DOWNLOAD_URL 环境变量');
  process.exit(1);
}

const server = http.createServer((req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);

  // 健康检查接口（无性能开销，客户端用于快速降级感知）
  if (url.pathname === '/health' && req.method === 'GET') {
    res.writeHead(200, {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
    });
    res.end(JSON.stringify({ status: 'UP', timestamp: Date.now() }));
    return;
  }

  // 仅允许 GET + /download 路径
  if (req.method !== 'GET' || url.pathname !== '/download') {
    res.writeHead(404, { 'Content-Type': 'text/plain' });
    res.end('Not found');
    return;
  }

  const token = url.searchParams.get('token');
  if (!token) {
    res.writeHead(400, { 'Content-Type': 'text/plain' });
    res.end('Bad request');
    return;
  }

  // 1. 验证令牌
  const verified = verifyToken(token);
  if (!verified) {
    res.writeHead(403, { 'Content-Type': 'text/plain' });
    res.end('Forbidden');
    return;
  }

  // 2. 频率限制（安全提取真实客户端 IP）
  const rawIp = req.headers['x-real-ip']
    || req.headers['x-forwarded-for']?.split(',')[0]?.trim()
    || req.socket.remoteAddress
    || 'unknown';
  const ip = rawIp.startsWith('::ffff:') ? rawIp.slice(7) : rawIp;
  if (!checkRateLimit(verified.sigPart, ip)) {
    res.writeHead(429, { 'Content-Type': 'text/plain' });
    res.end('Rate limited');
    return;
  }

  // 3. 302 重定向到真实下载 URL
  res.writeHead(302, { Location: DOWNLOAD_URL });
  res.end();
});

server.listen(PORT, () => {
  console.log(`✅ SafeMask 鉴权代理运行中: http://0.0.0.0:${PORT}`);
  console.log(`   重定向目标: ${DOWNLOAD_URL}`);
});
