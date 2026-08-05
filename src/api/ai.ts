import { invoke } from "@tauri-apps/api/core";
import type {
  AiProvider,
  CreateAiProvider,
  UpdateAiProvider,
  AiApiKey,
  CreateAiApiKey,
  PromptTemplate,
  CreatePromptTemplate,
  UpdatePromptTemplate,
  CompleteRequest,
  CompleteResponse,
} from "../types/ai";

// ---- AI Provider ----

export async function listAiProviders(): Promise<AiProvider[]> {
  return invoke<AiProvider[]>("list_ai_providers");
}

export async function getAiProvider(id: string): Promise<AiProvider> {
  return invoke<AiProvider>("get_ai_provider", { id });
}

export async function createAiProvider(input: CreateAiProvider): Promise<AiProvider> {
  return invoke<AiProvider>("create_ai_provider", { input });
}

export async function updateAiProvider(id: string, input: UpdateAiProvider): Promise<AiProvider> {
  return invoke<AiProvider>("update_ai_provider", { id, input });
}

export async function deleteAiProvider(id: string): Promise<void> {
  return invoke("delete_ai_provider", { id });
}

export async function getDefaultAiProvider(): Promise<AiProvider | null> {
  return invoke<AiProvider | null>("get_default_ai_provider");
}

// ---- AI API Key ----

export async function listAiApiKeys(providerId: string): Promise<AiApiKey[]> {
  return invoke<AiApiKey[]>("list_ai_api_keys", { providerId });
}

export async function createAiApiKey(input: CreateAiApiKey): Promise<AiApiKey> {
  return invoke<AiApiKey>("create_ai_api_key", { input });
}

export async function deleteAiApiKey(id: string): Promise<void> {
  return invoke("delete_ai_api_key", { id });
}

export async function setDefaultAiApiKey(id: string): Promise<void> {
  return invoke("set_default_ai_api_key", { id });
}

// ---- Prompt Template ----

export async function listPromptTemplates(): Promise<PromptTemplate[]> {
  return invoke<PromptTemplate[]>("list_prompt_templates");
}

export async function getPromptTemplate(id: string): Promise<PromptTemplate> {
  return invoke<PromptTemplate>("get_prompt_template", { id });
}

export async function listPromptTemplatesByType(templateType: string): Promise<PromptTemplate[]> {
  return invoke<PromptTemplate[]>("list_prompt_templates_by_type", { templateType });
}

export async function createPromptTemplate(input: CreatePromptTemplate): Promise<PromptTemplate> {
  return invoke<PromptTemplate>("create_prompt_template", { input });
}

export async function updatePromptTemplate(id: string, input: UpdatePromptTemplate): Promise<PromptTemplate> {
  return invoke<PromptTemplate>("update_prompt_template", { id, input });
}

export async function deletePromptTemplate(id: string): Promise<void> {
  return invoke("delete_prompt_template", { id });
}

// ---- AI Completion ----

export async function aiComplete(request: CompleteRequest): Promise<CompleteResponse> {
  return invoke<CompleteResponse>("ai_complete", { request });
}

export async function aiCompleteStream(request: CompleteRequest): Promise<void> {
  return invoke("ai_complete_stream", { request });
}

export async function aiRenderTemplate(
  templateId: string,
  variables: Record<string, string>
): Promise<string> {
  return invoke<string>("ai_render_template", { templateId, variables });
}
