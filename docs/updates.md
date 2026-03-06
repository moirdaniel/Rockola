# Actualizaciones automáticas (Tauri)

La Rockola en modo escritorio (Tauri) soporta actualizaciones firmadas: solo el **mantenedor** (admin con sesión) puede buscar, descargar e instalar actualizaciones desde el panel Mantenedores → sección **Actualizaciones**.

## Requisitos

- **Tauri 2** con `tauri-plugin-updater` y `tauri-plugin-process`.
- Actualizaciones **firmadas**: no se pueden desactivar. Necesitas un par de claves (pública en la app, privada para firmar los instaladores).

## Generar claves de firma

En la raíz del proyecto frontend:

```bash
cd frontend
npm run tauri signer generate -- -w ~/.tauri/rockola.key
```

Esto crea:

- **`~/.tauri/rockola.key`** — clave privada. **No la compartas.** Guárdala en un lugar seguro; si la pierdes no podrás publicar nuevas actualizaciones para instalaciones ya desplegadas.
- **`~/.tauri/rockola.key.pub`** — clave pública. Su **contenido** (no la ruta) debe ir en `tauri.conf.json` → `plugins.updater.pubkey`.

## Configuración en `tauri.conf.json`

1. **Clave pública:** Pega el contenido de `rockola.key.pub` en `plugins.updater.pubkey` (reemplaza `REPLACE_WITH_PUBLIC_KEY_CONTENT`).

2. **Endpoints:** Por defecto se usa un placeholder de GitHub. Para **stable** y **beta**:
   - **Stable:** URL que devuelve un JSON de actualización (p. ej. `https://github.com/USER/REPO/releases/latest/download/latest.json`).
   - **Beta:** Opcional; en el código Rust `build_endpoints("beta")` puede usar una segunda URL (p. ej. `latest-beta.json`).

3. **Crear artefactos de actualización:**

```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "pubkey": "CONTENIDO_DE_rockola.key.pub",
      "endpoints": [
        "https://github.com/USER/REPO/releases/latest/download/latest.json"
      ],
      "dialog": false
    }
  }
}
```

## Formato del JSON de actualización (GitHub Releases / estático)

El endpoint debe devolver un JSON con esta forma (o la variante por plataforma):

```json
{
  "version": "0.2.0",
  "notes": "Correcciones y mejoras.",
  "pub_date": "2025-03-15T12:00:00Z",
  "platforms": {
    "linux-x86_64": {
      "signature": "CONTENIDO_DEL_ARCHIVO_.sig",
      "url": "https://.../Rockola_0.2.0_amd64.AppImage"
    },
    "windows-x86_64": {
      "signature": "CONTENIDO_DEL_ARCHIVO_.sig",
      "url": "https://.../Rockola_0.2.0_x64-setup.nsis.zip"
    },
    "darwin-x86_64": { "signature": "...", "url": "..." },
    "darwin-aarch64": { "signature": "...", "url": "..." }
  }
}
```

- **version:** SemVer (con o sin `v`).
- **signature:** Contenido del archivo `.sig` generado al construir con la clave privada.
- **url:** Enlace directo al instalador o paquete de actualización.

## Publicar una release (estable)

1. **Exportar la clave privada** (solo en tu máquina o CI segura):

   ```bash
   export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/rockola.key)"
   # Opcional si la clave tiene contraseña:
   export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="tu_password"
   ```

2. **Build:**

   ```bash
   cd frontend
   npm run build
   npm run tauri build
   ```

3. En `src-tauri/target/release/bundle/` (y subcarpetas `appimage/`, `nsis/`, `macos/`) tendrás los instaladores y sus `.sig`.

4. **Subir a GitHub Releases:**
   - Crear un release con tag (p. ej. `v0.2.0`).
   - Subir los archivos de cada plataforma y sus `.sig`.
   - Crear un `latest.json` (o el nombre que use tu endpoint) con la estructura anterior y las URLs de los artefactos subidos. Puedes colgar `latest.json` como asset del release y usar la URL de descarga directa como endpoint.

## Canales stable / beta

- En el código Rust, `build_endpoints(channel)` devuelve una URL u otra según `channel` (`"stable"` o `"beta"`).
- Puedes configurar en `tauri.conf.json` dos entradas en `endpoints` y en tiempo de ejecución elegir la primera (stable) o la segunda (beta), o construir la URL dinámicamente.
- Para beta, publica en una URL distinta (p. ej. `latest-beta.json`) con versiones de pre-release.

## Seguridad

- **Pubkey:** Solo la clave pública va en la app; así se verifica la firma sin exponer la privada.
- **Endpoints:** Usa HTTPS. No uses `dangerousInsecureTransportProtocol` en producción.
- **Almacenamiento de la clave privada:** En CI, usa secretos (GitHub Secrets, etc.) y nunca los subas al repo.

## Auditoría

Cada acción de actualización (check, available, download started/done, install started/done, failed) se registra en el backend en `admin_audit_log` si el mantenedor tiene sesión. Acciones: `UPDATE_CHECK`, `UPDATE_AVAILABLE`, `UPDATE_DOWNLOAD_STARTED`, `UPDATE_DOWNLOAD_DONE`, `UPDATE_INSTALL_STARTED`, `UPDATE_INSTALL_DONE`, `UPDATE_FAILED`. El payload incluye `version_current`, `version_target`, `error` cuando aplique. Si se activa el modo recuperación por crash-loop se registra `UPDATE_RECOVERY_MODE_ACTIVATED`. Si un admin desactiva el modo recuperación se registra `UPDATE_RECOVERY_MODE_CLEARED`. Mientras el modo recuperación está activo, no se pueden usar Buscar actualización / Descargar / Instalar hasta desactivarlo.

