#!/usr/bin/env bash
# Fuerza recompilar y ejecutar el backend desde esta carpeta.
# Así te aseguras de tener la ruta GET /api/maintenance (vista mantenedores).
set -e
cd "$(dirname "$0")"
echo "Compilando backend..."
cargo build
echo "Arrancando backend (Ctrl+C para parar)..."
echo "Si falla con 'Address already in use', para el otro proceso en el puerto 3000: pkill -f rockola-backend"
exec cargo run
