use std::path::Path;

use crate::error::AppError;

/// HuggingFace 模型仓库（bge-small-zh-v1.5，512 维，~95MB，专为中文优化）
const MODEL_REPO: &str = "BAAI/bge-small-zh-v1.5";

/// hypembed 必需的三个文件
const REQUIRED_FILES: &[&str] = &["config.json", "vocab.txt", "model.safetensors"];

/// 确保模型文件已下载到指定目录
///
/// 逐个检查文件是否存在，缺失的自动从 HuggingFace 下载。
/// 已存在的文件跳过（支持断点续传：删掉损坏文件重新运行即可）。
pub async fn ensure_model(model_dir: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(model_dir)
        .map_err(|e| AppError::Internal(format!("创建模型目录失败: {}", e)))?;

    for filename in REQUIRED_FILES {
        let filepath = model_dir.join(filename);
        if filepath.exists() {
            tracing::debug!("模型文件已存在: {}", filename);
            continue;
        }

        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            MODEL_REPO, filename
        );
        tracing::info!("正在下载模型文件: {}", filename);

        download_file(&url, &filepath).await?;
        let size = std::fs::metadata(&filepath)
            .map(|m| m.len())
            .unwrap_or(0);
        tracing::info!(
            "下载完成: {} ({:.1} MB)",
            filename,
            size as f64 / 1_048_576.0
        );
    }

    tracing::info!("嵌入模型就绪: {:?}", model_dir);
    Ok(())
}

/// 检查模型是否已就绪（所有必需文件都存在）
pub fn is_model_ready(model_dir: &Path) -> bool {
    REQUIRED_FILES
        .iter()
        .all(|f| model_dir.join(f).exists())
}

/// 下载单个文件到指定路径（先写 .tmp 再重命名，避免半成品）
async fn download_file(url: &str, dest: &Path) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| AppError::Internal(format!("创建 HTTP 客户端失败: {}", e)))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("下载请求失败: {} - {}", url, e)))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "下载失败: {} HTTP {}",
            url,
            resp.status()
        )));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("读取响应内容失败: {}", e)))?;

    // 先写临时文件，再原子重命名
    let tmp = dest.with_extension("tmp");
    std::fs::write(&tmp, &bytes)
        .map_err(|e| AppError::Internal(format!("写入文件失败: {:?} - {}", tmp, e)))?;
    std::fs::rename(&tmp, dest)
        .map_err(|e| AppError::Internal(format!("重命名文件失败: {:?} - {}", dest, e)))?;

    Ok(())
}
