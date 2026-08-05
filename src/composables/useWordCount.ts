import { computed, type Ref } from "vue";

/**
 * 前端实时字数统计 composable
 * 算法：统计文本中的中文字符 + 英文单词数
 * 权威值以保存时 Rust 端返回的 word_count 为准
 */
export function useWordCount(text: Ref<string>) {
  const wordCount = computed(() => {
    const content = text.value;
    if (!content || !content.trim()) return 0;

    // 统计中文字符
    const chineseChars = (content.match(/[\u4e00-\u9fff\u3400-\u4dbf]/g) || []).length;

    // 统计英文单词（去除中文后的空白分隔单词）
    const textWithoutChinese = content.replace(/[\u4e00-\u9fff\u3400-\u4dbf]/g, " ");
    const englishWords = (
      textWithoutChinese.match(/[a-zA-Z0-9]+(?:['-][a-zA-Z0-9]+)*/g) || []
    ).length;

    return chineseChars + englishWords;
  });

  return { wordCount };
}
