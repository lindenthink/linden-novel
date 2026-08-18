export interface GenerationParameters {
  // 期望章节字数（作为 prompt 软性引导，而非硬性的 token 上限）
  target_words?: number;
  temperature?: number;
  style?: string;
}

export interface AiGenerationHistory {
  id: string;
  chapter_id: string;
  mode: string;
  input_context: string;
  output_content: string;
  parameters_json: string;
  created_at: string;
}

export interface GenerateRequest {
  chapter_id: string;
  mode: 'continuation' | 'expansion' | 'rewrite' | 'polish' | 'outline';
  user_instruction?: string;
  parameters?: GenerationParameters;
}

export interface GenerateResponse {
  content: string;
  history: AiGenerationHistory;
}

// 流式生成事件 payload
export interface GenerationChunkEvent {
  content: string;
  done: boolean;
}

// 推理过程增量事件 payload（DeepSeek thinking 模式）
export interface ReasoningChunkEvent {
  reasoning: string;
  done: boolean;
}

export interface GenerationDoneEvent {
  content: string;
  history: AiGenerationHistory;
}
