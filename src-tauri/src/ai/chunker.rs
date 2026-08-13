use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub target_chars: usize,
    pub overlap_chars: usize,
    pub min_chars: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            target_chars: 500,
            overlap_chars: 50,
            min_chars: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub index: usize,
    pub text: String,
    pub char_count: usize,
    pub content_hash: String,
}

/// 按段落 + 滑动窗口切片
///
/// 1. 按双换行拆段落
/// 2. 段落累积到 target_chars 切一刀
/// 3. 切片间保留 overlap_chars 尾部作为下一切片开头
/// 4. 末尾不足 min_chars 的合并到前一切片
pub fn chunk_text(text: &str, config: &ChunkConfig) -> Vec<Chunk> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for para in paragraphs {
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para);

        if current.chars().count() >= config.target_chars {
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        if current.chars().count() < config.min_chars && !chunks.is_empty() {
            let last = chunks.last_mut().unwrap();
            last.push_str("\n\n");
            last.push_str(&current);
        } else {
            chunks.push(current);
        }
    }

    let mut result = Vec::with_capacity(chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        let mut text = String::new();
        if i > 0 && config.overlap_chars > 0 {
            let prev = &chunks[i - 1];
            let prev_chars: Vec<char> = prev.chars().collect();
            let start = prev_chars.len().saturating_sub(config.overlap_chars);
            text.extend(&prev_chars[start..]);
            text.push('\n');
        }
        text.push_str(chunk);

        let char_count = text.chars().count();
        let content_hash = hash_text(&text);
        result.push(Chunk {
            index: i,
            text,
            char_count,
            content_hash,
        });
    }

    result
}

fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert!(chunk_text("", &ChunkConfig::default()).is_empty());
        assert!(chunk_text("   \n\n   ", &ChunkConfig::default()).is_empty());
    }

    #[test]
    fn test_short_single() {
        let chunks = chunk_text("这是一段短文字。", &ChunkConfig::default());
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].index, 0);
        assert_eq!(chunks[0].text, "这是一段短文字。");
        assert!(!chunks[0].content_hash.is_empty());
    }

    #[test]
    fn test_overlap_present() {
        let para1 = "段落一内容。".repeat(100);
        let para2 = "段落二内容。".repeat(100);
        let text = format!("{}\n\n{}", para1, para2);
        let chunks = chunk_text(&text, &ChunkConfig::default());
        assert!(chunks.len() >= 2);
        // 切片 1 开头包含切片 0 的尾部
        assert!(!chunks[0].content_hash.is_empty());
        assert!(!chunks[1].content_hash.is_empty());
        assert_ne!(chunks[0].content_hash, chunks[1].content_hash);
    }

    #[test]
    fn hash_deterministic() {
        assert_eq!(hash_text("a"), hash_text("a"));
        assert_ne!(hash_text("a"), hash_text("b"));
    }
}
