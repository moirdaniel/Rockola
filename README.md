# 🎵 Rockola

Rockola es una aplicación multimedia tipo **jukebox / rockola digital**, orientada a la reproducción local de **audio y video**, con una interfaz inmersiva tipo TV/Karaoke y soporte para ejecución en **desktop (Tauri + Rust)** y **web**.

El proyecto está pensado para funcionar como un sistema de reproducción continuo (24/7), con cola de reproducción, control por inactividad y múltiples fuentes de medios.

---

## ✨ Características Principales

- **Reproducción multimedia**: Soporte para audio y video locales
- **Cola de reproducción**: Organiza tu lista de reproducción con arrastrar y soltar
- **Interfaz inmersiva**: Modo TV/Karaoke con pantalla completa automática
- **Búsqueda inteligente**: Encuentra rápidamente tus artistas y canciones
- **Control de volumen y tiempo**: Controles completos de reproducción
- **Modo Rockola**: Reproducción automática con cuenta regresiva
- **Soporte multiplataforma**: Desktop (Windows, Mac, Linux) y web

---

## 🧱 Arquitectura General

```
Rockola/
├── apps/
│   ├── desktop/           # Aplicación de escritorio (Tauri + Rust)
│   └── ui/                # Interfaz web (React + TypeScript + Astro)
├── core/
│   ├── db/                # Base de datos SQLite (Rusqlite)
│   ├── domain/            # Lógica de dominio (Rust)
│   ├── events/            # Sistema de eventos (Rust)
│   ├── player/            # Motor de reproducción (Rust)
│   └── scan/              # Escáner de medios (Rust)
├── ui/                    # Frontend web
│   ├── src/
│   │   ├── lib/           # Librerías y utilidades
│   │   ├── pages/         # Páginas de la aplicación
│   │   ├── styles/        # Hojas de estilo
│   │   └── ui/            # Componentes de interfaz
└── README.md
```

---

## 🛠 Tecnologías Utilizadas

### Backend (Rust)
- **Tauri v2**: Framework para aplicaciones desktop
- **Rusqlite**: Base de datos SQLite
- **Tokio**: Runtime asíncrono
- **Axum**: Servidor HTTP para medios
- **Serde**: Serialización de datos

### Frontend (Web/React)
- **React**: Biblioteca de interfaces
- **Astro**: Framework web moderno
- **TypeScript**: Tipado estático
- **CSS Moderno**: Estilos con variables y flexibilidad

---

## 🚀 Funcionalidades Destacadas

### Reproducción
- Reproductor de audio y video integrado
- Controles de reproducción (play/pause, seek, volumen)
- Modo fullscreen automático
- Temporizador de inicio automático
- Historial de reproducción

### Gestión de Medios
- Escaneo automático de carpetas
- Indexación de artistas y canciones
- Búsqueda avanzada
- Vista por artistas
- Soporte para múltiples fuentes

### Experiencia de Usuario
- Tema claro/oscuro
- Interfaz responsive
- Modo TV/Karaoke
- Arrastrar y soltar en la cola
- Repetición y aleatorio

---

## 📁 Estructura de Carpetas

### `apps/desktop/`
Aplicación de escritorio con Tauri que envuelve la interfaz web y proporciona acceso al sistema de archivos.

### `ui/`
Interfaz web desarrollada con React y Astro, adaptable para web y desktop.

### `core/db/`
Capa de persistencia con SQLite, migraciones y consultas optimizadas.

### `core/player/`
Motor completo de reproducción con cola, historial y controles.

### `core/scan/`
Sistema de escaneo de medios con seguimiento de cambios.

---

## 🏗 Compilación y Ejecución

### Prerrequisitos
- Rust (cargo) - Versión 1.70 o superior
- Node.js y npm - Versión 18 o superior
- Tauri CLI
- Dependencias del sistema (WebKit2GTK, GTK3, OpenSSL, etc.)

### Configuración del Entorno

1. Clona el repositorio:
```bash
git clone https://github.com/moirdaniel/Rockola.git
cd Rockola
```

2. Instala Rust y las dependencias del sistema:
Sigue las instrucciones detalladas en el archivo [INSTALACION.md](./INSTALACION.md) para tu sistema operativo.

3. Instala las dependencias del frontend:
```bash
cd ui
npm install
```

4. Ejecuta la aplicación de desarrollo:
```bash
# En una terminal, iniciar el servidor de desarrollo del frontend
cd ui
npm run dev

# En otra terminal, iniciar la aplicación desktop
cd apps/desktop
cargo tauri dev
```

### Compilación para Producción
```bash
# Compilar el frontend
cd ui
npm run build

# Compilar la aplicación desktop
cd apps/desktop
cargo tauri build
```

---

## 🧪 Desarrollo

### Contribuciones
Las contribuciones son bienvenidas. Por favor, abre un issue o un pull request con tus ideas o correcciones.

### Estado del Proyecto
🟢 En desarrollo activo  
Actualmente se están implementando las siguientes características:
- Sistema de reproducción completo
- Mejoras en la interfaz de usuario
- Soporte para más formatos de medios
- Funciones de playlist y favoritos

---

## 📄 Licencia

Este proyecto es de código abierto y está disponible bajo los términos de la licencia MIT.

---

## 🧑‍💻 Autor

Daniel Moir  
Proyecto personal / experimental
