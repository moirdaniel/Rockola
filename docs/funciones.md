# Guía de funciones - Rockola Digital

Este documento describe **qué hace cada función o módulo** principal del backend y del frontend para orientarse en el código.

---

## Backend (Rust)

### `main.rs` — Punto de entrada

| Función / bloque | Qué hace |
|------------------|----------|
| `main()` | Inicializa el logger (`tracing`), carga `.env`, crea `Config`, asegura directorios para DB y `MEDIA_ROOT`, conecta SQLite, ejecuta migraciones, llama a `ensure_default_credits`, monta la app con `create_app` y sirve en `0.0.0.0:PORT`. |

### `lib.rs` — App y rutas

| Función / tipo | Qué hace |
|----------------|----------|
| `AppState` | Estado compartido: `pool` (SQLite) y `config`. |
| `create_app(state)` | Construye el `Router` de Axum con todas las rutas (`/api/search`, `/api/media/stream`, `/api/queue`, `/api/credits`, `/api/admin/reset`, `/api/maintenance`, `/health`) y CORS abierto. |

### Handlers (peticiones HTTP)

#### `handlers/search.rs`

| Función | Ruta | Qué hace |
|---------|------|----------|
| `search()` | `GET /api/search?q=` | Si `q` está vacío devuelve `{ artists: [], songs: [] }`. Si no, llama al servicio de búsqueda (índice local + YouTube vía yt-dlp) y devuelve el resultado o 502 con mensaje de error. |

#### `handlers/media.rs`

| Función | Ruta | Qué hace |
|---------|------|----------|
| `stream()` | `GET /api/media/stream?id=&source=` | Sirve el archivo desde la biblioteca local. Con `source=youtube` solo sirve si ya está en `media_library`; si no, 404 (modo kiosko, sin descarga on-the-fly). |

#### `handlers/queue.rs`

| Función | Ruta | Qué hace |
|---------|------|----------|
| `get_queue()` | `GET /api/queue` | Devuelve la cola de reproducción (solo ítems no reproducidos, orden FIFO). |
| `add_to_queue()` | `POST /api/queue` | Descuenta créditos; si no hay saldo devuelve 402. Si el ítem es YouTube y ya está en biblioteca, encola con stream local; si no, crea job de descarga, encola con `download_id` y el worker descarga en background. Devuelve la cola actualizada. |
| `next()` | `POST /api/queue/next` | Marca el primer ítem como reproducido y devuelve la cola sin ese ítem. |
| `clear_queue()` | `DELETE /api/queue` | Vacía la cola (solo no reproducidos). |

#### `handlers/credits.rs`

| Función | Ruta | Qué hace |
|---------|------|----------|
| `get_credits()` | `GET /api/credits` | Devuelve saldo del usuario `default`. |
| `add_credits()` | `POST /api/credits/add` | Suma créditos (body: `{ amount }`). |

#### `handlers/admin.rs`

| Función | Ruta | Qué hace |
|---------|------|----------|
| `login()` | `POST /api/admin/login` | Valida PIN (`ADMIN_PIN`); si es correcto crea sesión (15 min) y devuelve token. Rate limit: 5 fallos → bloqueo 5 min. |
| `logout()` | `POST /api/admin/logout` | Invalida la sesión (header `Authorization: Bearer <token>`). |
| `session_status()` | `GET /api/admin/session` | Comprueba si el token es válido. |
| `reset()` | `POST /api/admin/reset` | Requiere sesión si `ADMIN_PIN` está definido. Borra cola, caché y biblioteca; resetea créditos; registra en audit log. |
| `maintenance()` | `GET /api/maintenance` | Requiere sesión si `ADMIN_PIN` está definido. Devuelve cola, biblioteca reciente, créditos y conteos. |
| `audit_log()` | `GET /api/admin/audit-log` | Requiere sesión. Devuelve últimas entradas del registro de auditoría. |

#### `handlers/downloads.rs`

| Función | Ruta | Qué hace |
|---------|------|----------|
| `list_downloads()` | `GET /api/downloads` | Requiere sesión si `ADMIN_PIN` está definido. Lista jobs de la cola de descargas. |
| `retry_download()` | `POST /api/downloads/:id/retry` | Requiere sesión. Pone un job `failed` de nuevo en `queued` y lanza el worker. |

