# Panel de mantenedor (admin)

El panel de mantenedor permite gestionar la rockola en modo kiosko: ver la cola, la biblioteca, la cola de descargas, resetear datos y consultar el registro de auditoría. Opcionalmente se protege con **PIN** mediante la variable de entorno `ADMIN_PIN`.

## Configuración

### Variable `ADMIN_PIN`

- **Si no se define** (o está vacía): los endpoints de admin **no exigen autenticación**. Cualquiera puede llamar a `/api/maintenance`, `/api/admin/reset`, `/api/downloads`, etc. Útil para desarrollo o entornos de confianza.
- **Si se define** (p. ej. `ADMIN_PIN=1234`): para acceder a los endpoints protegidos hay que **iniciar sesión** con ese PIN y enviar el token en el header `Authorization: Bearer <token>`.

Ejemplo en `.env` del backend:

```bash
ADMIN_PIN=1234
```

## Flujo de autenticación

1. **Login:** `POST /api/admin/login` con body `{ "pin": "1234" }`.  
   - Si el PIN es correcto, el servidor devuelve `{ "token": "<uuid>", "expiresInSecs": 900 }`.  
   - La sesión dura **15 minutos**.  
   - Tras **5 intentos fallidos**, el login queda bloqueado **5 minutos** (429 Too Many Requests).

2. **Uso del token:** En todas las peticiones a endpoints protegidos se envía el header:
   ```http
   Authorization: Bearer <token>
   ```

3. **Comprobar sesión:** `GET /api/admin/session` con ese header. Responde `{ "valid": true }` o 401 si el token no es válido o ha expirado.

4. **Logout:** `POST /api/admin/logout` con el mismo header. Invalida la sesión en el servidor. El cliente debe borrar el token (p. ej. en la vista mantenedores, “Cerrar sesión”).

## Endpoints protegidos (cuando `ADMIN_PIN` está definido)

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/api/maintenance` | Cola, biblioteca reciente, créditos y conteos |
| POST | `/api/admin/reset` | Borrar cola, caché, biblioteca y resetear créditos |
| GET | `/api/downloads` | Lista de jobs de descarga |
| POST | `/api/downloads/:id/retry` | Reintentar un job fallido |
| GET | `/api/admin/audit-log` | Registro de auditoría |

Sin token (o con token inválido/expirado) estos endpoints responden **401 Unauthorized**.

## Registro de auditoría

Cada acción relevante se registra en la tabla `admin_audit_log`:

| Acción | Cuándo |
|--------|--------|
| `admin.login` | Login correcto |
| `admin.logout` | Logout (token enviado) |
| `admin.reset_all` | Se ejecutó el reset de datos |

El frontend muestra las últimas entradas en la sección **Registro de auditoría** de la vista mantenedores (cuando hay sesión).

## Uso en el frontend

1. **Vista Mantenedores** (`/mantenedores`):  
   - Si el backend devuelve **401** al cargar mantenimiento (porque `ADMIN_PIN` está definido y no hay token), se muestra un **formulario de login** (campo PIN).  
   - Al introducir el PIN correcto se guarda el token en **sessionStorage** y se recargan cola, biblioteca, **Cola de descargas** y **Registro de auditoría**.  
   - El botón **Cerrar sesión** borra el token y vuelve a pedir PIN en la siguiente carga.

2. **Admin básico** (`/admin`):  
   - El botón “Borrar data y resetear créditos” usa el token del store si existe.  
   - Si el backend responde 401, se muestra un mensaje indicando que hay que iniciar sesión en Vista mantenedores.

3. **Store:** `useAdminStore` (`stores/adminStore.ts`) guarda el token y lo persiste en `sessionStorage` para que la sesión sobreviva a recargas de la página.

## Resumen de seguridad

- El PIN se compara en texto plano en el backend (no se almacena hash). Para entornos más exigentes se podría sustituir por hash + salt.
- El token es un UUID v4; las sesiones están en memoria (se pierden al reiniciar el backend).
- Rate limiting: 5 intentos fallidos → bloqueo 5 minutos por instancia del backend.
