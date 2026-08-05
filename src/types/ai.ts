// ---- AI Provider ----
export interface AiProvider {
  id: string;
  name: string;
  provider_type: string;
  base_url: string;
  models_json: string;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateAiProvider {
  name: string;
  provider_type: string;
  base_url: string;
  models_json: string;
  is_default?: boolean;
}

export interface UpdateAiProvider {
  name?: string;
  provider_type?: string;
  base_url?: string;
  models_json?: string;
  is_default?: boolean;
}

// ---- AI API Key ----
export interface AiApiKey {
  id: string;
  provider_id: string;
  name: string;
  encrypted_key: string;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateAiApiKey {
  provider_id: string;
  name: string;
  api_key: string;
  is_default?: boolean;
}

// ---- Prompt Template ----
export interface PromptTemplate {
  id: string;
  name: string;
  template_type: string;
  content: string;
  variables_json: string;
  created_at: string;
  updated_at: string;
}

export interface CreatePromptTemplate {
  name: string;
  template_type: string;
  content: string;
  variables_json: string;
}

export interface UpdatePromptTemplate {
  name?: string;
  template_type?: string;
  content?: string;
  variables_json?: string;
}

// ---- AI Completion ----
export interface Message {
  role: string;
  content: string;
}

export interface CompleteRequest {
  provider_id?: string;
  api_key_id?: string;
  model: string;
  messages: Message[];
  temperature?: number;
  max_tokens?: number;
  stream?: boolean;
}

export interface CompleteResponse {
  content: string;
  model: string;
  usage?: UsageInfo;
}

export interface UsageInfo {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface StreamChunkEvent {
  content: string;
  done: boolean;
}
