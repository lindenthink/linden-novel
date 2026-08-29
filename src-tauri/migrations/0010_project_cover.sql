-- 项目封面图片：存相对 app_data_dir 的相对路径（如 covers/{uuid}.png）
ALTER TABLE projects ADD COLUMN cover_path TEXT;
