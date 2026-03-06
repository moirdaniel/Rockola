# API - Rockola Digital

Base URL (local): `http://localhost:3000`

Respuestas en JSON. Errores devuelven cuerpo opcional y código HTTP apropiado.

---

## Búsqueda

### `GET /api/search`

Búsqueda unificada: primero en índice local (`media_library`); si no hay resultados, en YouTube con yt-dlp. Requiere `yt-dlp` instalado para resultados de YouTube.

**Query**

| Parámetro | Tipo   | Descripción   |
|----------|--------|----------------|
| q        | string | Término a buscar |

**Respuesta:** `200 OK` — array de `MediaItem`

```json
[
  {
    "id": "string",
    "source": "string",
    "title": "string",
    "artist": "string | null",
    "album": "string | null",
    "durationSeconds": 0,
    "thumbnailUrl": "string | null",
    "type": "audio | video",
    "streamId": "string | null"
  }
]
```

---

## Media / stream

### `GET /api/media/stream`

Sirve un archivo de audio o video **desde la biblioteca local**. Modo kiosko: no se descarga on-the-fly desde YouTube.

**Query**

| Parámetro | Tipo   | Descripción   |
|----------|--------|----------------|
| id       | string | ID del media (media_library.id si source=local, o video id de YouTube si source=youtube) |
| source   | string | `local` \| `youtube` (por defecto `local`) |

- **source=local:** `id` es `media_library.id`. Se sirve el archivo desde `MEDIA_ROOT` según `local_path`.
- **source=youtube:** Solo se sirve si ese video **ya está en media_library** (descargado previamente). Si no está, responde **404** indicando que hay que añadir el ítem a la cola para descargarlo.

**Respuesta:** `200 OK` — cuerpo binario (audio/mpeg, video/mp4, etc.) o `404` / `403` si no se puede servir.

---

## Cola

### `GET /api/queue`

Obtiene la cola de reproducción (FIFO, solo no reproducidos).

**Respuesta:** `200 OK`

```json
{
  "queue": [
    {
      "queueId": "string",
      "addedAt": "string (ISO)",
      "order": 0,
      "id": "string",
      "source": "string",
      "title": "string",
      "artist": "string | null",
      "album": "string | null",
      "durationSeconds": 0,
      "thumbnailUrl": "string | null",
      "type": "audio | video",
      "streamId": "string | null",
      "downloadId": "string | null",
      "downloadStatus": "string | null"
    }
  ]
}
```

`downloadId` y `downloadStatus` aparecen cuando el ítem se encoló para descarga (YouTube no estaba en biblioteca). `downloadStatus`: `queued` \| `downloading` \| `done` \| `failed`. Cuando es `done`, el backend ya actualizó `streamId` con la URL local.

### `POST /api/queue`

Añade un item a la cola. Descuenta créditos (`COST_PER_SONG`). Si no hay saldo suficiente devuelve `402 Payment Required`.

**Body**

```json
{
  "mediaItem": {
    "id": "string",
    "source": "string",
    "title": "string",
    "artist": "string | null",
    "album": "string | null",
    "durationSeconds": 0,
    "thumbnailUrl": "string | null",
    "type": "audio | video",
    "streamId": "string | null"
  }
}
```

**Respuesta:** `200 OK` — mismo formato que `GET /api/queue` con la cola actualizada.

### `POST /api/queue/next`

Marca el primer item de la cola como reproducido y devuelve la cola actualizada (sin ese item).

**Respuesta:** `200 OK` — mismo formato que `GET /api/queue`.

### `DELETE /api/queue`

Vacía la cola (solo items no reproducidos).

**Respuesta:** `200 OK` — `{ "queue": [] }`.

---

## Créditos

### `GET /api/credits`

Obtiene el saldo del usuario (por ahora usuario `default`).

**Respuesta:** `200 OK`

```json
{
  "id": "string",
  "balance": 0,
  "updatedAt": "string (ISO)"
}
```

### `POST /api/credits/add`

Suma créditos (admin/demo).

**Body**

```json
{
  "amount": 100
}
```

**Respuesta:** `200 OK` — mismo formato que `GET /api/credits` con el saldo actualizado.

---

## Health

### `GET /health`

Comprueba que el servicio está vivo.

**Respuesta:** `200 OK` — cuerpo de texto `ok`.

---

## Admin

Cuando la variable de entorno **`ADMIN_PIN`** está definida, los endpoints de administración (reset, maintenance, downloads, audit-log) requieren un **token de sesión** en el header `Authorization: Bearer <token>`. El token se obtiene con `POST /api/admin/login`. Si `ADMIN_PIN` no está definido, no se exige autenticación. Ver [Panel de mantenedor](admin-panel.md).

### `POST /api/admin/login`

Inicia sesión con el PIN de administrador. Solo útil cuando `ADMIN_PIN` está definido.

**Body:** `{ "pin": "1234" }`

**Respuesta:** `200 OK` — `{ "token": "uuid", "expiresInSecs": 900 }`. Errores: `401` PIN incorrecto; `429` demasiados intentos; `400` si admin no configurado.

### `POST /api/admin/logout`

Cierra la sesión. Header: `Authorization: Bearer <token>`. **Respuesta:** `200 OK` — `{ "ok": true }`.

### `GET /api/admin/session`

Comprueba si el token es válido. Header: `Authorization: Bearer <token>`. **Respuesta:** `200 OK` — `{ "valid": true }` o `401`.

### `POST /api/admin/reset`

Borra la cola, la caché de medios y la biblioteca, y resetea los créditos. **Requiere sesión** si `ADMIN_PIN` está definido. **Respuesta:** `200 OK` — `{ "ok": true }`.

### `GET /api/admin/audit-log`

Últimas entradas del registro de auditoría. **Requiere sesión** si `ADMIN_PIN` está definido. **Respuesta:** `200 OK` — array con `id`, `action`, `entityType`, `entityId`, `payloadJson`, `createdAt`.

---

## Descargas (cola de descargas)

**Requieren sesión** si `ADMIN_PIN` está definido.

### `GET /api/downloads`

Lista los últimos jobs de descarga. **Respuesta:** `200 OK` — array con `id`, `youtubeVideoId`, `requestedMediaType`, `status`, `progress`, `targetPath`, `errorMessage`, `createdAt`, `updatedAt`.

### `POST /api/downloads/:id/retry`

Reintenta un job con estado `failed`. **Respuesta:** `200 OK` (cuerpo texto). `404` job no existe; `400` estado no es `failed`.

---

## Mantenimiento (vista mantenedores)

### `GET /api/maintenance`

Devuelve en una sola respuesta: cola actual, últimos ítems de la biblioteca, créditos y conteos. **Requiere sesión** si `ADMIN_PIN` está definido.

**Respuesta:** `200 OK`

```json
{
  "queue": [ /* mismos ítems que GET /api/queue */ ],
  "mediaLibrary": [
    {
      "id": "string",
      "title": "string",
      "artist": "string | null",
      "source": "string",
      "localPath": "string | null",
      "mediaType": "audio | video",
      "externalId": "string | null"
    }
  ],
  "credits": { "id": "string", "balance": 0, "updatedAt": "string" },
  "stats": {
    "queueCount": 0,
    "mediaLibraryCount": 0,
    "mediaCacheCount": 0
  }
}
```
