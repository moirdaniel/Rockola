# Almacenamiento local (modo kiosko)

En modo kiosko la reproducción es **solo desde archivos locales**. YouTube se usa solo para buscar y descargar; no se hace stream remoto ni descarga on-the-fly al reproducir.

## Estructura bajo `MEDIA_ROOT`

Los archivos descargados (p. ej. con yt-dlp) se guardan en:

```
MEDIA_ROOT/
├── video/
│   └── <artist>/
│       └── <título>.mp4    (o <título> (1).mp4 si hay colisión)
└── audio/
    └── <artist>/
        └── <título>.mp3    (o <título> (1).mp3 si hay colisión)
```

- **artist** y **título** se obtienen de la metadata del video (yt-dlp: `uploader` y `title`) y se **sanitizan** para el sistema de archivos (caracteres no permitidos reemplazados, etc.).
- Si ya existe un archivo con el mismo artista y título, se añade un sufijo numérico: `(1)`, `(2)`, etc.

La ruta relativa (p. ej. `video/Artist Name/Song Title.mp4`) se guarda en `media_library.local_path` y se usa en `GET /api/media/stream?id=<lib_id>&source=local` para servir el archivo.

## Cola de descargas

Cuando el usuario añade a la cola un ítem de YouTube que **no está** en `media_library`:

1. Se crea un job en `downloads_queue` (estado `queued`).
2. Se añade el ítem a `play_queue` con `download_id` y sin `stream_id`.
3. Un worker en background procesa los jobs: descarga con yt-dlp a la ruta única, inserta en `media_library` y actualiza la fila de la cola con `stream_id` (URL local). Cuando el estado del job es `done`, el reproductor puede usar esa URL.

Ver también: [API](api.md) (stream, cola, descargas), [Panel de mantenedor](admin-panel.md).
