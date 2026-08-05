use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::content::{ChapterContent, SaveContent};

pub async fn get(pool: &SqlitePool, chapter_id: &str) -> Result<Option<ChapterContent>, sqlx::Error> {
    sqlx::query_as::<_, ChapterContent>("SELECT * FROM chapter_contents WHERE chapter_id = ?")
        .bind(chapter_id)
        .fetch_optional(pool)
        .await
}

/// 保存正文（INSERT OR REPLACE）：前端传 content_json + content_text，
/// Rust 从 content_text 计算 word_count（中文按字符 + 英文按词），返回权威 word_count。
pub async fn save(pool: &SqlitePool, input: &SaveContent) -> Result<i64, sqlx::Error> {
    let word_count = count_words(&input.content_text);
    let ts = pool::now();

    sqlx::query(
        "INSERT INTO chapter_contents (chapter_id, content_json, content_text, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(chapter_id) DO UPDATE SET
            content_json = excluded.content_json,
            content_text = excluded.content_text,
            updated_at = excluded.updated_at",
    )
    .bind(&input.chapter_id)
    .bind(&input.content_json)
    .bind(&input.content_text)
    .bind(&ts)
    .execute(pool)
    .await?;

    // 同步更新 chapters.word_count
    sqlx::query("UPDATE chapters SET word_count = ?, updated_at = ? WHERE id = ?")
        .bind(word_count)
        .bind(&ts)
        .bind(&input.chapter_id)
        .execute(pool)
        .await?;

    Ok(word_count)
}

/// 字数计算：中文按字符（仅汉字，不含标点）+ 英文按空白分词
fn count_words(text: &str) -> i64 {
    let mut count: i64 = 0;
    let mut in_english_word = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '\'' {
            if !in_english_word {
                count += 1;
                in_english_word = true;
            }
        } else if is_cjk_ideograph(ch) {
            count += 1;
            in_english_word = false;
        } else {
            in_english_word = false;
        }
    }
    count
}

/// 判断是否为 CJK 统一汉字（不含标点、符号）
fn is_cjk_ideograph(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'  | // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}'  | // CJK Extension A
        '\u{20000}'..='\u{2A6DF}'| // CJK Extension B
        '\u{2A700}'..='\u{2B73F}'| // CJK Extension C
        '\u{2B740}'..='\u{2B81F}'| // CJK Extension D
        '\u{2B820}'..='\u{2CEAF}'| // CJK Extension E
        '\u{F900}'..='\u{FAFF}'    // CJK Compatibility Ideographs
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_words_pure_chinese() {
        assert_eq!(count_words("你好世界"), 4);
    }

    #[test]
    fn test_count_words_pure_english() {
        assert_eq!(count_words("hello world"), 2);
    }

    #[test]
    fn test_count_words_mixed() {
        assert_eq!(count_words("你好hello世界"), 5); // 你(1)好(2) + hello(3) + 世(4)界(5)
    }

    #[test]
    fn test_count_words_with_punctuation() {
        assert_eq!(count_words("你好，世界！"), 4); // 逗号/感叹号不计
    }
}
