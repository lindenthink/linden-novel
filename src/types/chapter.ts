// ---- Chapter ----
export interface Chapter {
  id: string;
  volume_id: string;
  project_id: string;
  title: string;
  order_index: number;
  status: "draft" | "writing" | "final";
  word_count: number;
  summary: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateChapter {
  volume_id: string;
  project_id: string;
  title: string;
}

export interface UpdateChapterMeta {
  title?: string | null;
  status?: "draft" | "writing" | "final" | null;
  summary?: string | null;
}

// ---- Chapter Content ----
export interface ChapterContent {
  chapter_id: string;
  content_json: string;
  content_text: string;
  updated_at: string;
}
