-- Tabla de fuentes de medios
CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_path TEXT NOT NULL UNIQUE,
    enabled BOOLEAN DEFAULT 1,
    status TEXT DEFAULT 'ok',
    last_scan_at INTEGER,
    last_seen_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Tabla de artistas
CREATE TABLE IF NOT EXISTS artists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    display_name TEXT NOT NULL,
    artist_key TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Índice para búsquedas rápidas por nombre de artista
CREATE INDEX IF NOT EXISTS idx_artists_display_name ON artists(display_name COLLATE NOCASE);

-- Tabla de elementos multimedia
CREATE TABLE IF NOT EXISTS media_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    artist_id INTEGER NOT NULL,
    full_path TEXT NOT NULL UNIQUE,
    relative_path TEXT NOT NULL,
    title TEXT NOT NULL,
    media_type TEXT NOT NULL, -- 'audio', 'video', 'unknown'
    duration_ms INTEGER,
    size_bytes INTEGER NOT NULL,
    mtime_unix INTEGER NOT NULL,
    playable TEXT DEFAULT 'unknown', -- 'unknown', 'playable', 'not_playable'
    status TEXT DEFAULT 'active', -- 'active', 'missing'
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE,
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
);

-- Índices para búsquedas rápidas
CREATE INDEX IF NOT EXISTS idx_media_items_artist_id ON media_items(artist_id);
CREATE INDEX IF NOT EXISTS idx_media_items_media_type ON media_items(media_type);
CREATE INDEX IF NOT EXISTS idx_media_items_status ON media_items(status);
CREATE INDEX IF NOT EXISTS idx_media_items_title ON media_items(title COLLATE NOCASE);

-- Trigger para actualizar la fecha de modificación
CREATE TRIGGER IF NOT EXISTS update_media_items_updated_at 
AFTER UPDATE ON media_items
BEGIN
    UPDATE media_items SET updated_at = (unixepoch()) WHERE id = NEW.id;
END;

-- Tabla de colas de reproducción (para futuras funcionalidades)
CREATE TABLE IF NOT EXISTS queues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Tabla de elementos en cola
CREATE TABLE IF NOT EXISTS queue_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    queue_id INTEGER NOT NULL,
    media_item_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    added_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (queue_id) REFERENCES queues(id) ON DELETE CASCADE,
    FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE
);

-- Índice para la cola
CREATE INDEX IF NOT EXISTS idx_queue_items_queue_pos ON queue_items(queue_id, position);

-- Tabla de reproducciones recientes (historial)
CREATE TABLE IF NOT EXISTS playback_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    media_item_id INTEGER NOT NULL,
    played_at INTEGER NOT NULL DEFAULT (unixepoch()),
    duration_played INTEGER, -- cuánto se reprodujo
    completed BOOLEAN DEFAULT 0, -- si se completó la reproducción
    FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE
);

-- Índice para historial
CREATE INDEX IF NOT EXISTS idx_playback_history_played_at ON playback_history(played_at DESC);