## Configuración de actualizaciones (solo admin)

En Mantenedores hay una sección **Configuración de actualizaciones** que lee/escribe en la tabla `settings` del backend:

- **updates.enabled** — Activar o desactivar la búsqueda e instalación de actualizaciones.
- **updates.channel** — `stable` o `beta`.
- **updates.autoCheck** — Comprobar actualizaciones en segundo plano con un intervalo.
- **updates.checkIntervalMinutes** — Intervalo en minutos (p. ej. 720 = 12 h). Máx. 10080 (7 días).
- **updates.endpointOverride** — URL opcional del feed de actualizaciones. Si está vacío se usa el endpoint por defecto (GitHub Releases).

La instalación siempre es **manual** por el admin; el autoCheck solo informa si hay actualización disponible.

API: `GET /api/admin/settings/updates` y `PUT /api/admin/settings/updates` (requieren sesión admin).

## Modo seguro / crash-loop tras actualización

Si la app se reinicia varias veces seguidas tras instalar una actualización (p. ej. 3 reinicios), se activa el **modo recuperación**:

- Se desactiva conceptualmente el flujo de actualización automático hasta que un admin lo desactive.
- En la sección Actualizaciones se muestra un banner **Modo recuperación** con un botón **Desactivar modo recuperación**.
- El estado se guarda en un archivo en el directorio de datos de la app (`app_data_dir/rockola-update-recovery.json`).
- Tras unos segundos de ejecución estable (p. ej. 60 s), el contador de reinicios se limpia para no disparar recovery en el próximo arranque normal.

Constantes en Rust: `CRASH_LOOP_THRESHOLD = 3`, `CLEAR_RESTARTS_AFTER_SECS = 60`.

## Cómo probar el flujo (Mantenedores y modo recuperación)

### 1. Arrancar backend y app

En una terminal:

```bash
cd backend && cargo run
```

En otra:

```bash
cd frontend && npm run tauri dev
```

(Asegúrate de tener `ADMIN_PIN` en `.env` o en el entorno si el backend exige PIN para admin.)

### 2. Probar en la app Tauri

1. Abre la app (se abrirá sola con `tauri dev`).
2. Ve a **Admin** → **Mantenedores** e inicia sesión con el PIN.
3. Revisa **Configuración de actualizaciones**: cambia canal, intervalo o “Actualizaciones habilitadas” y pulsa **Guardar configuración**.
4. En **Actualizaciones** verás la versión actual. Si el endpoint no está configurado o no hay release, “Buscar actualización” puede devolver error (es esperado si no hay `latest.json` publicado).
5. En navegador (sin Tauri) la sección Actualizaciones mostrará “Solo disponible en la app de escritorio”.

### 3. Simular modo recuperación para probar el banner

Para ver el banner **Modo recuperación** y el botón **Desactivar modo recuperación** sin hacer 3 reinicios reales:

1. Cierra la app Tauri si está abierta.
2. Crea el directorio de datos de la app y el archivo de estado (en Linux suele ser `~/.local/share/digital.rockola.app/`):

   ```bash
   mkdir -p ~/.local/share/digital.rockola.app
   echo '{"restarts_after_update":0,"recovery_mode":true}' > ~/.local/share/digital.rockola.app/rockola-update-recovery.json
   ```

3. Vuelve a abrir la app (`npm run tauri dev`) y entra en Mantenedores → Actualizaciones.
4. Deberías ver el banner ámbar de modo recuperación y el mensaje de que no puedes buscar actualizaciones hasta desactivarlo.
5. Pulsa **Desactivar modo recuperación**: el banner desaparece y vuelven a mostrarse “Buscar actualización” y el resto.
6. En el **Registro de auditoría** debería aparecer `UPDATE_RECOVERY_MODE_CLEARED`.

En macOS el directorio suele ser `~/Library/Application Support/digital.rockola.app/`; en Windows, `%APPDATA%\digital.rockola.app\`.

## Troubleshooting

- **"No hay endpoints de actualización configurados"**: Las URLs en `endpoints` deben parsear como `Url`. Revisa que no estén vacías y que el formato sea correcto.
- **Firma inválida:** El contenido de cada `.sig` debe coincidir con el instalador que se descarga; la clave pública en la app debe ser la pareja de la privada con la que se firmó.
- **404 al comprobar actualización:** El endpoint debe ser accesible desde el equipo donde corre la app (y devolver 200 con el JSON correcto).
- **La app no reinicia tras instalar:** En Windows el instalador suele cerrar la app; en Linux/macOS se llama a `relaunch()` del plugin process. Asegúrate de tener el permiso `process:allow-restart` en la capability de escritorio.

## Scripts opcionales (CI)

Ejemplo de pasos para GitHub Actions:

1. Checkout, setup Node y Rust.
2. Cargar la clave privada desde un secret.
3. `npm run build` y `npm run tauri build`.
4. Subir los artefactos de `target/release/bundle/**` a la release (por ejemplo con `softprops/action-gh-release`).
5. Generar `latest.json` con las URLs de los artefactos y el contenido de cada `.sig`, y subirlo como asset del release.

No incluyas la clave privada en el repo ni en logs.
