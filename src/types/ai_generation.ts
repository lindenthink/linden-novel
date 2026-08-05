export interface GenerationParameters {
  max_tokens?: number;
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
