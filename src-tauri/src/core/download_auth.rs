use std::path::Path;
use std::fs;
use uuid::Uuid;
use base64::Engine;

/// Worker 代理的公共 URL（由部署者更新为实际 Worker 域名）
pub const WORKER_BASE_URL: &str = "https://www.xiaosheng.com";

/// 令牌有效时长（秒）— 用于 Worker 端校验，Rust 侧仅作文档参考
#[allow(dead_code)]
pub const TOKEN_TTL_SECS: u64 = 3600;
/// HMAC 密钥来源：优先构建时环境变量，无则使用开发版保底
fn get_hmac_key() -> &'static str {
    option_env!("SAFEMASK_HMAC_KEY").unwrap_or("dev-fallback-key")
}

/// 读取或创建设备指纹（UUID v4），持久化到 `storage_dir/device_id`
pub fn get_or_create_device_id(storage_dir: &Path) -> String {
    let path = storage_dir.join("device_id");
    if path.exists() {
        if let Ok(id) = fs::read_to_string(&path) {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    let id = Uuid::new_v4().to_string();
    let _ = fs::write(&path, &id);
    id
}

/// 生成 HMAC-SHA256 下载令牌
///
/// 令牌格式: `base64url_nopad(HMAC-SHA256(key, device_id:timestamp)) : device_id : timestamp`
pub fn generate_download_token(device_id: &str) -> String {
    use hmac::Mac;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let message = format!("{}:{}", device_id, timestamp);

    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(get_hmac_key().as_bytes())
        .expect("HMAC key");
    mac.update(message.as_bytes());
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(mac.finalize().into_bytes());

    format!("{}:{}:{}", signature, device_id, timestamp)
}

/// 构造完整的 Worker 代理下载 URL（含实时生成的令牌）
pub fn generate_worker_url(device_id: &str) -> String {
    let token = generate_download_token(device_id);
    format!("{}/download?token={}", WORKER_BASE_URL, token)
}
