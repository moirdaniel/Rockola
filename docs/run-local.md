# Ejecución en local - Rockola Digital

## Requisitos

- **Frontend**: Node.js 18+, npm
- **Backend**: Rust (stable), SQLite
- **Opcional**: Docker y Docker Compose

## 1. Backend (Rust)

```bash
cd backend
cp .env.example .env
# Crear directorio de datos para SQLite (opcional; el backend lo crea si falta)
mkdir -p data

# Compilar y ejecutar (ejecutar siempre desde el directorio backend)
cargo run
```

Si aparece **"unable to open database file"**, usa ruta absoluta en `.env`:

```bash
# En backend/.env
DATABASE_URL=sqlite:/ruta/completa/a/rockola-web/backend/data/rockola.db
```

O quita la línea `DATABASE_URL` de `.env` para usar la ruta por defecto del proyecto.

El API quedará en `http://localhost:3000`. Health: `curl http://localhost:3000/health`.

Si aparece **"Address already in use (os error 98)"**, el puerto 3000 está ocupado (p. ej. otra instancia del backend). Libera el puerto (`kill <PID>` del proceso que usa el 3000 con `lsof -i :3000`) o cambia `PORT=3001` en `.env`.

## 2. Frontend (Vite)

```bash
cd frontend
npm install
cp .env.example .env
# Opcional: VITE_API_BASE_URL=http://localhost:3000 si no usas proxy

npm run dev
```

La app estará en `http://localhost:5173`. El proxy de Vite redirige `/api` al backend en el puerto 3000. Si usas otro puerto para el backend (p. ej. `PORT=3001`), cambia en `frontend/vite.config.ts` la opción `proxy['/api'].target` a `http://localhost:3001`.

**Si al buscar no ves resultados**: asegúrate de que el backend esté en marcha y de tener **yt-dlp** instalado (`yt-dlp --version`). Si falta o falla, el backend devuelve un error y en la app verás un modal con el mensaje (p. ej. "yt-dlp no encontrado"). Instalación: `sudo pacman -S yt-dlp` (Arch) o `pip install yt-dlp` / ver [yt-dlp](https://github.com/yt-dlp/yt-dlp).

## 3. Probar flujo

1. Abre `http://localhost:5173`.
2. En **Admin** agrega créditos si el saldo es 0.
3. En **Inicio** busca (por ejemplo "Queen" o "Eagles") y agrega canciones a la cola.
4. La primera se reproducirá en la barra inferior; **Siguiente** o al terminar pasa a la siguiente.
5. **Cola** muestra el contenido; **Créditos** el saldo.

## 4. Docker (producción)

Desde la raíz del repo:

```bash
cd docker
docker compose up --build
```

- Frontend: `http://localhost:8080`
- Backend solo interno (proxy desde nginx).
- Timezone: `America/Santiago`.
- Volumen `backend_data` para SQLite.

## Variables de entorno

### Backend (`.env`)

| Variable        | Descripción              | Por defecto (si no .env) |
|----------------|--------------------------|---------------------------|
| DATABASE_URL   | Ruta SQLite              | sqlite:./data/rockola.db  |
| PORT           | Puerto HTTP              | 3000                      |
| RUST_LOG       | Nivel de log             | info                      |
| COST_PER_SONG  | Créditos por canción     | 100                       |

### Frontend (`.env`)

| Variable            | Descripción                    |
|--------------------|---------------------------------|
| VITE_API_BASE_URL  | Base URL del API (vacío = mismo origen/proxy) |
