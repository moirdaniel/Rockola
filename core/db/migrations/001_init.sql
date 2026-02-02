PRAGMA foreign_keys = ON;

-- ==============
-- SOURCES
-- ==============
CREATE TABLE IF NOT EXISTS sources (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  root_path     TEXT NOT NULL UNIQUE,
  enabled       INTEGER NOT NULL DEFAULT 1,
  last_scan_at  INTEGER,                  -- unix epoch seconds
  last_seen_at  INTEGER,                  -- unix epoch seconds
  status        TEXT NOT NULL DEFAULT 'unknown' -- unknown|ok|unavailable
);

CREATE INDEX IF NOT EXISTS idx_sources_enabled ON sources(enabled);

-- ==============
-- ARTISTS (unificados por artist_key)
-- ==============
CREATE TABLE IF NOT EXISTS artists (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  display_name  TEXT NOT NULL,
  artist_key    TEXT NOT NULL UNIQUE,
  image_path    TEXT
);

CREATE INDEX IF NOT EXISTS idx_artists_display_name ON artists(display_name);

-- Alias opcional para merges manuales (futuro)
CREATE TABLE IF NOT EXISTS artist_aliases (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  alias_key   TEXT NOT NULL UNIQUE,
  artist_id   INTEGER NOT NULL,
  FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
);

-- ==============
-- MEDIA ITEMS (audio+video mezclados)
-- ==============
CREATE TABLE IF NOT EXISTS media_items (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  source_id      INTEGER NOT NULL,
  artist_id      INTEGER NOT NULL,

  full_path      TEXT NOT NULL UNIQUE,
  relative_path  TEXT NOT NULL,

  title          TEXT NOT NULL,
  media_type     TEXT NOT NULL DEFAULT 'unknown', -- audio|video|unknown

  duration_ms    INTEGER,
  size_bytes     INTEGER NOT NULL,
  mtime_unix     INTEGER NOT NULL,

  playable       TEXT NOT NULL DEFAULT 'unknown', -- unknown|playable|not_playable
  status         TEXT NOT NULL DEFAULT 'active',  -- active|missing

  created_at     INTEGER NOT NULL,
  updated_at     INTEGER NOT NULL,

  FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE,
  FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_media_items_artist_id ON media_items(artist_id);
CREATE INDEX IF NOT EXISTS idx_media_items_source_id ON media_items(source_id);
CREATE INDEX IF NOT EXISTS idx_media_items_title ON media_items(title);
CREATE INDEX IF NOT EXISTS idx_media_items_status ON media_items(status);