### `repository.rs` — Acceso a datos

| Función | Qué hace |
|---------|----------|
| `get_queue()` | Lista ítems de `play_queue` no reproducidos, ordenados por `order`. |
| `add_to_queue()` | Inserta en `play_queue` y devuelve la cola actualizada. |
| `mark_next_played()` | Marca el primer ítem como reproducido y devuelve la cola restante. |
| `clear_queue()` | Elimina ítems no reproducidos de `play_queue`. |
| `reset_all()` | Vacía cola, tabla de caché y `media_library`; resetea créditos del usuario `default`. |
| `get_credits()` | Obtiene la fila de créditos del usuario `default`. |
| `ensure_default_credits()` | Inserta la fila `default` en `user_credits` si no existe (evita 500 en créditos/cola/maintenance). |
| `add_credits()` | Suma `amount` al saldo del usuario `default`. |
| `deduct_credits()` | Resta créditos; devuelve `false` si no hay saldo suficiente. |
| `search_media_library()` | Busca en `media_library` por título/artista (LIKE). |
| `get_media_by_id()` | Busca en `media_library` por `id`. |
| `get_media_by_external_id()` | Busca por `external_id` (p. ej. ID de YouTube). |
| `insert_media_library()` | Inserta o actualiza un ítem en `media_library`. |
| `list_media_library_recent()` | Lista los últimos `limit` ítems de la biblioteca. |
| `get_maintenance_counts()` | Devuelve (queueCount, mediaLibraryCount, mediaCacheCount). |

### Servicios

#### `services/search.rs`

| Función | Qué hace |
|---------|----------|
| `search()` | Busca primero en `media_library`; si no hay resultados, llama a `yt_dlp::search_youtube`. Convierte filas a `MediaItem` y devuelve `{ artists, songs }`. |

#### `services/yt_dlp.rs`

| Función | Qué hace |
|---------|----------|
| `search_youtube()` | Ejecuta yt-dlp para buscar en YouTube; parsea salida y devuelve artistas y canciones (`MediaItem`). |
| `download_audio()` | Descarga audio con yt-dlp a `MEDIA_ROOT/audio/youtube/`. |
| `download_video()` | Descarga video (mp4) a `MEDIA_ROOT/video/youtube/<id>/`. |
| `youtube_url_from_id()` | Devuelve la URL de YouTube para un ID dado. |

### `config.rs`

| Función / tipo | Qué hace |
|----------------|----------|
| `Config::from_env()` | Carga configuración desde variables de entorno (DATABASE_URL, PORT, MEDIA_ROOT, COST_PER_SONG, yt_dlp path). |
| `default_database_path()` | Ruta por defecto de SQLite si no se define DATABASE_URL. |

---

## Frontend (React / TypeScript)

### Adaptadores (`adapters/`)

| Función / módulo | Qué hace |
|------------------|----------|
| `getAdapters()` | Devuelve la lista de adapters de búsqueda (en producción solo `backendAdapter`). |
| `getAdapterBySourceId(sourceId)` | Devuelve el adapter con ese `sourceId`; si es `youtube` o `local` y no está en la lista, devuelve `backendAdapter` (para que la cola siempre pueda obtener la URL de stream del backend). |
| `backendAdapter` | Implementa `MediaSourceAdapter`: `search()` llama a `GET /api/search`, `getStreamUrl()` construye la URL de `GET /api/media/stream?id=&source=`. |
| `mockLocalAdapter`, `mockYoutubeAdapter`, `mockSpotifyAdapter` | Mocks opcionales para desarrollo; no se usan si solo está registrado `backendAdapter`. |

### Servicios

#### `services/api.ts`

| Función | Qué hace |
|---------|----------|
| `getBaseUrl()` | Devuelve la base URL del API (variable de entorno o mismo origen para proxy). |
| `api.get/post/delete()` | Cliente HTTP para llamar al backend; maneja errores y devuelve JSON. |

#### `services/queueApi.ts`

