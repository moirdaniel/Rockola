-- downloads_queue: cola de descargas locales desde YouTube.
CREATE TABLE IF NOT EXISTS downloads_queue (
    id TEXT PRIMARY KEY,
    youtube_video_id TEXT NOT NULL,
    requested_media_type TEXT NOT NULL CHECK (requested_media_type IN ('audio', 'video')),
    status TEXT NOT NULL CHECK (status IN ('queued', 'downloading', 'done', 'failed')),
    progress REAL,
    target_path TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_downloads_queue_status ON downloads_queue(status);
CREATE INDEX IF NOT EXISTS idx_downloads_queue_youtube ON downloads_queue(youtube_video_id);

-- admin_audit_log: registro de acciones del mantenedor/admin.
CREATE TABLE IF NOT EXISTS admin_audit_log (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    payload_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_admin_audit_action ON admin_audit_log(action);
CREATE INDEX IF NOT EXISTS idx_admin_audit_entity ON admin_audit_log(entity_type, entity_id);

-- settings: configuración global de la app (display, rutas, etc.).
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Valores por defecto básicos.
INSERT OR IGNORE INTO settings (key, value) VALUES
  ('displayMode', 'single'),
  ('displayFullscreenOnIdleSeconds', '5'),
  ('mediaRootPath', 'data/media'),
  ('downloadMode', 'video');

