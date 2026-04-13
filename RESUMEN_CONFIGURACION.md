# ZeroClaw - Resumen de Configuración y Google Workspace Integration

## Estado Actual

El proyecto **ZeroClaw** está configurado y funcionando localmente con Docker, integrando:
- Telegram como canal de comunicación
- Google Workspace (Gmail, Drive, Calendar, Docs, Sheets, Slides, Chat)
- OpenRouter como provider de LLM
- GitHub y Notion

---

## Problemas Resueltos

### 1. Build de Docker fallaba

**Problema**: Errores de compilación en `google_workspace.rs`:
- `missing field google_workspace in initializer of Config`
- Type errors con `.ok_or()` y `.ok_or_else()`
- Errores con `format!()` vs `anyhow::anyhow!()` en contexto async

**Solución**:
- Agregar `google_workspace: GoogleWorkspaceConfig::default()` en todos los inicializadores de Config
- Cambiar `.ok_or("msg")` por `.ok_or_else(|| anyhow::anyhow!("msg"))`
- Cambiar `format!()` por `anyhow::anyhow!()` en funciones async
- Usar `.json::<serde_json::Value>()` para inferencia de tipos explícita

### 2. Dashboard no cargaba

**Problema**: Frontend no incluido en la imagen Docker

**Solución**:
```bash
cd web && npm install && npm run build
```

### 3. API key de OpenRouter no reconocida

**Problema**: La variable `API_KEY` no era reconocida

**Solución**: Cambiar a `OPENROUTER_API_KEY` en `.env`

### 4. Telegram no se iniciaba

**Problema**: 
- Config incorrecta en Dockerfile (`[channels.telegram]` vs `[channels_config.telegram]`)
- Faltaban campos requeridos
- No se iniciaban los canales automáticamente

**Solución**:
- Corregir config a `[channels_config.telegram]`
- Agregar `ENV ZEROCLAW_AUTOSTART_CHANNELS=1`
- Cambiar CMD de `gateway` a `daemon`

### 5. Google Workspace no cargaba credenciales

**Problema**: Las variables de entorno no se cargaban desde `--env-file`

**Solución**: Agregar debugging logs en `load_oauth_from_env()`

### 6. Scopes insuficientes para Gmail

**Problema**: Error 403 "Insufficient authentication scopes"

**Solución**: Actualizar script para incluir scope `gmail.compose`

### 7. Drive create no soportaba contenido

**Problema**: Solo creaba archivos vacíos

**Solución**: Actualizar para soportar carpetas y archivos con contenido

---

## Archivos Modificados

| Archivo | Cambios |
|---------|---------|
| `src/tools/google_workspace.rs` | 34 handlers implementados |
| `src/config/schema.rs` | Agregado GoogleWorkspaceConfig |
| `src/onboard/wizard.rs` | Agregado google_workspace field |
| `src/tools/mod.rs` | Configuración de tools |
| `Dockerfile` | Web assets, canales, ENV vars, CMD daemon |
| `.env` | OPENROUTER_API_KEY, GOOGLE_REFRESH_TOKEN/CLIENT_ID/CLIENT_SECRET |
| `scripts/get_google_refresh_token.py` | Scopes actualizados |

---

## API Reference - Google Workspace

### Servicios y Acciones (34 handlers)

| Servicio | Acciones Disponibles |
|----------|----------------------|
| **gmail** | list, get, send, draft, delete, attachments |
| **drive** | list, get, create, update, delete |
| **calendar** | list, get, create, update, delete |
| **docs** | list, get, create, update, delete |
| **sheets** | list, get, create, update, append, delete |
| **slides** | create, get, add_slide |
| **chat** | send, list |

### Parámetros Soportados

