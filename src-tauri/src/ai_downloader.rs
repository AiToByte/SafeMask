use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use log::{info, error};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use std::time::{Duration, Instant};

// ── Consts ──
const DOWNLOAD_TIMEOUT: u64 = 300;          // 5 min
const REQUIRED_SPACE: u64 = 2 * 1024 * 1024 * 1024; // 2 GB
const PROGRESS_INTERVAL_MS: u128 = 100;
const MAX_RETRIES: u32 = 1;
const RETRY_DELAY_SECS: u64 = 3;
const STALE_LOCK_MINS: u64 = 10;
const LOCK_FILE: &str = ".model_downloading";

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

/// 检查模型文件完整性 (四件套校验)
#[tauri::command]
pub fn check_model_file(app: AppHandle) -> Result<String, String> {
    let portable_dir = app
        .path()
        .resource_dir()
        .unwrap_or_default()
        .join("models")
        .join("privacy-filter");
    if verify_model_integrity(&portable_dir) {
        return Ok("READY_PORTABLE".to_string());
    }

    let local_dir = app
        .path()
        .app_local_data_dir()
        .unwrap_or_default()
        .join("models")
        .join("privacy-filter");
    if verify_model_integrity(&local_dir) {
        // 清除残留锁文件（若有）
        let _ = std::fs::remove_file(local_dir.join(LOCK_FILE));
        return Ok("READY".to_string());
    }

    // 清理过期锁文件
    let lock_path = local_dir.join(LOCK_FILE);
    if lock_path.exists() {
        if let Ok(meta) = std::fs::metadata(&lock_path) {
            if let Ok(modified) = meta.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed > Duration::from_secs(STALE_LOCK_MINS * 60) {
                        let _ = std::fs::remove_file(&lock_path);
                    }
                }
            }
        }
    }

    Ok("MISSING".to_string())
}

/// 启动下载与解压流水线
#[tauri::command]
pub async fn start_model_download(
    app: AppHandle,
    state: tauri::State<'_, DownloadState>,
    url: String,
) -> Result<String, String> {
    info!("[ModelDownload] start_model_download called, url={}", url);
    state.cancel_flag.store(false, Ordering::SeqCst);
    let cancel_token = state.cancel_flag.clone();

    let local_data = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?;
    let model_dir = local_data.join("models").join("privacy-filter");
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
    let app_clone = app.clone();

    // 写入下载锁标记
    info!("[ModelDownload] writing lock file");
    let _ = std::fs::write(&lock_path, "downloading");

    tokio::spawn(async move {
        let result = perform_download_pipeline(
            &app_clone, &url, &zip_path, &model_dir, cancel_token,
        )
        .await;

        // 清理锁文件
        let _ = std::fs::remove_file(&lock_path);

        match result {
            Ok(_) => {
                info!("[ModelDownload] pipeline completed successfully");
                let _ = app_clone.emit("model-download-status", "READY");
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
) -> Result<(), String> {
    info!("[ModelDownload] performing download pipeline, url={}", url);
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

    // 下载完成 → 解压
    info!("[ModelDownload] download complete, starting extraction");
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
            || total_size.map_or(false, |ts| downloaded >= ts);

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
fn build_client() -> Result<reqwest::Client, String> {
    // 检测代理
    for var in ["HTTPS_PROXY", "HTTP_PROXY", "https_proxy", "http_proxy"] {
        if let Ok(proxy_env) = std::env::var(var) {
            if !proxy_env.is_empty() {
                info!("[ModelDownload] detected proxy {}={}", var, proxy_env);
                break;
            }
        }
    }
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT));

    // 自动检测代理：HTTPS_PROXY > HTTP_PROXY > https_proxy > http_proxy
    for var in ["HTTPS_PROXY", "HTTP_PROXY", "https_proxy", "http_proxy"] {
        if let Ok(proxy_url) = std::env::var(var) {
            if !proxy_url.is_empty() {
                if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                    builder = builder.proxy(proxy);
                }
                break;
            }
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
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    info!("[ModelDownload] extracted {} entries to {:?}", file_count, extract_to);
    Ok(())
}

// ── Helpers ──

/// 内部函数：验证4个核心模型组件是否健全
fn verify_model_integrity(dir: &Path) -> bool {
    let files = [
        "model_q4.onnx",
        "model_q4.onnx_data",
        "tokenizer.json",
        "config.json",
    ];
    for file in &files {
        let path = dir.join(file);
        if !path.exists() {
            return false;
        }
        if let Ok(meta) = std::fs::metadata(&path) {
            if meta.len() == 0 {
                return false;
            }
        }
    }
    true
}
