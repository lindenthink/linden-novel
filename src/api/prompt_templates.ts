import { invoke } from '@tauri-apps/api/core';
import type {
  PromptTemplate,
  CreatePromptTemplate,
  UpdatePromptTemplate,
} from '../types/ai';

export async function listPromptTemplates(): Promise<PromptTemplate[]> {
  return invoke<PromptTemplate[]>('list_prompt_templates');
}

export async function listPromptTemplatesByType(
  template_type: string
): Promise<PromptTemplate[]> {
  return invoke<PromptTemplate[]>('list_prompt_templates_by_type', { templateType: template_type });
}

export async function getPromptTemplate(id: string): Promise<PromptTemplate> {
  return invoke<PromptTemplate>('get_prompt_template', { id });
}

export async function createPromptTemplate(
  input: CreatePromptTemplate
): Promise<PromptTemplate> {
  return invoke<PromptTemplate>('create_prompt_template', { input });
}

export async function updatePromptTemplate(
  id: string,
  input: UpdatePromptTemplate
): Promise<PromptTemplate> {
  return invoke<PromptTemplate>('update_prompt_template', { id, input });
}

export async function deletePromptTemplate(id: string): Promise<void> {
  return invoke('delete_prompt_template', { id });
}

export async function resetPromptTemplateBuiltin(
  id: string
): Promise<PromptTemplate> {
  return invoke<PromptTemplate>('reset_prompt_template_builtin', { id });
}
