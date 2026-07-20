use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use log::{info, error, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use std::time::{Duration, Instant};
use crate::infra::ai::model_manager::validate_model_dir;

// ── Consts ──
const DOWNLOAD_TIMEOUT: u64 = 300;          // 5 min
const REQUIRED_SPACE: u64 = 2 * 1024 * 1024 * 1024; // 2 GB
const PROGRESS_INTERVAL_MS: u128 = 100;
const MAX_RETRIES: u32 = 1;
const RETRY_DELAY_SECS: u64 = 3;
const STALE_LOCK_MINS: u64 = 10;
const LOCK_FILE: &str = ".model_downloading";

/// 允许的下载 host 白名单
/// 与 `AppSettings::default().model_download_urls` 保持一致
const ALLOWED_HOSTS: &[&str] = &[
    "obs.behource.com",
    "950544b1401caf10f82ba1e82b03f89a.r2.cloudflarestorage.com",
    "github.com",
    "www.xiaosheng.com",
    "xiaosheng.com",
];

// ── Types ──

#[derive(Clone, Serialize)]
struct ProgressPayload {
    percentage: f64,
    downloaded_bytes: u64,
    total_bytes: u64,
    speed_mbps: f64,
}

#[derive(Default)]
pub struct DownloadState {
    pub cancel_flag: Arc<AtomicBool>,
}

// ── Commands ──

/// 检查模型文件完整性
/// 优先级: .exe 同级 → AppData → 便携式; 支持 privacy-filter/ 子目录、根级文件、子目录三种布局
#[tauri::command]
pub fn check_model_file(app: AppHandle) -> Result<String, String> {
    // Priority 0: .exe 同级 models/
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent() {
            let base = exe_dir.join("models");
            clean_stale_lock(&base.join("privacy-filter").join(LOCK_FILE));
            if has_valid_model_in(&base) {
                return Ok("READY_PORTABLE".to_string());
            }
        }
    // Priority 1: AppData/models/
    if let Ok(local) = app.path().app_local_data_dir() {
        let base = local.join("models");
        clean_stale_lock(&base.join("privacy-filter").join(LOCK_FILE));
        if has_valid_model_in(&base) {
            return Ok("READY".to_string());
        }
    }
    // Priority 2: 资源目录 models/
    if let Ok(portable) = app.path().resource_dir()
        && has_valid_model_in(&portable.join("models")) {
            return Ok("READY_PORTABLE".to_string());
        }
    Ok("MISSING".to_string())
}

/// 扫描 parent 目录下所有可能的模型布局
fn has_valid_model_in(parent: &Path) -> bool {
    if !parent.exists() {
        return false;
    }
    // 快速路径: privacy-filter/ 子目录（下载后的标准位置）
    if validate_model_dir(&parent.join("privacy-filter")) {
        return true;
    }
    // 根级文件直接放置
    if validate_model_dir(parent) {
        return true;
    }
    // 其他子目录
    if let Ok(entries) = std::fs::read_dir(parent) {
        entries.filter_map(Result::ok).any(|e| e.path().is_dir() && validate_model_dir(&e.path()))
    } else {
        false
    }
}

/// 清理过期锁文件
fn clean_stale_lock(lock_path: &Path) {
    if !lock_path.exists() {
        return;
    }
    if let Ok(meta) = std::fs::metadata(lock_path)
        && let Ok(modified) = meta.modified()
            && let Ok(elapsed) = modified.elapsed()
                && elapsed > Duration::from_secs(STALE_LOCK_MINS * 60) {
                    let _ = std::fs::remove_file(lock_path);
                }
}

/// 启动下载与解压流水线
///
/// # 参数
/// - `url`：下载 URL，必须是 HTTPS 且 host 在 [`ALLOWED_HOSTS`] 白名单内
/// - `expected_sha256`：可选的 SHA-256 校验值（小写 hex）。提供时强制校验；未提供时会输出警告日志但继续。
#[tauri::command]
pub async fn start_model_download(
    app: AppHandle,
    state: tauri::State<'_, DownloadState>,
    url: String,
    expected_sha256: Option<String>,
) -> Result<String, String> {
    // ── URL 校验：仅允许 HTTPS + 白名单 host（防 SSRF / 明文下载） ──
    validate_download_url(&url)?;
    info!("[ModelDownload] start_model_download called, url={}", redact_url(&url));
    if expected_sha256.is_none() {
        warn!("[ModelDownload] expected_sha256 未提供，跳过完整性校验（存在供应链风险）");
    }

    state.cancel_flag.store(false, Ordering::SeqCst);
    let cancel_token = state.cancel_flag.clone();

    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?;
    let exe_dir = exe_path.parent()
        .ok_or_else(|| "无法获取可执行文件目录".to_string())?;
    let model_dir = exe_dir.join("models").join("privacy-filter");
    tokio::fs::create_dir_all(&model_dir)
        .await
        .map_err(|e| e.to_string())?;

    // 磁盘空间检测
    if let Ok(available_space) = fs2::available_space(&model_dir) {
        if available_space < REQUIRED_SPACE {
            error!("[ModelDownload] insufficient disk space: {} bytes", available_space);
            return Err("DISK_SPACE_LOW".to_string());
        }
        info!("[ModelDownload] disk space OK: {} bytes", available_space);
    }

    let zip_path = model_dir.join("privacy-filter.zip");
    let lock_path = model_dir.join(LOCK_FILE);

    // ── 原子化占用锁：create_new 只在文件不存在时成功 ──
    // 先清理过期残留锁，再尝试独占创建
    clean_stale_lock(&lock_path);
    let lock_guard = LockGuard::acquire(&lock_path)?;
    info!("[ModelDownload] lock acquired: {}", lock_path.display());

    let app_clone = app.clone();

    tokio::spawn(async move {
        let result = perform_download_pipeline(
            &app_clone, &url, &zip_path, &model_dir, cancel_token, expected_sha256,
        )
        .await;

        // 锁在 lock_guard drop 时自动清理
        drop(lock_guard);

        match result {
            Ok(_) => {
                info!("[ModelDownload] pipeline completed successfully");

                // 通知前端下载完成
                let _ = app_clone.emit("model-download-status", "READY");

                // 重新初始化 AI 引擎（下载前可能因模型缺失而跳过）
                let state = app_clone.state::<crate::common::state::AppState>();
                let mut guard = state.engine.write();
                if let Some(engine_ref) = std::sync::Arc::get_mut(&mut *guard) {
                    engine_ref.enable_ai_engine(&model_dir);
                    info!("🤖 AI 引擎已重新初始化（下载后）");
                } else {
                    warn!("⚠️ 引擎引用不唯一，跳过 AI 引擎热加载");
                }
            }
            Err(e) => {
                error!("[ModelDownload] pipeline failed: {}", e);
                // 失败时清理残留 zip
                let _ = tokio::fs::remove_file(&zip_path).await;
                let _ = app_clone.emit("model-download-status", format!("ERROR: {}", e));
            }
        }
    });

    Ok("STARTED".to_string())
}

/// 校验下载 URL：必须是 HTTPS 且 host 在白名单内
fn validate_download_url(url: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|e| format!("URL 解析失败: {}", e))?;

    if parsed.scheme() != "https" {
        return Err(format!("仅允许 HTTPS，收到 scheme: {}", parsed.scheme()));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL 缺少 host".to_string())?;

    if !ALLOWED_HOSTS.iter().any(|allowed| host.eq_ignore_ascii_case(allowed)) {
        return Err(format!("host 不在白名单: {}", host));
    }

    Ok(())
}

/// 脱敏 URL 中的凭据（供日志输出使用）
fn redact_url(url: &str) -> String {
    match reqwest::Url::parse(url) {
        Ok(mut u) => {
            let _ = u.set_username("");
            let _ = u.set_password(None);
            u.to_string()
        }
        Err(_) => url.to_string(),
    }
}

/// 独占锁守卫：drop 时自动删除锁文件
struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    fn acquire(lock_path: &Path) -> Result<Self, String> {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(mut file) => {
                use std::io::Write;
                // 写入 PID + 时间戳，便于诊断
                let payload = format!(
                    "pid={} started_at={}",
                    std::process::id(),
                    chrono::Local::now().to_rfc3339()
                );
                let _ = file.write_all(payload.as_bytes());
                Ok(Self { path: lock_path.to_path_buf() })
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err("DOWNLOAD_IN_PROGRESS".to_string())
            }
            Err(e) => Err(format!("锁文件创建失败: {}", e)),
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// 取消当前下载任务
#[tauri::command]
pub fn cancel_model_download(state: tauri::State<'_, DownloadState>) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

// ── Pipeline ──

async fn perform_download_pipeline(
    app: &AppHandle,
    url: &str,
    zip_path: &Path,
    extract_to: &Path,
    cancel_token: Arc<AtomicBool>,
    expected_sha256: Option<String>,
) -> Result<(), String> {
    info!("[ModelDownload] performing download pipeline, url={}", redact_url(url));
    let client = build_client()?;

    let mut attempt: u32 = 0;

    loop {
        attempt += 1;
        info!("[ModelDownload] download attempt {}/{}", attempt, MAX_RETRIES + 1);

        if cancel_token.load(Ordering::SeqCst) {
            info!("[ModelDownload] cancelled before attempt {}", attempt);
            return Err("DOWNLOAD_CANCELLED".to_string());
        }

        match download_file(app, &client, url, zip_path, cancel_token.clone()).await {
            Ok(()) => {
                info!("[ModelDownload] download attempt {} succeeded", attempt);
                break;
            }
            Err(e) => {
                if e == "DOWNLOAD_CANCELLED" {
                    info!("[ModelDownload] cancelled during attempt {}", attempt);
                    return Err(e);
                }
                if attempt <= MAX_RETRIES {
                    error!("[ModelDownload] attempt {} failed: {} — retrying in {}s", attempt, e, RETRY_DELAY_SECS);
                    let _ = app.emit(
                        "model-download-status",
                        format!("RETRYING: {}/{}", attempt, MAX_RETRIES + 1),
                    );
                    tokio::time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                    continue;
                }
                error!("[ModelDownload] all {} attempts exhausted, last error: {}", attempt, e);
                return Err(e);
            }
        }
    }

    // 下载完成 → 若提供了期望 hash，先做 SHA-256 完整性校验
    if let Some(expected) = expected_sha256.as_deref() {
        info!("[ModelDownload] verifying SHA-256 against expected hash");
        let _ = app.emit("model-download-status", "VERIFYING");
        let zip_path_buf = zip_path.to_path_buf();
        let expected_owned = expected.to_ascii_lowercase();
        let actual = tokio::task::spawn_blocking(move || compute_sha256(&zip_path_buf))
            .await
            .map_err(|e| format!("SHA-256 校验任务失败: {}", e))??;
        if actual != expected_owned {
            error!(
                "[ModelDownload] SHA-256 mismatch: expected={} actual={}",
                expected_owned, actual
            );
            let _ = tokio::fs::remove_file(zip_path).await;
            return Err(format!("CHECKSUM_MISMATCH: {}", actual));
        }
        info!("[ModelDownload] SHA-256 verified: {}", actual);
    } else {
        warn!("[ModelDownload] no expected SHA-256 provided, skipping integrity check");
    }

    // 解压
    info!("[ModelDownload] starting extraction");
    let _ = app.emit("model-download-status", "EXTRACTING");

    let zip_path_buf = zip_path.to_path_buf();
    let extract_to_buf = extract_to.to_path_buf();

    tokio::task::spawn_blocking(move || {
        info!("[ModelDownload] extracting zip to {:?}", extract_to_buf);
        extract_zip(&zip_path_buf, &extract_to_buf)
    })
        .await
        .map_err(|e| e.to_string())??;

    info!("[ModelDownload] extraction complete, removing zip");
    let _ = std::fs::remove_file(zip_path);
    Ok(())
}

async fn download_file(
    app: &AppHandle,
    client: &reqwest::Client,
    url: &str,
    zip_path: &Path,
    cancel_token: Arc<AtomicBool>,
) -> Result<(), String> {
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("网络连接失败: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("HTTP 状态码错误: {}", res.status()));
    }

    let total_size: Option<u64> = res.content_length();
    info!(
        "[ModelDownload] HTTP {} — total_size={:?}",
        res.status(),
        total_size
    );
    let mut file = File::create(zip_path)
        .await
        .map_err(|e| format!("写入文件失败: {}", e))?;
    let mut stream = res.bytes_stream();

    let mut downloaded: u64 = 0;
    let start_time = Instant::now();
    let mut last_emit = Instant::now();

    while let Some(chunk_result) = stream.next().await {
        if cancel_token.load(Ordering::SeqCst) {
            drop(file);
            let _ = tokio::fs::remove_file(zip_path).await;
            return Err("DOWNLOAD_CANCELLED".to_string());
        }

        let chunk = chunk_result.map_err(|e| format!("数据流读取中断: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("磁盘写入失败: {}", e))?;
        downloaded += chunk.len() as u64;

        // 每 100ms 或下载完成时发射进度
        let should_emit = last_emit.elapsed().as_millis() >= PROGRESS_INTERVAL_MS
            || total_size.is_some_and(|ts| downloaded >= ts);

        if should_emit {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed_mbps = if elapsed > 0.0 {
                (downloaded as f64 / (1024.0 * 1024.0)) / elapsed
            } else {
                0.0
            };
            let percentage = total_size
                .map(|ts| (downloaded as f64 / ts as f64) * 100.0)
                .unwrap_or(0.0);

            let _ = app.emit(
                "model-download-progress",
                ProgressPayload {
                    percentage,
                    downloaded_bytes: downloaded,
                    total_bytes: total_size.unwrap_or(downloaded),
                    speed_mbps,
                },
            );
            last_emit = Instant::now();
        }
    }

    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    Ok(())
}

/// 从环境变量自动检测 HTTP 代理
///
/// 优先级：HTTPS_PROXY > HTTP_PROXY > https_proxy > http_proxy。
/// 日志中的代理 URL 会脱敏用户名/密码。
fn build_client() -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT));

    for var in ["HTTPS_PROXY", "HTTP_PROXY", "https_proxy", "http_proxy"] {
        if let Ok(proxy_url) = std::env::var(var) {
            if proxy_url.is_empty() {
                continue;
            }
            info!("[ModelDownload] detected proxy {}={}", var, redact_url(&proxy_url));
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => builder = builder.proxy(proxy),
                Err(e) => warn!("[ModelDownload] proxy parse failed: {}", e),
            }
            break;
        }
    }

    builder.build().map_err(|e| {
        error!("[ModelDownload] failed to build reqwest client: {}", e);
        e.to_string()
    })
}

/// 同步解压函数
fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), String> {
    info!("[ModelDownload] opening zip archive: {:?}", zip_path);
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let file_count = archive.len();
    info!("[ModelDownload] zip contains {} entries", file_count);

    for i in 0..file_count {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_to.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent()
                && !p.exists() {
                    std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    info!("[ModelDownload] extracted {} entries to {:?}", file_count, extract_to);
    Ok(())
}

// ── Helpers ──

/// 计算文件 SHA-256（同步，供 spawn_blocking 使用）
fn compute_sha256(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("打开文件失败: {}", e))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("读取失败: {}", e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(hex_encode(&digest))
}

/// 简易十六进制编码（避免引入 hex crate）
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_download_url_accepts_whitelisted_https() {
        assert!(validate_download_url("https://github.com/AiToByte/SafeMask/releases/download/v2.0.0/privacy-filter.zip").is_ok());
        assert!(validate_download_url("https://obs.behource.com:9004/gxzh/2026/07/06/privacy-filter.zip").is_ok());
    }

    #[test]
    fn validate_download_url_rejects_http() {
        let err = validate_download_url("http://github.com/some.zip").unwrap_err();
        assert!(err.contains("HTTPS"));
    }

    #[test]
    fn validate_download_url_rejects_untrusted_host() {
        let err = validate_download_url("https://evil.example.com/model.zip").unwrap_err();
        assert!(err.contains("白名单"));
    }

    #[test]
    fn validate_download_url_rejects_garbage() {
        assert!(validate_download_url("").is_err());
        assert!(validate_download_url("not a url").is_err());
    }

    #[test]
    fn redact_url_strips_credentials() {
        let redacted = redact_url("https://user:secret@github.com/path");
        assert!(!redacted.contains("user"));
        assert!(!redacted.contains("secret"));
        assert!(redacted.contains("github.com"));
    }

    #[test]
    fn hex_encode_round_trip() {
        assert_eq!(hex_encode(&[0x00, 0xff, 0xab]), "00ffab");
        assert_eq!(hex_encode(&[]), "");
    }
}
