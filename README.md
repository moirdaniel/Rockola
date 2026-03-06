# Rockola Digital

Jukebox digital para reproducir **audio y video** con búsqueda unificada, cola de reproducción y sistema de créditos. Incluye **app web**, **app de escritorio (Tauri)** con modo kiosko y **actualizaciones automáticas** para despliegues en PCs reciclados.

---

## Funcionalidades

- **Búsqueda unificada** — Índice local (SQLite) + YouTube vía [yt-dlp](https://github.com/yt-dlp/yt-dlp); resultados en un solo listado.
- **Cola de reproducción** — FIFO persistida; reproducción automática de audio y video.
- **Créditos** — Saldo por usuario; costo por canción configurable; validación en backend.
- **Panel mantenedor** — Vista admin: cola, biblioteca, descargas, reset de datos, registro de auditoría; opcionalmente protegido por PIN.
- **App de escritorio (Tauri 2)** — Ventana nativa, modo kiosko (pantalla completa, sin decoraciones), ideal para rockola dedicada.
- **Actualizaciones firmadas** — Solo admin puede buscar/descargar/instalar actualizaciones; modo recuperación ante fallos de actualización.

---

## Stack

| Capa     | Tecnología |
|----------|------------|
| Frontend | React 18, TypeScript, Vite, TailwindCSS, Zustand, React Query, React Router |
| Escritorio | Tauri 2, plugin updater, plugin process |
| Backend  | Rust, Axum, Tokio, SQLx |
| Base de datos | SQLite |
| Media    | yt-dlp, ffmpeg |

---

## Inicio rápido

**Requisito:** Para búsqueda en YouTube instala **yt-dlp** (y ffmpeg para audio). En Arch: `sudo pacman -S yt-dlp ffmpeg`.

### Una sola terminal

Desde la raíz del repo:

```bash
./start.sh
```

Se inicia el backend (puerto 3000) y el frontend (puerto 5173). Abre **http://localhost:5173**. Ctrl+C cierra ambos.

### Dos terminales

**Terminal 1 — Backend:**

```bash
cd backend
cp .env.example .env   # si aún no existe
mkdir -p data
./run.sh
```

Debe quedar en `http://localhost:3000`.

**Terminal 2 — Frontend:**

```bash
cd frontend
npm install
npm run dev
```

Abre **http://localhost:5173**. El proxy de Vite redirige `/api` al backend.

**Vista Mantenedores:** Admin → **Vista mantenedores** (o `/mantenedores`). Si ves "Not Found", arranca el backend con `./run.sh` desde `backend/` para que exista `GET /api/maintenance`. Si el puerto 3000 está ocupado: `pkill -f rockola-backend` y vuelve a ejecutar `./run.sh`.

---

## Docker

```bash
cd docker && docker compose up --build
```

App en **http://localhost:8080** (timezone America/Santiago).

---

## App de escritorio (Tauri)

Con el backend en marcha en otro terminal (`cd backend && ./run.sh`):

```bash
cd frontend
npm run tauri dev    # desarrollo (abre ventana)
npm run tauri build  # ejecutable e instaladores en src-tauri/target/release/bundle/
```

Requisitos: [Tauri — Prerequisites](https://v2.tauri.app/start/prerequisites/).

- **Modo kiosko:** fullscreen, sin barra de título; ideal para PCs antiguos como rockola dedicada.
- **Actualizaciones:** configuración en Mantenedores → Actualizaciones; ver [docs/updates.md](docs/updates.md).
- Detalles: [docs/desktop-kiosk.md](docs/desktop-kiosk.md).

---

## Tests

```bash
# Backend (Rust)
cd backend && cargo test

# Frontend (Vitest)
cd frontend && npm run test
```

---

## Estructura del proyecto

```
rockola-web/
├── frontend/           # SPA React + Tauri
│   ├── src/            # Componentes, páginas, stores, servicios
│   └── src-tauri/      # Rust Tauri 2 (ventana, build, actualizaciones)
├── backend/            # API Rust (Axum, SQLite)
├── docker/             # Dockerfiles y docker-compose
└── docs/               # Documentación
```

---

## Documentación

| Documento | Contenido |
|-----------|-----------|
| [Arquitectura](docs/arquitectura.md) | Visión general, stack, flujos, BD, estado global |
| [API](docs/api.md) | Endpoints del backend |
| [Guía de funciones](docs/funciones.md) | Qué hace cada módulo/función principal |
| [yt-dlp](docs/yt-dlp.md) | Búsqueda y descargas con yt-dlp |
| [Ejecución local](docs/run-local.md) | Desarrollo local paso a paso |
| [Escritorio y modo kiosko](docs/desktop-kiosk.md) | Tauri, fullscreen, despliegue en PC |
| [Actualizaciones](docs/updates.md) | Actualizaciones firmadas, canales, modo recuperación |
| [Panel mantenedor](docs/admin-panel.md) | Admin, PIN, sesión, auditoría |
| [Storage](docs/storage-layout.md) | Layout de datos y medios |

---

## Próximas fases

- [ ] Reemplazar adapters mock por YouTube API / Spotify API reales
- [ ] Proxy de stream en backend para fuentes externas
- [ ] Autenticación y múltiples usuarios
- [ ] Playlists y favoritos
- [ ] PWA y soporte offline
- [ ] Tests E2E

---

## Repositorio

Código en **[GitHub — moirdaniel/Rockola](https://github.com/moirdaniel/Rockola)**.

---

**Autor:** Daniel Moir · Proyecto personal / experimental
