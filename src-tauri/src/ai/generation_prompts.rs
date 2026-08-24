use crate::models::ai_generation::GenerationContext;

pub fn build_generation_prompt(
    context: &GenerationContext,
    mode: &str,
    user_instruction: Option<&str>,
    target_words: Option<i32>,
    narrative_rules: &str,
) -> String {
    let mut prompt = String::new();

    // 系统提示
    prompt.push_str("你是一位专业的小说创作助手。请根据以下信息生成高质量的文本内容。\n\n");

    // 章节信息
    prompt.push_str(&format!("## 章节信息\n"));
    prompt.push_str(&format!("- 章节标题：{}\n", context.chapter_title));
    if let Some(summary) = &context.chapter_summary {
        prompt.push_str(&format!("- 章节摘要：{}\n", summary));
    }
    prompt.push('\n');

    // 相邻章节摘要（情节连贯性）
    if context.previous_chapter_summary.is_some() || context.next_chapter_summary.is_some() {
        prompt.push_str("## 相邻章节摘要\n");
        if let Some(prev) = &context.previous_chapter_summary {
            prompt.push_str(&format!("- 前一章：{}\n", prev));
        }
        if let Some(next) = &context.next_chapter_summary {
            prompt.push_str(&format!("- 后一章：{}\n", next));
        }
        prompt.push('\n');
    }

    // 角色信息
    if !context.characters.is_empty() {
        prompt.push_str("## 关联角色\n");
        for character in &context.characters {
            prompt.push_str(&format!("- {}", character.name));
            if let Some(personality) = &character.personality {
                prompt.push_str(&format!("（角色：{}）", personality));
            }
            if let Some(desc) = &character.description {
                prompt.push_str(&format!("：{}", desc));
            }
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    // 情节线索
    if !context.storylines.is_empty() {
        prompt.push_str("## 情节线索\n");
        for storyline in &context.storylines {
            prompt.push_str(&format!("- {}", storyline.title));
            if let Some(desc) = &storyline.description {
                prompt.push_str(&format!("：{}", desc));
            }
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    // 世界观设定
    if !context.worldviews.is_empty() {
        prompt.push_str("## 世界观设定\n");
        for worldview in &context.worldviews {
            prompt.push_str(&format!("- {}", worldview.name));
            if let Some(desc) = &worldview.description {
                prompt.push_str(&format!("：{}", desc));
            }
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    // 伏笔：本章需埋下的
    if !context.foreshadows_to_plant.is_empty() {
        prompt.push_str("## 本章需埋下的伏笔\n");
        prompt.push_str("请在正文中自然埋下以下伏笔，不要刻意点明：\n");
        for f in &context.foreshadows_to_plant {
            prompt.push_str(&format!("- 「{}」", f.title));
            if let Some(desc) = &f.description {
                prompt.push_str(&format!("：{}", desc));
            }
            if let Some(note) = &f.plant_note {
                prompt.push_str(&format!("（埋点说明：{}）", note));
            }
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    // 伏笔：本章可回收的
    if !context.foreshadows_to_resolve.is_empty() {
        prompt.push_str("## 本章可回收的伏笔\n");
        prompt.push_str("以下伏笔已埋下，若情节合适可在此章回收：\n");
        for f in &context.foreshadows_to_resolve {
            prompt.push_str(&format!("- 「{}」", f.title));
            if let Some(desc) = &f.description {
                prompt.push_str(&format!("：{}", desc));
            }
            if let Some(note) = &f.resolve_note {
                prompt.push_str(&format!("（回收说明：{}）", note));
            }
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    // 当前内容
    if !context.chapter_content.is_empty() {
        prompt.push_str("## 当前内容\n");
        prompt.push_str(&context.chapter_content);
        prompt.push_str("\n\n");
    }

    // RAG 检索到的相关上下文
    if let Some(rag) = &context.rag_context {
        if !rag.trim().is_empty() {
            prompt.push_str(rag);
        }
    }

    // 生成要求
    prompt.push_str("## 创作指令\n");
    match mode {
        "continuation" => {
            prompt.push_str("- 请继续续写以上内容，保持风格和情节的连贯性。\n");
        }
        "expansion" => {
            prompt.push_str("- 请扩写以上内容，增加细节描写和情节发展。\n");
        }
        "rewrite" => {
            prompt.push_str("- 请改写以上内容，使其更加生动、流畅。\n");
        }
        "polish" => {
            prompt.push_str("- 请润色以上内容，优化语言表达和文学性。\n");
        }
        "outline" => {
            prompt.push_str("- 请为以上内容生成详细的章节大纲。\n");
        }
        _ => {
            prompt.push_str("- 请根据上下文生成合适的内容。\n");    
        }
    }
    prompt.push('\n');

      // 用户补充指令
    if let Some(instruction) = user_instruction {
        if !instruction.trim().is_empty() {
            prompt.push_str(&format!("- 用户补充指令：{}\n", instruction));
        }
    }

    // 叙事规则：已由调用方按「约束程度」从 prompt_templates 取到正文（找不到时会回退编译期默认）。
    // trim_end + 显式换行：归一化末尾换行，避免编辑器/文本框自动追加空行导致 prompt 多空行
    prompt.push_str(narrative_rules.trim_end());
    prompt.push_str("\n\n");

    prompt.push_str("## 输出要求\n");
    prompt.push_str("- 直接输出小说正文，不要包含标题、解释或标记。\n");
        // 期望字数（作为软性引导，避免硬 token 限制在推理模型上耗尽 thinking 额度）
    if let Some(n) = target_words {
        if n > 0 {
            prompt.push_str(&format!("- 字数控制在 {} 字左右，无需严格相等，情节自然即可。\n", n));
        }
    }
    
    prompt
}
