# Rockola Electron + React (MVP)

## Requisitos
- Node.js 18+
- (Opcional) FFmpeg instalado en el sistema para mejores compatibilidades
- Directorios locales con música/videos
- (Opcional) Credenciales Spotify para búsquedas por artista y top tracks

## Configuración
1. Copia `.env.example` a `.env` y setea:
   - `ROCKOLA_MEDIA_DIRS` con rutas absolutas separadas por `;`.
   - `SPOTIFY_CLIENT_ID` y `SPOTIFY_CLIENT_SECRET` si usarás búsqueda en Spotify (solo metadatos en este MVP).
2. Instala dependencias:
   ```bash
   npm i
   ```
3. Dev:
   ```bash
   npm run dev
   ```
   - Vite levantará el renderer en `http://localhost:5173`
   - Electron levantará la app y cargará esa URL.

## Uso
- Presiona **📚 Reindexar** para escanear los directorios de `ROCKOLA_MEDIA_DIRS`.
- Busca por **artista** (barra superior) → selecciona un artista en la lista → verás sus pistas locales.
- En la pestaña **YouTube** del artista, se muestran resultados relacionados: puedes reproducir directamente.
- La pestaña **Spotify** se deja lista para integrar (metadatos / sugerencias); el playback de Spotify no está implementado por requerir SDK Premium.

## Limitaciones conocidas
- `ytdl-core` puede devolver formatos no progresivos según el video; se intenta elegir `audioandvideo` progresivo y se cae a `audio` si no hay. Para máxima robustez, integrar muxing con `ffmpeg` (pendiente).
- Indexación: heurística básica para artista/título si no hay tags. Recomendado mantener estructura `Artista/Album/Track.ext` o `Artista - Título.ext`.
- No hay colas/playlist aún. Se reproduce un solo elemento a la vez.
- Los directorios se pueden actualizar desde la app con el botón **⚙️ Configurar**.

## Roadmap corto
- [ ] Playlist/cola de reproducción y modo "jukebox" con créditos.
- [ ] Carga/edición de arte de artista y carátulas.
- [ ] Integración YouTube más robusta (HLS/dash → transcode con ffmpeg cuando sea necesario).
- [ ] Búsqueda global unificada (local + YouTube + Spotify).
- [ ] Theming/skins para la interfaz de rockola.
- [ ] Empaquetado con `electron-builder`.

## Licencia
MIT

---

# Guía PASO A PASO (Dev → MVP funcionando)

> Este paso a paso asume Windows/Linux/macOS con **Node 18+** y (opcional) **FFmpeg** instalado.

## 1) Clonar / crear carpeta de proyecto
```bash
mkdir rockola && cd rockola
```

## 2) Variables de entorno
- Copia `.env.example` a `.env` y setea:
  - `ROCKOLA_MEDIA_DIRS` con rutas absolutas separadas por `;`.
  - (Opcional) `SPOTIFY_CLIENT_ID` y `SPOTIFY_CLIENT_SECRET` si usarás búsqueda en Spotify (solo metadatos en el MVP).

## 3) Instalar dependencias
```bash
npm i
```

## 4) Ejecutar en modo desarrollo
```bash
npm run dev
```
- Se abrirá la app de Electron.

## 5) Indexar tu biblioteca local
- En la app, presiona **⚙️ Configurar** para definir las rutas si aún no lo hiciste.
- Presiona **📚 Reindexar** (usa las rutas definidas) para construir la librería.
- Busca **por artista** en la barra superior, selecciona un artista → verás sus pistas locales.

## 6) Reproducir desde YouTube
- En el detalle del artista, pestaña **YouTube** → resultados → **▶️** para reproducir.

## 7) (Opcional) Spotify – búsqueda por artista
- Define `SPOTIFY_CLIENT_ID`/`SPOTIFY_CLIENT_SECRET` y reinicia.
- Pestaña **Spotify** (metadatos, preview si existe `preview_url`).

## 8) Estructura de carpetas recomendada para medios
```
/Music
  /Soda Stereo
    /Canción Animal
      01 - En el séptimo día.flac
      02 - Un millón de años luz.flac
  /Charly García
    Charly García - Nos siguen pegando abajo.mp3
/Videos
  Queen - Live Aid 1985.mp4
```
- Si no hay tags, el indexador usa heurística `Artista - Título.ext` o nombre de carpeta como artista.

---

# Mejora de Interfaz (UI) Amigable y Editable

A continuación se añade **TailwindCSS** y **shadcn/ui** para acelerar el diseño, hacerlo consistente y **muy editable** por tokens/clases. Se incluyen cambios en archivos clave del renderer.

