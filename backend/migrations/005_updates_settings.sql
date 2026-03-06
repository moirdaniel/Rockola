-- Valores por defecto para configuración de actualizaciones (solo admin).
INSERT OR IGNORE INTO settings (key, value) VALUES
  ('updates.enabled', 'true'),
  ('updates.channel', 'stable'),
  ('updates.autoCheck', 'true'),
  ('updates.checkIntervalMinutes', '720'),
  ('updates.endpointOverride', '');
