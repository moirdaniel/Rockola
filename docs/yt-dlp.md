# yt-dlp en Rockola

El backend usa **yt-dlp** para buscar en YouTube y descargar audio. La primera vez que se reproduce un tema de YouTube se descarga en segundo plano y se guarda en `MEDIA_ROOT`; las siguientes veces se sirve desde local.

## Instalación

### Arch Linux

```bash
sudo pacman -S yt-dlp
sudo pacman -S ffmpeg   # necesario para convertir a mp3
```

### Otras distros

- **Debian/Ubuntu**: `sudo apt install yt-dlp ffmpeg`
- **Fedora**: `sudo dnf install yt-dlp ffmpeg`
- **macOS**: `brew install yt-dlp ffmpeg`

## Variables de entorno (backend)

| Variable     | Descripción                          | Por defecto      |
|-------------|--------------------------------------|------------------|
| MEDIA_ROOT  | Directorio raíz para audio/video     | ./rockola-media  |
| YT_DLP_PATH | Comando o ruta de yt-dlp             | yt-dlp           |

## Uso desde el backend

- **Búsqueda**: si no hay resultados en el índice local, se ejecuta  
  `yt-dlp --flat-playlist -j --no-download "ytsearch10: QUERY"`  
  y se parsea la salida JSON.
- **Descarga**: al reproducir un tema de YouTube no cacheado se ejecuta  
  `yt-dlp -x --audio-format mp3 --audio-quality 0 -o "MEDIA_ROOT/audio/youtube/yt-VIDEO_ID.%(ext)s" URL`  
  y se guarda la metadata en `media_library`.

## Descarga manual (ejemplos)

Solo audio (recomendado para rockola):

```bash
yt-dlp -x --audio-format mp3 --audio-quality 0 \
  -o "/media/rockola/%(title)s.%(ext)s" \
  "https://www.youtube.com/watch?v=VIDEO_ID"
```

Video completo:

```bash
yt-dlp -f "bestvideo+bestaudio/best" \
  -o "/media/rockola/%(title)s.%(ext)s" \
  "URL"
```

## Estructura recomendada en disco

```
/rockola-media
   /audio
      artista/
         album/
            track.mp3
      youtube/
         yt-VIDEO_ID.mp3
   /video
      artista/
         video.mp4
```

El backend crea automáticamente `MEDIA_ROOT/audio/youtube/` al descargar.