## A) Instalar y configurar Tailwind
```bash
npm i -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

- **tailwind.config.js**
```js
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './src/**/*.{ts,tsx}',
    './index.html'
  ],
  theme: {
    extend: {
      colors: {
        bg: '#0b0b0f',
        card: '#11131a',
        border: '#232637',
        text: '#e6e6e9',
        muted: '#9aa0a6'
      },
      borderRadius: {
        xl: '14px',
        '2xl': '18px'
      },
      boxShadow: {
        soft: '0 6px 20px rgba(0,0,0,.25)'
      }
    }
  },
  plugins: []
}
```

- **src/styles/index.css**
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

:root { color-scheme: dark; }
body { @apply bg-bg text-text; }
.btn { @apply inline-flex items-center gap-2 rounded-xl border border-border px-3 py-2 hover:brightness-110; }
.card { @apply rounded-2xl border border-border bg-card shadow-soft; }
.input { @apply w-full rounded-xl border border-border bg-[#15151c] px-3 py-2 text-text placeholder:opacity-60; }
.badge { @apply text-sm text-muted; }
.sidebar { @apply border-r border-border overflow-auto; }
.header { @apply flex items-center gap-3 p-3 border-b border-border; }
.maingrid { @apply grid; grid-template-columns: 340px 1fr; height: calc(100vh - 64px); }
.list { @apply p-3 grid gap-2; }
.item { @apply card p-3 flex items-center justify-between; }
```

## B) Instalar shadcn/ui (opcional pero recomendado)
```bash
npx shadcn@latest init
npx shadcn@latest add button card input tabs scroll-area separator
```

## C) Actualizar componentes del renderer a UI amigable

### `src/App.tsx`
```tsx
<header className="header">
  <button className="btn" onClick={handleScan}>📚 Reindexar</button>
  <SearchBar placeholder="Buscar artista local…" value={q} onChange={handleSearchArtists} />
  <button className="btn" onClick={() => setShowConfig(true)}>⚙️ Configurar</button>
</header>
```

### `src/components/SearchBar.tsx`
```tsx
<input
  className="input"
  placeholder={placeholder}
  value={value}
  onChange={(e) => onChange(e.target.value)}
/>
```

### `src/components/ArtistList.tsx`
```tsx
<div className="list">
  {artists.map(a => (
    <div key={a.id} className="item cursor-pointer" onClick={() => onPick(a)}>
      <div>
        <div className="font-semibold">{a.name}</div>
        <div className="badge">Artista</div>
      </div>
      <span>›</span>
    </div>
  ))}
</div>
```

### `src/components/ArtistDetail.tsx`
```tsx
<div className="flex gap-2">
  <button className={`btn ${tab==='local' ? 'brightness-110' : ''}`} onClick={() => setTab('local')}>Local</button>
  <button className={`btn ${tab==='youtube' ? 'brightness-110' : ''}`} onClick={() => setTab('youtube')}>YouTube</button>
  <button className={`btn ${tab==='spotify' ? 'brightness-110' : ''}`} onClick={() => setTab('spotify')}>Spotify</button>
</div>
```

### `src/components/Player.tsx`
```tsx
<div className="fixed left-5 right-5 bottom-5 card p-3">
  <div className="mb-1 opacity-90">Reproduciendo: {nowPlaying.title}</div>
  {isVideo ? (
    <video src={nowPlaying.src} controls className="w-full max-h-[360px]" />
  ) : (
    <audio src={nowPlaying.src} controls className="w-full" />
  )}
</div>
```

## D) Panel de Configuración (editable por usuario)
- Guarda rutas en SQLite (tabla `settings`) y permite editarlas desde la UI.

## E) Tematización ultra-rápida
- Cambia tokens en `tailwind.config.js` y utilidades en `src/styles/index.css`.

---

# Buenas prácticas y escalado
- **Cola/Jukebox**: crea una store (p. ej. Zustand) para manejar queue y créditos.
- **Persistencia**: guarda `artists/tracks/settings` en SQLite.
- **FFmpeg**: útil cuando `ytdl` no entregue progresivo.
- **Atajos**: agrega shortcuts de teclado.
- **Métricas**: registra reproducciones por artista/track.

---

# Checklist de entrega
- [x] App levanta con `npm run dev`.
- [x] Tailwind activo, UI con botones/cartas/inputs estilizados.
- [x] Reindexa directorios y navega por artista.
- [x] Reproduce local + YouTube.
- [x] Modal Config para rutas sin tocar `.env`.
- [x] Tokens de tema editables en `tailwind.config.js`.