| Parámetro | Tipo | Descripción |
|-----------|------|--------------|
| `name` | string | Nombre del archivo/documento |
| `title` | string | Título (alias de name) |
| `content` | string | Contenido del archivo |
| `body` | string | Alias de content |
| `type` | string | Tipo (ej: "folder") |
| `mimeType` | string | MIME type de Google |
| `parent` | string | ID de carpeta padre |
| `folderId` | string | ID de carpeta (para listar contenido) |
| `range` | string | Rango de celdas (Sheets) |
| `values` | array | Datos para Sheets (array de arrays) |
| `query` | string | Búsqueda |
| `max_results` | integer | Límite de resultados |
| `pageToken` | string | Token de paginación |
| `id` | string | ID del recurso |
| `to` | string | Destinatario (Gmail) |
| `subject` | string | Asunto (Gmail) |
| `space` | string | Espacio (Google Chat) |
| `message` | string | Mensaje (Google Chat) |
| `layout` | string | Layout (Slides) |

---

## Ejemplos de Uso

### Drive - Crear carpeta
```
{"service": "drive", "action": "create", "params": {"name": "MiCarpeta", "type": "folder"}}
```

### Drive - Crear archivo con contenido
```
{"service": "drive", "action": "create", "params": {"name": "documento.txt", "content": "Hola mundo", "parent": "CARPETA_ID"}}
```

### Docs - Crear documento con contenido
```
{"service": "docs", "action": "create", "params": {"title": "Mi Documento", "content": "Contenido inicial"}}
```

### Docs - Actualizar documento
```
{"service": "docs", "action": "update", "params": {"id": "DOC_ID", "content": "Nuevo contenido"}}
```

### Sheets - Crear spreadsheet
```
{"service": "sheets", "action": "create", "params": {"name": "Mis Datos"}}
```

### Sheets - Agregar datos
```
{"service": "sheets", "action": "append", "params": {"id": "SPREADSHEET_ID", "range": "Sheet1!A1", "values": [["Nombre", "Edad"], ["Juan", "30"]]}}
```

### Calendar - Crear evento
```
{"service": "calendar", "action": "create", "params": {"summary": "Reunión", "start_time": "2024-01-15T10:00:00Z", "end_time": "2024-01-15T11:00:00Z"}}
```

### Gmail - Enviar correo
```
{"service": "gmail", "action": "send", "params": {"to": "dest@email.com", "subject": "Asunto", "body": "Mensaje"}}
```

### Slides - Crear presentación
```
{"service": "slides", "action": "create", "params": {"title": "Mi Presentación"}}
```

### Slides - Agregar slide
```
{"service": "slides", "action": "add_slide", "params": {"id": "PRESENTATION_ID", "layout": "TITLE_AND_BODY"}}
```

---

## Cómo Ejecutar

### Local con Docker:
```bash
cd zeroclaw_render-free

# Build
docker build -t zeroclaw .

# Run
docker run -d -p 42617:42617 --name zeroclaw-test --env-file .env zeroclaw
```

### Variables de entorno (`.env`):
```env
OPENROUTER_API_KEY=sk-or-v1-...
DEFAULT_PROVIDER=openrouter
DEFAULT_MODEL=openrouter/free

TELEGRAM_BOT_TOKEN=...
TELEGRAM_ALLOWED_USERS=...

GOOGLE_REFRESH_TOKEN=...
GOOGLE_CLIENT_ID=...
GOOGLE_CLIENT_SECRET=...

GITHUB_TOKEN=...
NOTION_KEY=...
```

---

## Notas Importantes

1. **Scopes OAuth**: Para usar todas las funciones, el refresh token debe incluir `gmail.compose`, `drive`, `calendar`, `docs`, `spreadsheets`, `presentations`
2. **Dockerfile CMD**: Debe ser `daemon` no `gateway` para que inicien los canales
3. **Puertos**: Gateway escucha en 42617 (no 8080)
4. **Security**: No commitear `.env` ni credenciales - están en .gitignore

---

## Links de Referencia

- [OAuth Refresh Token Generator](./scripts/get_google_refresh_token.py)
- [API Reference](./GOOGLE_WORKSPACE.md)
- [Deploy Guide](./RENDER_DEPLOY_GUIDE.md)