/// <reference types="@cloudflare/workers-types" />

interface Env {
  SAFEMASK_DL: KVNamespace;
  HMAC_KEY: string;
  DOWNLOAD_URL: string;
}

// ── 配置 ────────────────────────────────────────────
const TOKEN_TTL_SECS = 3600;           // 1 小时
const MAX_DOWNLOADS_PER_TOKEN = 1;     // 每令牌 1 次
const MAX_DOWNLOADS_PER_IP_DAILY = 2;  // 每 IP 每天 2 次

// ── 工具函数 ────────────────────────────────────────

/** URL-safe base64 解码 */
function base64UrlDecode(s: string): Uint8Array {
  const base64 = s.replace(/-/g, '+').replace(/_/g, '/') + '=='.slice(0, (4 - s.length % 4) % 4);
  return Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
}

/** HMAC-SHA256 (Web Crypto) */
async function hmacSha256(key: string, data: string): Promise<string> {
  const encoder = new TextEncoder();
  const cryptoKey = await crypto.subtle.importKey(
    'raw', encoder.encode(key), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']
  );
  const sigBytes = await crypto.subtle.sign('HMAC', cryptoKey, encoder.encode(data));
  return btoa(String.fromCharCode(...new Uint8Array(sigBytes)))
    .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

// ── 令牌验证 ────────────────────────────────────────

async function verifyToken(token: string, env: Env): Promise<{ ok: true; deviceId: string } | { ok: false; status: number; body: string }> {
  const parts = token.split(':');
  if (parts.length !== 3) {
    return { ok: false, status: 400, body: 'Bad request' };
  }

  const [sig, deviceId, ts] = parts;
  const timestamp = parseInt(ts, 10);
  if (isNaN(timestamp)) {
    return { ok: false, status: 400, body: 'Bad request' };
  }

  // 1. TTL 检查
  const ageSec = (Date.now() - timestamp * 1000) / 1000;
  if (ageSec > TOKEN_TTL_SECS) {
    return { ok: false, status: 403, body: 'Token expired' };
  }

  // 2. HMAC 签名验证
  const expectedSig = await hmacSha256(env.HMAC_KEY, `${deviceId}:${ts}`);
  if (sig !== expectedSig) {
    return { ok: false, status: 403, body: 'Forbidden' };
  }

  return { ok: true, deviceId };
}

// ── 频率限制 ────────────────────────────────────────

async function checkRateLimit(env: Env, deviceId: string, clientIp: string): Promise<{ ok: true } | { ok: false; status: number; body: string }> {
  // 2a. 每令牌计数器（key = "token:{signature}"，TTL = 令牌剩余有效期）
  // 由 URL 参数中的 signature 段作为 key
  // 注意: 这里只在下载成功后加 1，见下方 fetch handler

  // 2b. 每 IP 每天计数器
  const today = new Date().toISOString().slice(0, 10);
  const ipKey = `ip:${clientIp}:${today}`;
  const ipCount = await env.SAFEMAKER_DL.get(ipKey);
  const ipNum = parseInt(ipCount ?? '0', 10);
  if (ipNum >= MAX_DOWNLOADS_PER_IP_DAILY) {
    return { ok: false, status: 429, body: 'Rate limited: daily IP quota exceeded' };
  }

  return { ok: true };
}

// ── 请求入口 ────────────────────────────────────────

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // 仅允许 GET + /download 路径
    if (request.method !== 'GET' || url.pathname !== '/download') {
      return new Response('Not found', { status: 404 });
    }

    const token = url.searchParams.get('token');
    if (!token) {
      return new Response('Bad request', { status: 400 });
    }

    // 1. 验证令牌
    const verified = await verifyToken(token, env);
    if (!verified.ok) {
      return new Response(verified.body, { status: verified.status });
    }

    const clientIp = request.headers.get('CF-Connecting-IP') ?? 'unknown';

    // 2. 频率限制
    const rateLimit = await checkRateLimit(env, verified.deviceId, clientIp);
    if (!rateLimit.ok) {
      return new Response(rateLimit.body, { status: rateLimit.status });
    }

    // 3. 更新计数器（先检查 token 级：同一 token 最多 MAX_DOWNLOADS_PER_TOKEN 次）
    // 使用 signature 段作为 token ID
    const sigPart = token.split(':')[0];
    const tokenKey = `token:${sigPart}`;
    const tokenCount = await env.SAFEMAKER_DL.get(tokenKey);
    const tokenNum = parseInt(tokenCount ?? '0', 10);
    if (tokenNum >= MAX_DOWNLOADS_PER_TOKEN) {
      return new Response('Rate limited: token exhausted', { status: 429 });
    }
    await env.SAFEMAKER_DL.put(tokenKey, String(tokenNum + 1), { expirationTtl: TOKEN_TTL_SECS });

    // IP 计数器（每日）
    const today = new Date().toISOString().slice(0, 10);
    const ipKey = `ip:${clientIp}:${today}`;
    const ipCount = parseInt((await env.SAFEMAKER_DL.get(ipKey)) ?? '0', 10);
    await env.SAFEMAKER_DL.put(ipKey, String(ipCount + 1), { expirationTtl: 86400 });

    // 4. 302 重定向到真实下载 URL
    return Response.redirect(env.DOWNLOAD_URL, 302);
  },
};
