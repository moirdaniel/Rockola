#!/usr/bin/env bash
# Levanta Backend (puerto 3000) y Frontend (puerto 5173) desde la raíz del repo.
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
BACKEND_PID=""

cleanup() {
  if [[ -n "$BACKEND_PID" ]]; then
    echo ""
    echo "Cerrando backend (PID $BACKEND_PID)..."
    kill "$BACKEND_PID" 2>/dev/null || true
  fi
  exit 0
}
trap cleanup SIGINT SIGTERM

echo "=== Backend ==="
cd "$ROOT/backend"
mkdir -p data
[[ -f .env ]] || cp .env.example .env 2>/dev/null || true
cargo build -q 2>/dev/null || cargo build
cargo run &
BACKEND_PID=$!
cd "$ROOT"

echo "Esperando a que el backend escuche en :3000..."
for i in {1..30}; do
  if curl -s -o /dev/null http://127.0.0.1:3000/health 2>/dev/null; then
    echo "Backend listo (http://localhost:3000)."
    break
  fi
  sleep 0.5
done
if ! curl -s -o /dev/null http://127.0.0.1:3000/health 2>/dev/null; then
  echo "No se pudo conectar al backend en :3000. Revisa la salida del backend."
fi

echo ""
echo "=== Frontend ==="
cd "$ROOT/frontend"
npm run dev
