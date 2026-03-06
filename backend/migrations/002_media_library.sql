-- Biblioteca local: índex de archivos descargados (yt-dlp) y metadata.
-- source: 'local' | 'youtube'; local_path rellenado cuando el archivo existe.
CREATE TABLE IF NOT EXISTS media_library (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    artist TEXT,
    album TEXT,
    source TEXT NOT NULL DEFAULT 'local',
    local_path TEXT,
    duration_seconds INTEGER,
    thumbnail_url TEXT,
    media_type TEXT NOT NULL DEFAULT 'audio',
    external_id TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_media_library_title ON media_library(title);
CREATE INDEX IF NOT EXISTS idx_media_library_artist ON media_library(artist);
CREATE INDEX IF NOT EXISTS idx_media_library_external_id ON media_library(external_id);