| Objeto / método | Qué hace |
|-----------------|----------|
| `queueApi.getQueue()` | GET `/api/queue` → devuelve el array `queue`. |
| `queueApi.add(payload)` | POST `/api/queue` con `mediaItem`; devuelve cola actualizada. |
| `queueApi.next()` | POST `/api/queue/next`; devuelve cola actualizada. |
| `queueApi.clear()` | DELETE `/api/queue`. |
| `creditsApi.get()` | GET `/api/credits`. |
| `creditsApi.add(amount)` | POST `/api/credits/add`. |
| `adminApi.resetData()` | POST `/api/admin/reset`. |
| `adminApi.getMaintenance()` | GET `/api/maintenance` (cola, biblioteca reciente, créditos, stats). |

#### `services/unifiedSearch.ts`

| Función | Qué hace |
|---------|----------|
| `searchMedia(query)` | Llama a `search()` de todos los adapters en paralelo, combina ítems y artistas y devuelve `{ items, artists, query, sourcesQueried }`. Si la query está vacía, devuelve listas vacías. |

### Stores (Zustand)

| Store | Qué guarda / hace |
|-------|-------------------|
| `playerStore` | Item actual, isPlaying, currentTime, duration, volume, error; métodos para play/pause, seek, cargar ítem, etc. |
| `queueStore` | Lista de ítems de la cola, loading, error; carga desde `queueApi.getQueue()`, add/next/clear vía API. |
| `creditsStore` | Créditos y costPerSong; carga con `creditsApi.get()`, add con `creditsApi.add()`. |
| `searchStore` | query, resultados (items/artists), isSearching; usa `searchMedia()`. |
| `errorStore` | Mensaje de error global para mostrar en `ErrorModal`. |

### Páginas y componentes principales

| Página / componente | Qué hace |
|---------------------|----------|
| `Home` | Búsqueda unificada (usa searchStore y unifiedSearch), lista de resultados y botón “Añadir a la cola”. |
| `Queue` | Muestra la cola (queueStore), botones Siguiente y Vaciar cola. |
| `Credits` | Muestra saldo (creditsStore) y formulario para añadir créditos. |
| `Admin` | Añadir créditos y (opcional) enlace o acciones de administración. |
| `Mantenedores` | Llama a `adminApi.getMaintenance()`, muestra cola, biblioteca reciente, créditos y estadísticas; botones Actualizar y “Borrar data y resetear créditos”. |
| `NowPlaying` / `TvNowPlaying` | Vista de “ahora suena” a pantalla completa; usan el ítem actual del reproductor y la URL de stream (vía adapter por `source`). |
| `Layout` | Estructura común: navegación y outlet para rutas. |
| `PlayerBar` | Reproductor en barra inferior: usa `playerStore`, obtiene URL de stream con `getAdapterBySourceId(item.source)?.getStreamUrl(item)`, priorizando `item.streamId`; al terminar llama a `queueApi.next()` y reproduce el siguiente. |
| `MediaCard` | Tarjeta de un ítem con título/artista y acción “Añadir a la cola” o reproducir. |
| `ErrorModal` | Muestra el mensaje de `errorStore` y permite cerrarlo. |
| `PlayerRefsProvider` / `usePlayerRefs` | Contexto para referencias al reproductor (audio/video) usadas por PlayerBar y páginas de now-playing. |

---

## Resumen de flujos

1. **Búsqueda:** Usuario escribe en Home → `searchMedia()` → adapters (backend) → `GET /api/search` → backend busca en `media_library` y/o YouTube → resultados en searchStore.
2. **Añadir a la cola:** Usuario pulsa “Añadir” en un ítem → `queueApi.add({ mediaItem })` → backend deduce créditos e inserta en `play_queue` → queueStore se actualiza.
3. **Reproducir:** PlayerBar toma el primer ítem de la cola, obtiene URL con `getAdapterBySourceId(source).getStreamUrl(item)` (streamId o id/source), reproduce; al terminar llama `queueApi.next()` y reproduce el siguiente.
4. **Mantenedores:** Página Mantenedores → `adminApi.getMaintenance()` → `GET /api/maintenance` → backend devuelve cola, biblioteca, créditos y stats. Si `ADMIN_PIN` está definido, antes se hace login (`POST /api/admin/login`) y el token se envía en las peticiones.

Con esta guía puedes localizar rápidamente qué función o módulo hace cada cosa en backend y frontend.

**Documentación adicional:** [API](api.md), [Panel de mantenedor (admin)](admin-panel.md), [Almacenamiento local](storage-layout.md), [App escritorio / kiosko](desktop-kiosk.md).
