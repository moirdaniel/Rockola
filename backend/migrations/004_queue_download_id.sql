-- Vincula ítems de la cola con un job de descarga.
ALTER TABLE play_queue ADD COLUMN download_id TEXT;

CREATE INDEX IF NOT EXISTS idx_play_queue_download_id ON play_queue(download_id);
