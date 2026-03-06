-- media_cache: caché de items de media (opcional para búsqueda server-side)
CREATE TABLE IF NOT EXISTS media_cache (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    title TEXT NOT NULL,
    artist TEXT,
    album TEXT,
    duration_seconds INTEGER NOT NULL,
    thumbnail_url TEXT,
    type TEXT NOT NULL,
    stream_id TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- play_queue: cola FIFO persistente
CREATE TABLE IF NOT EXISTS play_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    queue_id TEXT NOT NULL UNIQUE,
    media_id TEXT NOT NULL,
    source TEXT NOT NULL,
    title TEXT NOT NULL,
    artist TEXT,
    album TEXT,
    duration_seconds INTEGER NOT NULL,
    thumbnail_url TEXT,
    type TEXT NOT NULL,
    stream_id TEXT,
    "order" INTEGER NOT NULL,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    played_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_play_queue_order ON play_queue("order");

-- user_credits: saldo de créditos (un usuario por ahora)
CREATE TABLE IF NOT EXISTS user_credits (
    id TEXT PRIMARY KEY,
    balance INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- playlists: listas de reproducción (estructura para futuras fases)
CREATE TABLE IF NOT EXISTS playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Seed créditos iniciales
INSERT OR IGNORE INTO user_credits (id, balance, updated_at) VALUES ('default', 1000, datetime('now'));
