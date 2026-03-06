# TODOs - Siguientes fases

## Fase 2 - Fuentes reales

- [ ] Implementar adapter YouTube con API oficial (o proxy)
- [ ] Implementar adapter Spotify con API oficial
- [ ] Adapter música local: escaneo de carpetas y servir archivos
- [ ] Backend: proxy de stream para YouTube/Spotify (evitar CORS y exponer URLs)

## Fase 3 - Usuarios y auth

- [ ] Modelo de usuarios en BD
- [ ] Autenticación (JWT o sesiones)
- [ ] Créditos por usuario (no solo `default`)
- [ ] Pantalla de login/registro

## Fase 4 - Playlists y UX

- [ ] CRUD de playlists en backend y frontend
- [ ] Añadir playlist a la cola
- [ ] Favoritos por usuario
- [ ] Modo pantalla completa real para video
- [ ] Controles de volumen en UI

## Fase 5 - Producción

- [ ] Tests unitarios (backend y frontend)
- [ ] Tests E2E (Playwright o similar)
- [ ] CI/CD (GitHub Actions o similar)
- [ ] PWA y service worker para uso offline parcial
- [ ] Métricas y logging centralizado

## Fase 6 - Escalabilidad

- [ ] Opción de PostgreSQL para multi-instancia
- [ ] Redis para cola en memoria (opcional)
- [ ] Rate limiting y validación de entrada reforzada
