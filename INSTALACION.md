# Instalación de Rockola

Para ejecutar el proyecto Rockola, necesitas instalar Rust y sus dependencias. Sigue estos pasos:

## 1. Instalar Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

O visita https://www.rust-lang.org/tools/install para obtener instrucciones alternativas.

Después de instalar Rust, reinicia tu terminal o ejecuta:
```bash
source ~/.cargo/env
```

## 2. Instalar dependencias de sistema

Para compilar aplicaciones Tauri, necesitas instalar algunas dependencias del sistema:

### Ubuntu/Debian:
```bash
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Fedora:
```bash
sudo dnf install gcc \
    gcc-c++ \
    cmake \
    webkit2gtk3-devel \
    gtk3-devel \
    libappindicator-devel \
    librsvg2-devel \
    openssl-devel
```

### Arch Linux:
```bash
sudo pacman -S webkit2gtk \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    openssl
```

## 3. Instalar Tauri CLI

```bash
cargo install tauri-cli
```

## 4. Instalar Node.js y npm

Rockola también necesita Node.js para el frontend:

### Opción 1: Usando nvm (recomendado)
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install node
nvm use node
```

### Opción 2: Instalar directamente
Visita https://nodejs.org/ y descarga la versión LTS.

## 5. Instalar dependencias del frontend

```bash
cd ui
npm install
```

## 6. Ejecutar el proyecto

### Para desarrollar:
```bash
# En una terminal, iniciar el servidor de desarrollo del frontend
cd ui
npm run dev

# En otra terminal, iniciar la aplicación Tauri
cd apps/desktop
cargo tauri dev
```

### Para compilar para producción:
```bash
# Compilar el frontend
cd ui
npm run build

# Compilar la aplicación Tauri
cd apps/desktop
cargo tauri build
```

## 7. Solución de problemas comunes

### Error de permisos
Si tienes problemas de permisos con cargo, asegúrate de que ~/.cargo/bin esté en tu PATH:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Error de compilación en Linux
Si recibes errores relacionados con bibliotecas faltantes, asegúrate de haber instalado todas las dependencias del sistema mencionadas arriba.

### Problemas con webkit2gtk
En algunas distribuciones, puedes necesitar una versión diferente:
```bash
# Para Ubuntu 22.04+
sudo apt install libwebkit2gtk-4.1-dev
```

## 8. Verificación de instalación

Para verificar que todo esté correctamente instalado:

```bash
rustc --version
cargo --version
node --version
npm --version
cargo tauri info
```

Una vez que tengas todo instalado, podrás ejecutar el proyecto Rockola.