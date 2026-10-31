//! FFmpeg 自动检测与下载。
//!
//! 应用启动时检查系统是否已有可用的 FFmpeg；若缺失，则自动从社区官方构建
//! （BtbN/FFmpeg-Builds）下载到软件目录（即可执行文件所在目录），保证封包合并可用。

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::utils::find_ffmpeg;

/// FFmpeg 社区官方构建（BtbN/FFmpeg-Builds）的 Windows x64 GPL 版，指向稳定"最新版"地址
const FFMPEG_WIN64_URL: &str =
    "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";

#[derive(Debug, thiserror::Error)]
pub enum FfmpegError {
    #[error("无法确定软件可执行文件所在目录")]
    NoSoftwareDir,
    #[error("下载 FFmpeg 失败: {0}")]
    Download(String),
    #[error("解压 FFmpeg 失败: {0}")]
    Extract(String),
    #[error("下载内容中未找到 ffmpeg.exe")]
    MissingBinary,
    #[error("写入文件失败: {0}")]
    Write(String),
}

pub type FfmpegResult<T> = Result<T, FfmpegError>;

/// 全局"本进程只尝试下载一次"守卫，避免并发触发重复下载
static DOWNLOAD_ATTEMPTED: OnceLock<()> = OnceLock::new();

/// 软件目录：可执行文件（便携版即 exe）所在目录
pub fn software_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

/// 软件目录中已下载的 ffmpeg.exe 路径
pub fn local_ffmpeg_path() -> Option<PathBuf> {
    software_dir().map(|d| d.join("ffmpeg.exe"))
}

/// 检查系统中是否存在可用的 FFmpeg（含软件目录中已下载的），存在则返回其路径
pub fn find_available_ffmpeg() -> Option<PathBuf> {
    if let Some(p) = find_ffmpeg() {
        return Some(p);
    }
    local_ffmpeg_path().filter(|p| p.is_file())
}

/// 确保 FFmpeg 可用：
/// 1. 系统 / 常见路径 / 软件目录中已存在 → 直接返回其路径；
/// 2. 否则自动下载 FFmpeg 到软件目录并返回路径。
pub async fn ensure_ffmpeg() -> FfmpegResult<PathBuf> {
    if let Some(p) = find_available_ffmpeg() {
        log::info!("[FFmpeg] 检测到可用 FFmpeg: {:?}", p);
        return Ok(p);
    }

    log::warn!("[FFmpeg] 未检测到 FFmpeg，开始自动下载到软件目录...");
    // 本进程内只尝试下载一次（若失败，重启应用可再次尝试）
    let _guard = DOWNLOAD_ATTEMPTED.get_or_init(|| ());

    // 再次检查，避免刚被其他流程下载完成
    if let Some(p) = find_available_ffmpeg() {
        return Ok(p);
    }

    let dest_dir = software_dir().ok_or(FfmpegError::NoSoftwareDir)?;
    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir).map_err(|e| FfmpegError::Write(e.to_string()))?;
    }

    let temp_zip = dest_dir.join("ffmpeg_download.zip");

    // 1) 下载
    let client = reqwest::Client::builder()
        .user_agent("AVDL/0.4.2")
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| FfmpegError::Download(e.to_string()))?;

    log::info!("[FFmpeg] 正在下载: {}", FFMPEG_WIN64_URL);
    let resp = client
        .get(FFMPEG_WIN64_URL)
        .send()
        .await
        .map_err(|e| FfmpegError::Download(format!("请求失败: {}", e)))?;

    if !resp.status().is_success() {
        return Err(FfmpegError::Download(format!("HTTP {}", resp.status())));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| FfmpegError::Download(format!("读取响应失败: {}", e)))?;
    tokio::fs::write(&temp_zip, &bytes)
        .await
        .map_err(|e| FfmpegError::Write(e.to_string()))?;
    log::info!(
        "[FFmpeg] 下载完成 ({} MB)，开始解压...",
        bytes.len() / (1024 * 1024)
    );

    // 2) 解压出 ffmpeg.exe（zip 解析为同步阻塞操作，放入 spawn_blocking 避免阻塞异步运行时）
    let dest_dir2 = dest_dir.clone();
    let temp_zip2 = temp_zip.clone();
    let result = tokio::task::spawn_blocking(move || extract_ffmpeg_exe(&temp_zip2, &dest_dir2))
        .await
        .map_err(|e| FfmpegError::Extract(format!("解压任务异常: {}", e)))??;

    let _ = tokio::fs::remove_file(&temp_zip).await;

    log::info!("[FFmpeg] FFmpeg 已就绪: {:?}", result);
    Ok(result)
}

/// 从下载的 zip 压缩包中提取 ffmpeg.exe 到软件目录
fn extract_ffmpeg_exe(zip_path: &Path, dest_dir: &Path) -> FfmpegResult<PathBuf> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| FfmpegError::Extract(format!("打开压缩包失败: {}", e)))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| FfmpegError::Extract(e.to_string()))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| FfmpegError::Extract(e.to_string()))?;
        let name = entry.name().to_lowercase();
        // BtbN 构建的 zip 结构为 <前缀>/bin/ffmpeg.exe，兼容根目录直接是 ffmpeg.exe 的情况
        if name.ends_with("bin/ffmpeg.exe")
            || name.ends_with("bin\\ffmpeg.exe")
            || name == "ffmpeg.exe"
        {
            let out_path = dest_dir.join("ffmpeg.exe");
            let mut out_file = std::fs::File::create(&out_path)
                .map_err(|e| FfmpegError::Write(e.to_string()))?;
            std::io::copy(&mut entry, &mut out_file)
                .map_err(|e| FfmpegError::Extract(e.to_string()))?;
            return Ok(out_path);
        }
    }

    Err(FfmpegError::MissingBinary)
}
