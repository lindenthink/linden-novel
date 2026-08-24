// ---- Character (人物) ----
export interface Character {
  id: string;
  project_id: string;
  name: string;
  role: string | null;
  description: string | null;
  avatar: string | null;
  order_index: number;
  created_at: string;
  updated_at: string;
}

export interface CreateCharacter {
  project_id: string;
  name: string;
  role?: string | null;
  description?: string | null;
}

export interface UpdateCharacter {
  name?: string | null;
  role?: string | null;
  description?: string | null;
  avatar?: string | null;
}

// ---- Storyline (故事线) ----
export interface Storyline {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  status: "active" | "completed" | "abandoned";
  order_index: number;
  created_at: string;
  updated_at: string;
}

export interface CreateStoryline {
  project_id: string;
  name: string;
  description?: string | null;
}

export interface UpdateStoryline {
  name?: string | null;
  description?: string | null;
  status?: "active" | "completed" | "abandoned" | null;
}

// ---- Worldview (世界观) ----
export interface WorldviewEntry {
  id: string;
  project_id: string;
  name: string;
  category: string | null;
  description: string | null;
  order_index: number;
  created_at: string;
  updated_at: string;
}

export interface CreateWorldviewEntry {
  project_id: string;
  name: string;
  category?: string | null;
  description?: string | null;
}

export interface UpdateWorldviewEntry {
  name?: string | null;
  category?: string | null;
  description?: string | null;
}

// ---- Chapter Element (章节-元素关联) ----
export type ElementType = "character" | "storyline" | "worldview";

export interface ChapterElement {
  id: string;
  chapter_id: string;
  element_type: ElementType;
  element_id: string;
}

export interface CreateChapterElement {
  chapter_id: string;
  element_type: ElementType;
  element_id: string;
}

// ---- Foreshadow (伏笔) ----
export type ForeshadowImportance = "minor" | "normal" | "major";
export type ForeshadowStatus = "pending" | "planted" | "resolved" | "abandoned";

export interface Foreshadow {
  id: string;
  project_id: string;
  title: string;
  description: string | null;
  importance: ForeshadowImportance;
  status: ForeshadowStatus;
  plant_chapter_id: string | null;
  resolve_chapter_id: string | null;
  plant_note: string | null;
  resolve_note: string | null;
  order_index: number;
  created_at: string;
  updated_at: string;
}

export interface CreateForeshadow {
  project_id: string;
  title: string;
  description?: string | null;
  importance?: ForeshadowImportance;
  status?: ForeshadowStatus;
  plant_chapter_id?: string | null;
  resolve_chapter_id?: string | null;
  plant_note?: string | null;
  resolve_note?: string | null;
}

export interface UpdateForeshadow {
  title?: string | null;
  description?: string | null;
  importance?: ForeshadowImportance | null;
  status?: ForeshadowStatus | null;
  plant_chapter_id?: string | null;
  resolve_chapter_id?: string | null;
  plant_note?: string | null;
  resolve_note?: string | null;
}
