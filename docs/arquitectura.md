# Arquitectura - Rockola Digital

## Visión general

Rockola Digital es una jukebox web con frontend React (Vite, TypeScript) y backend Rust (Axum). La búsqueda unifica varias fuentes: **índice local (SQLite) primero**, luego **YouTube vía yt-dlp**. Los archivos descargados se guardan en `media_library` y en disco (`MEDIA_ROOT`).

## Stack

| Capa       | Tecnología                          |
|-----------|--------------------------------------|
| Frontend  | React 18, TypeScript, Vite, TailwindCSS, Zustand, React Query, React Router |
| Backend   | Rust, Axum, Tokio, SQLx              |
| Base de datos | SQLite                         |
| Descargas | yt-dlp (+ ffmpeg para audio)         |

## Estructura del monorepo

```
rockola-web/
├── frontend/          # SPA React
│   ├── src/
│   │   ├── adapters/   # MediaSourceAdapter (local, youtube, spotify mocks)
│   │   ├── components/
│   │   ├── pages/
│   │   ├── services/   # API client, unified search
│   │   ├── stores/     # Zustand
│   │   └── types/
│   └── docker/
├── backend/           # API Rust
│   ├── src/
│   │   ├── handlers/   # Axum handlers (search, queue, credits)
│   │   ├── repository/
│   │   ├── models/
│   │   ├── config/
│   │   └── services/
│   └── migrations/
├── docker/            # Dockerfiles y docker-compose
└── docs/
```

## Flujos principales

### Búsqueda unificada

- El **Unified Search Service** (frontend) consulta en paralelo todos los adapters registrados.
- Cada adapter implementa `MediaSourceAdapter`: `search(query)` y `getStreamUrl(id)`.
- El usuario ve un único listado; la fuente (local / youtube / spotify) es transparente.
- El backend expone `GET /api/search?q=` con un mock opcional; el frontend puede usar solo sus adapters.

### Cola de reproducción

- Cola FIFO persistida en SQLite (`play_queue`).
- Al agregar a la cola: se validan créditos, se deduce `COST_PER_SONG` y se inserta el item.
- `POST /api/queue/next` marca el primer item como reproducido y devuelve la cola actualizada.
- El reproductor (PlayerBar) reproduce la primera pista y al terminar llama a `next` y reproduce la siguiente.

### Créditos

- Un único usuario (`user_credits.id = 'default'`) con saldo.
- Costo por canción configurable (`COST_PER_SONG`).
- Validación en backend antes de insertar en cola; decremento atómico.

## Base de datos (SQLite)

- **media_library**: índice de archivos locales y descargados (id, title, artist, source, local_path, duration_seconds, external_id).
- **play_queue**: cola con `order`, `played_at` para FIFO y “siguiente”.
- **user_credits**: saldo y `updated_at`.
- **playlists**: tabla preparada para futuras fases.

## Estado global (Zustand)

- **playerStore**: item actual, isPlaying, currentTime, duration, volume, error.
- **queueStore**: items de la cola, loading, error.
- **creditsStore**: créditos, costPerSong.
- **searchStore**: query, resultados, isSearching.
- **uiStore**: vista actual, sidebar.

## Escalabilidad y siguientes fases

- Sustituir adapters mock por implementaciones reales (YouTube API, Spotify API, archivos locales).
- Añadir autenticación y múltiples usuarios.
- Stream de audio/video por proxy en el backend para fuentes externas.
- Playlists completas y favoritos.
- PWA y modo offline donde sea posible.
