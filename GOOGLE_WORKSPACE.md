# Google Workspace API - ZeroClaw Integration

> Guía completa de la integración de Google Workspace en ZeroClaw

## Tabla de Contenidos

1. [概述](#概述)
2. [Configuración](#configuración)
3. [Servicios Implementados](#servicios-implementados)
4. [Referencia de API](#referencia-de-api)
5. [Ejemplos de Uso](#ejemplos-de-uso)
6. [Autenticación OAuth](#autenticación-oauth)
7. [Solución de Problemas](#solución-de-problemas)

---

## 概述

ZeroClaw integra con las APIs de Google Workspace usando OAuth 2.0 para cuentas personales (@gmail.com). Soporta:

- **Gmail**: Leer, enviar, crear borradores, eliminar correos
- **Drive**: Listar, crear, actualizar, eliminar archivos y carpetas
- **Calendar**: Crear, modificar, eliminar eventos
- **Docs**: Crear y editar documentos de Google Docs
- **Sheets**: Crear y escribir en hojas de cálculo
- **Slides**: Crear presentaciones
- **Chat**: Enviar mensajes a Google Chat

---

## Configuración

### Variables de Entorno Requeridas

```env
GOOGLE_REFRESH_TOKEN=your_refresh_token
GOOGLE_CLIENT_ID=your_client_id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your_client_secret
```

### Cómo Obtener Credenciales

1. Ve a [Google Cloud Console](https://console.cloud.google.com)
2. Crea un nuevo proyecto
3. Habilita las APIs necesarias:
   - Gmail API
   - Google Drive API
   - Google Calendar API
   - Google Docs API
   - Google Sheets API
   - Google Slides API
   - Google Chat API
4. Ve a **Credentials** → **OAuth 2.0 Client ID**
5. Configura URIs de redirects
6. Descarga el archivo JSON de credenciales

Usa el script incluido para generar el refresh token:
```bash
python3 scripts/get_google_refresh_token.py
```

---

## Servicios Implementados

| Servicio | Acciones |
|----------|----------|
| **gmail** | list, get, send, draft, delete, attachments |
| **drive** | list, get, create, update, delete |
| **calendar** | list, get, create, update, delete |
| **docs** | list, get, create, update, delete |
| **sheets** | list, get, create, update, append, delete |
| **slides** | create, get, add_slide |
| **chat** | send, list |

---

## Referencia de API

### Parámetros Comunes

| Parámetro | Tipo | Descripción |
|-----------|------|-------------|
| `name` | string | Nombre del recurso |
| `title` | string | Título (alias de name) |
| `content` | string | Contenido del archivo |
| `body` | string | Alias de content |
| `type` | string | Tipo (ej: "folder") |
| `mimeType` | string | MIME type de Google |
| `parent` | string | ID de carpeta padre |
| `folderId` | string | ID de carpeta |
| `range` | string | Rango de celdas (Sheets) |
| `values` | array | Datos (array de arrays) |
| `query` | string | Búsqueda |
| `max_results` | integer | Límite de resultados |
| `pageToken` | string | Token de paginación |
| `id` | string | ID del recurso |
| `to` | string | Destinatario (Gmail) |
| `subject` | string | Asunto (Gmail) |
| `space` | string | Espacio (Chat) |
| `message` | string | Mensaje (Chat) |
| `layout` | string | Layout (Slides) |

---

### Gmail API

#### list
Listar correos del usuario.

```json
{"service": "gmail", "action": "list", "params": {"max_results": 10}}
```

#### get
Obtener detalles de un correo específico.

```json
{"service": "gmail", "action": "get", "params": {"id": "MESSAGE_ID"}}
```

#### send
Enviar correo.

```json
{"service": "gmail", "action": "send", "params": {"to": "dest@email.com", "subject": "Asunto", "body": "Mensaje"}}
```

#### draft / create
Crear borrador.

```json
{"service": "gmail", "action": "draft", "params": {"to": "dest@email.com", "subject": "Asunto", "body": "Mensaje"}}
```

#### delete
Eliminar correo.

```json
{"service": "gmail", "action": "delete", "params": {"id": "MESSAGE_ID"}}
```

#### attachments
Obtener adjuntos.

```json
{"service": "gmail", "action": "attachments", "params": {"id": "MESSAGE_ID"}}
```

---

### Drive API

#### list
Listar archivos.

```json
{"service": "drive", "action": "list", "params": {"max_results": 100, "query": "name contains 'test'"}}
```

```json
{"service": "drive", "action": "list", "params": {"folderId": "FOLDER_ID"}}
```

#### get
Obtener detalles de archivo.

```json
{"service": "drive", "action": "get", "params": {"id": "FILE_ID"}}
```

#### create
Crear archivo o carpeta.

```json
{"service": "drive", "action": "create", "params": {"name": "MiArchivo.txt", "content": "Contenido"}}
```

```json
{"service": "drive", "action": "create", "params": {"name": "MiCarpeta", "type": "folder"}}
```

#### update
Actualizar contenido.

```json
{"service": "drive", "action": "update", "params": {"id": "FILE_ID", "content": "Nuevo contenido"}}
```

#### delete
Eliminar archivo.

```json
{"service": "drive", "action": "delete", "params": {"id": "FILE_ID"}}
```

---

### Calendar API

#### list
Listar eventos.

```json
{"service": "calendar", "action": "list", "params": {"max_results": 10}}
```

#### get
Obtener evento.

```json
{"service": "calendar", "action": "get", "params": {"id": "EVENT_ID"}}
```

#### create
Crear evento.

```json
{"service": "calendar", "action": "create", "params": {"summary": "Reunión", "description": "Notas", "start_time": "2024-01-15T10:00:00Z", "end_time": "2024-01-15T11:00:00Z"}}
```

#### update
Actualizar evento.

```json
{"service": "calendar", "action": "update", "params": {"id": "EVENT_ID", "summary": "Nuevo título"}}
```

#### delete
Eliminar evento.

```json
{"service": "calendar", "action": "delete", "params": {"id": "EVENT_ID"}}
```

---

### Docs API

#### list
Listar documentos.

```json
{"service": "docs", "action": "list", "params": {"max_results": 10}}
```

#### get
Obtener documento.

```json
{"service": "docs", "action": "get", "params": {"id": "DOC_ID"}}
```

#### create
Crear documento con contenido.

```json
{"service": "docs", "action": "create", "params": {"title": "Mi Documento", "content": "Contenido inicial"}}
```

#### update
Agregar contenido.

```json
{"service": "docs", "action": "update", "params": {"id": "DOC_ID", "content": "Nuevo contenido"}}
```

#### delete
Eliminar documento.

```json
{"service": "docs", "action": "delete", "params": {"id": "DOC_ID"}}
```

---

### Sheets API

#### list
Listar hojas de cálculo.

```json
{"service": "sheets", "action": "list", "params": {"max_results": 10}}
```

#### get
Obtener datos.

```json
{"service": "sheets", "action": "get", "params": {"id": "SPREADSHEET_ID", "range": "Sheet1!A1:Z100"}}
```

#### create
Crear nueva hoja de cálculo.

```json
{"service": "sheets", "action": "create", "params": {"name": "Mis Datos"}}
```

#### update
Actualizar celdas.

```json
{"service": "sheets", "action": "update", "params": {"id": "SPREADSHEET_ID", "range": "Sheet1!A1", "values": [["Valor1", "Valor2"]]}}
```

#### append
Agregar filas.

```json
{"service": "sheets", "action": "append", "params": {"id": "SPREADSHEET_ID", "range": "Sheet1!A1", "values": [["NuevaFila"]]}}
```

#### delete
Eliminar hoja de cálculo.

```json
{"service": "sheets", "action": "delete", "params": {"id": "SPREADSHEET_ID"}}
```

---

### Slides API

#### create
Crear presentación.

```json
{"service": "slides", "action": "create", "params": {"title": "Mi Presentación"}}
```

#### get
Obtener presentación.

```json
{"service": "slides", "action": "get", "params": {"id": "PRESENTATION_ID"}}
```

#### add_slide
Agregar slide.

```json
{"service": "slides", "action": "add_slide", "params": {"id": "PRESENTATION_ID", "layout": "TITLE_AND_BODY"}}
```

---

### Chat API

#### send
Enviar mensaje.

```json
{"service": "chat", "action": "send", "params": {"space": "spaces/AAA...", "message": "Hola!"}}
```

#### list
Listar espacios.

```json
{"service": "chat", "action": "list", "params": {"max_results": 10}}
```

---

## Ejemplos de Uso

### Crear carpeta en Drive y documento dentro

1. Crear carpeta:
```json
{"service": "drive", "action": "create", "params": {"name": "Proyecto2024", "type": "folder"}}
```

2. Crear documento dentro:
```json
{"service": "docs", "action": "create", "params": {"title": "Notas", "content": "Contenido inicial"}}
```

### Crear hoja de cálculo con datos

```json
{
  "service": "sheets",
  "action": "create",
  "params": {"name": "Inventario"}
}
```

Luego agregar datos:
```json
{
  "service": "sheets",
  "action": "append",
  "params": {
    "id": "SPREADSHEET_ID",
    "range": "Sheet1!A1",
    "values": [["Producto", "Cantidad", "Precio"], ["Laptop", "5", "999"]]
  }
}
```

### Crear evento de calendario

```json
{
  "service": "calendar",
  "action": "create",
  "params": {
    "summary": "Reunión de equipo",
    "description": "Discuss project updates",
    "start_time": "2024-01-20T14:00:00Z",
    "end_time": "2024-01-20T15:00:00Z"
  }
}
```

---

## Autenticación OAuth

### Scopes Requeridos

El refresh token debe incluir estos scopes:
- `https://www.googleapis.com/auth/gmail.readonly`
- `https://www.googleapis.com/auth/gmail.send`
- `https://www.googleapis.com/auth/gmail.compose`
- `https://www.googleapis.com/auth/drive`
- `https://www.googleapis.com/auth/drive.file`
- `https://www.googleapis.com/auth/calendar`
- `https://www.googleapis.com/auth/documents`
- `https://www.googleapis.com/auth/spreadsheets`
- `https://www.googleapis.com/auth/presentations`

### Generar Refresh Token

Usa el script incluido:
```bash
python3 scripts/get_google_refresh_token.py
```

Sigue las instrucciones del navegador para autorizar la aplicación.

---

## Solución de Problemas

### Error 403: Insufficient authentication scopes

**Solución**: Regenerar el refresh token con todos los scopes necesarios.

### Error 401: Invalid credentials

**Solución**: Verificar que `GOOGLE_REFRESH_TOKEN`, `GOOGLE_CLIENT_ID` y `GOOGLE_CLIENT_SECRET` sean correctos.

### Error: Resource not found

**Solución**: Verificar que el ID del recurso sea correcto. Usar acción `list` para obtener IDs válidos.

### Timeout en operaciones grandes

**Solución**: Usar `max_results` menor para listas grandes, o paginación con `pageToken`.

---

## Links de Referencia

- [Google APIs Explorer](https://developers.google.com/apis-explorer)
- [Gmail API Reference](https://developers.google.com/gmail/api/v1/reference)
- [Drive API Reference](https://developers.google.com/drive/api/v3/reference)
- [Calendar API Reference](https://developers.google.com/calendar/api/v3/reference)
- [Sheets API Reference](https://developers.google.com/sheets/api/reference)