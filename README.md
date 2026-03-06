# Rockola Digital

Rockola (jukebox) digital: actualmente **demo web** (búsqueda unificada, cola, créditos). En camino: **app de escritorio** con **modo kiosko** para reciclar PCs antiguos como rockola dedicada (ver [docs/desktop-kiosk.md](docs/desktop-kiosk.md)).

## Stack

- **Frontend**: React 18, TypeScript, Vite, TailwindCSS, Zustand, React Query, React Router
- **Backend**: Rust, Axum, Tokio, SQLx, SQLite

## Inicio rápido

**Requisito:** Para que la búsqueda en YouTube funcione, instala [yt-dlp](https://github.com/yt-dlp/yt-dlp) (en Arch: `sudo pacman -S yt-dlp ffmpeg`).

**Levantar todo (una sola terminal):** desde la raíz del repo ejecuta `./start.sh`. Se inicia el backend (puerto 3000) y luego el frontend (puerto 5173). Ctrl+C cierra ambos.

**O en dos terminales:**

1. **Arranca el backend** (terminal 1):
   ```bash
   cd backend
   cp .env.example .env   # si aún no existe
   mkdir -p data
   ./run.sh
   ```
   (`./run.sh` recompila y ejecuta el backend desde esta carpeta; así la ruta **GET /api/maintenance** estará disponible para la vista Mantenedores. Si prefieres: `cargo run`.)
   Debe quedar escuchando en `http://localhost:3000`.

2. **Arranca el frontend** (terminal 2):
   ```bash
   cd frontend
   npm install
   npm run dev
   ```
   Abre **http://localhost:5173**. El proxy de Vite redirige `/api` al backend en el puerto 3000.

3. **Vista Mantenedores**: Admin → **Vista mantenedores** (o `/mantenedores`). Si ves **"Not Found"**:
   - Cierra el proceso del backend (Ctrl+C) y desde la carpeta **backend** ejecuta **`./run.sh`** (o `cargo run`). La ruta **GET /api/maintenance** debe estar disponible.
   - Si al arrancar el backend sale **"Address already in use"**, hay otro proceso en el puerto 3000 (por ejemplo un backend antiguo). Ciérralo con `pkill -f rockola-backend` o desde la terminal donde corre, y vuelve a ejecutar `./run.sh`.

Documentación en `/docs`:

- [Arquitectura](docs/arquitectura.md)
- [yt-dlp](docs/yt-dlp.md)
- [Ejecución local](docs/run-local.md)
- [API](docs/api.md)
- [Guía de funciones](docs/funciones.md)
- [Escritorio y modo kiosko](docs/desktop-kiosk.md) (plan para PCs antiguos)

**Copia de la demo web:** en `backups/` hay un snapshot (`.tar.gz`) de la versión solo-web para no perderla al pasar a escritorio; ver `backups/README.md`.

## Docker

```bash
cd docker && docker compose up --build
```

App en http://localhost:8080 (timezone America/Santiago). La vista Mantenedores funciona si la imagen del backend se ha construido con el código actual (`docker compose build backend`).

## Estructura

```
rockola-web/
├── frontend/       # SPA React + Tauri (app escritorio)
│   ├── src-tauri/  # Rust Tauri 2 (ventana, build, iconos)
├── backend/        # API Rust
├── docker/         # Dockerfiles y compose
├── docs/           # Documentación
└── backups/        # Copia demo web
```

## Tests

```bash
# Backend (Rust)
cd backend && cargo test

# Frontend (Vitest)
cd frontend && npm run test
```

## App de escritorio (Tauri)

Con el backend en marcha en otro terminal (`cd backend && ./run.sh`):

```bash
cd frontend
npm run tauri dev    # desarrollo (abre ventana)
npm run tauri build # ejecutable e instaladores en src-tauri/target/release/bundle/
```

Requisitos: [Tauri - Prerequisites](https://v2.tauri.app/start/prerequisites/). Detalles en [docs/desktop-kiosk.md](docs/desktop-kiosk.md).

## TODOs siguientes fases

- [ ] **Modo kiosko** en Tauri (fullscreen, sin decoraciones) y opcional arranque del backend desde la app — ver [docs/desktop-kiosk.md](docs/desktop-kiosk.md)
- [ ] Reemplazar adapters mock por YouTube API y Spotify API reales
- [ ] Proxy de stream en backend para fuentes externas
- [ ] Autenticación y múltiples usuarios
- [ ] Playlists completas y favoritos
- [ ] PWA y soporte offline
- [ ] Tests E2E y unitarios

## Subir a GitHub

El proyecto ya está inicializado con git (rama `main`, primer commit hecho). **No tengo acceso a tu cuenta de GitHub**, así que debes crear el repositorio y enlazarlo tú:

1. **Crea el repositorio en GitHub** (vacío, sin README):
   - Ve a [github.com/new](https://github.com/new)
   - Nombre sugerido: `rockola-web`
   - No marques "Add a README" (ya existe en el proyecto)

2. **Enlaza y sube** desde la raíz del proyecto (sustituye `TU_USUARIO` por tu usuario de GitHub):

   ```bash
   git remote add origin https://github.com/TU_USUARIO/rockola-web.git
   git push -u origin main
   ```

   Si usas SSH:

   ```bash
   git remote add origin git@github.com:TU_USUARIO/rockola-web.git
   git push -u origin main
   ```

Opcional: instala [GitHub CLI](https://cli.github.com/) (`gh`) y autentícate; luego puedes crear el repo y hacer push con:

   ```bash
   gh repo create rockola-web --private --source=. --remote=origin --push
   ```
