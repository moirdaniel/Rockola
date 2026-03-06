# Rockola Digital: app de escritorio y modo kiosko

## Objetivo

Convertir la rockola de **demo web** en una **aplicación de escritorio** con **modo kiosko**, pensada para **reciclar PCs antiguos**: un equipo viejo se convierte en una rockola/jukebox dedicada, sin necesidad de mantener un navegador abierto ni depender de un servidor en la red.

## Por qué escritorio + kiosko

- **Escritorio:** Un solo ejecutable (o instalador) que el usuario abre como cualquier programa; el backend puede ir embebido o como proceso local.
- **Modo kiosko:** Pantalla completa, sin barras de ventana ni menús del SO a la vista; la app ocupa todo y es difícil salir por error. Ideal para bares, eventos o uso en un rincón.
- **PCs antiguos:** Menor consumo de recursos que un navegador + pestañas; posibilidad de usar tecnologías ligeras (p. ej. Tauri) para que funcione en máquinas con poca RAM y CPU.

## Opciones técnicas

| Enfoque | Ventajas | Desventajas |
|--------|----------|-------------|
| **Tauri** | Binario pequeño, usa Rust (ya tenemos backend en Rust), bajo uso de RAM/CPU, ideal para PCs viejos. El frontend actual (React/Vite) se empaqueta dentro. | Requiere integrar el backend dentro del proceso o lanzarlo como subproceso. |
| **Electron** | Muy documentado, el frontend web corre tal cual. | Binarios grandes (~150 MB+), más consumo de memoria; en PCs muy antiguos puede ir justo. |
| **PWA en modo ventana** | Cambio mínimo: “instalar” la web como app y abrir en ventana sin chrome. | Sigue siendo un navegador por debajo; menos control que una app nativa y no es “modo kiosko” real sin más pasos. |

**Recomendación:** **Tauri** para priorizar PCs antiguos y reciclaje: mismo ecosistema Rust, ejecutable ligero y modo kiosko nativo (ventana fullscreen, sin marco).

## Modo kiosko (comportamiento deseado)

- **Pantalla completa** (fullscreen), sin barra de título ni bordes.
- **Opcional:** atajo de teclado para salir (p. ej. `Ctrl+Alt+Q` o `Ctrl+Shift+K`) para el mantenedor; en kiosko “estricto” se puede desactivar.
- **Opcional:** arranque automático al iniciar el SO (script de sistema o entrada en “aplicaciones al inicio”).
- **Opcional:** evitar que el usuario cierre la app con Alt+F4 (o permitirlo solo con contraseña / modo admin).

En Tauri: ventana a pantalla completa, `decorations: false`, y manejo de shortcuts en el frontend o vía comandos Tauri.

## Backend en app de escritorio

- **Opción A – Embeber:** El mismo binario Tauri lanza el servidor Axum en un hilo/subproceso y abre la ventana que apunta a `http://127.0.0.1:3000` (o un puerto fijo). Base de datos y `MEDIA_ROOT` en una carpeta del usuario (p. ej. `~/.local/share/rockola/` o similar).
- **Opción B – Incluir binario:** El instalador incluye el backend como ejecutable aparte; el instalador o un script lo inicia antes de abrir la ventana (mismo flujo que ahora: backend 3000, frontend en ventana).
- **Opción C – Todo en uno:** Migrar la lógica del backend a “comandos” Tauri y usar SQLite desde ahí; la UI sigue siendo la misma pero las llamadas van a Tauri en lugar de HTTP. Más trabajo de refactor.

Para reciclaje rápido de PCs, **A** o **B** permiten reutilizar el backend actual casi sin cambios.

## Resumen

1. **Copia guardada:** La demo web está respaldada en `backups/rockola-web-demo-web-20250305.tar.gz`.
2. **Siguiente paso:** Crear proyecto Tauri que cargue el frontend (build de Vite) y, en modo escritorio, arranque el backend (embebido o como proceso) y abra la ventana en `http://127.0.0.1:3000` (o proxy al backend).
3. **Modo kiosko:** Ventana fullscreen sin decoraciones; atajo opcional para salir; opcional autoarranque y bloqueo de cierre.
4. **PCs antiguos:** Tauri + backend Rust mantienen el uso de recursos bajo para que la rockola sea usable en equipos reciclados.

Cuando quieras avanzar con Tauri, el siguiente paso concreto sería: inicializar `rockola-desktop` con Tauri, configurar el build para usar el `dist` del frontend actual y añadir la lógica para iniciar el backend y abrir la app en modo ventana/kiosko.

---

## Reproducción local-first y admin

- **Reproducción:** Solo desde archivos locales (`MEDIA_ROOT`). YouTube solo para buscar y descargar; no hay stream remoto ni descarga al reproducir. Ver [Almacenamiento local](storage-layout.md).
- **Mantenedor:** El panel de mantenedores (cola, biblioteca, descargas, reset) puede protegerse con PIN (`ADMIN_PIN`). Ver [Panel de mantenedor](admin-panel.md).

---

## Tauri ya integrado (estado actual)

El frontend tiene **Tauri 2** integrado en `frontend/`:

- **Dependencias:** `@tauri-apps/cli`, `@tauri-apps/api` en el frontend.
- **Rust:** `frontend/src-tauri/` con Cargo, `tauri.conf.json`, `lib.rs`, `main.rs`, capacidades e iconos generados desde `public/favicon.svg`.
- **Vite:** Configurado para Tauri (puerto 5173 fijo, `strictPort`, `envPrefix` con `TAURI_`, build target chrome105/safari13).

### Cómo ejecutar la app de escritorio

1. **Requisitos:** Rust, Node.js, y dependencias de Tauri para tu SO (en Linux: webkit2gtk, etc.; ver [Tauri - requisitos](https://v2.tauri.app/start/prerequisites/)).
2. **Backend:** La UI en Tauri sigue usando el API en `http://localhost:3000`. Arranca el backend antes (en otra terminal): `cd backend && ./run.sh`.
3. **Desarrollo:** `cd frontend && npm run tauri dev`. Si el puerto 5173 está ocupado, cierra el otro proceso. `tauri dev` lanza Vite y abre la ventana.
4. **Build:** `cd frontend && npm run tauri build`. Salida en `src-tauri/target/release/` (binario) y `src-tauri/target/release/bundle/` (instaladores).

### Modo kiosko (próximo paso)

En `tauri.conf.json` → `app.windows[0]` se puede poner `"fullscreen": true` y `"decorations": false`. Opcional: comando Tauri para alternar fullscreen con un atajo.
