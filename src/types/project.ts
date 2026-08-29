// ---- Project ----
export interface Project {
  id: string;
  title: string;
  genre: string | null;
  summary: string | null;
  target_words: number | null;
  settings_json: string | null;
  cover_path: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateProject {
  title: string;
  genre?: string | null;
  summary?: string | null;
  target_words?: number | null;
  cover_path?: string | null;
}

export interface UpdateProject {
  title?: string | null;
  genre?: string | null;
  summary?: string | null;
  target_words?: number | null;
  settings_json?: string | null;
  cover_path?: string | null;
}

// ---- Volume ----
export interface Volume {
  id: string;
  project_id: string;
  title: string;
  order_index: number;
  created_at: string;
  updated_at: string;
}

export interface CreateVolume {
  project_id: string;
  title: string;
}

export interface UpdateVolume {
  title?: string | null;
}
