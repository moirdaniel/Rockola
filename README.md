# 🎵 Rockola

Rockola es una aplicación multimedia tipo **jukebox / rockola digital**, orientada a la reproducción local de **audio y video**, con una interfaz inmersiva tipo TV/Karaoke y soporte para ejecución en **desktop (Tauri + Arch Linux)** y **web**.

El proyecto está pensado para funcionar como un sistema de reproducción continuo (24/7), con cola de reproducción, control por inactividad y futura integración con múltiples fuentes de medios.

---

## 🧱 Arquitectura general

```
Rockola/
├── apps/
│   ├── ui/                # Frontend (React + TypeScript)
│   └── desktop/           # Wrapper Desktop (Tauri)
├── core/
│   └── db/                # Lógica de base de datos compartida
└── README.md
```

---

## 🖥 Frontend (UI)

### Stack
- React
- TypeScript
- CSS Grid / Flexbox

### Funcionalidades actuales
- Listado de artistas
- Selección de ítems
- Cola de reproducción
- Reproductor de audio / video
- Countdown antes de reproducir
- Fullscreen automático por inactividad
- Bloqueo de duplicados en la cola

---

## 🖥 Desktop (Tauri)

### Stack
- Tauri v2
- Rust
- WebView (Wry)

### Estado
Backend en proceso de estabilización.  
El foco actual está en UX y reproducción multimedia.

---

## ⏱ Comportamiento tipo Rockola

- Countdown de 10 segundos antes de reproducir
- Reproducción automática
- Fullscreen tras inactividad del usuario

---

## 🧠 Estado del proyecto

🟡 En desarrollo activo  
Prioridad actual:
- Estabilidad del reproductor
- UX tipo rockola / karaoke
- Flujo continuo de reproducción

---

## 🧑‍💻 Autor

Daniel Moir  
Proyecto personal / experimental
