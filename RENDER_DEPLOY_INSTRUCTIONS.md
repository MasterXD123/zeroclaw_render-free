# Instructivo para Deploy en Render (ZeroClaw-render-free)

## Requisitos Previos

| Cuenta | Enlace | Notas |
|--------|--------|-------|
| GitHub | github.com | Tu repositorio ya está conectado |
| Render | render.com | Cuenta gratuita |
| OpenRouter | openrouter.ai | $5 gratis/mes |

---

## Paso 1: Preparar tu repositorio

El código ya está en GitHub: `https://github.com/MasterXD123/zeroclaw_render-free`

---

## Paso 2: Crear Web Service en Render

1. Ve a [Render Dashboard](https://dashboard.render.com)
2. Click **"New"** → **"Web Service"**
3. En "Public Git repository", ingresa:
   ```
   https://github.com/MasterXD123/zeroclaw_render-free
   ```
4. Click **"Connect"**

---

## Paso 3: Configurar el Service

| Campo | Valor |
|-------|-------|
| **Name** | `zeroclaw` (o el nombre que prefieras) |
| **Region** | Frankfurt (o el más cercano) |
| **Branch** | `master` |
| **Build Command** | `docker build -t zeroclaw -f Dockerfile.render .` |
| **Start Command** | `docker run -p $PORT:$PORT -e OPENROUTER_API_KEY zeroclaw` |

---

## Paso 4: Environment Variables

### Obligatorias:

| Key | Value |
|-----|-------|
| `OPENROUTER_API_KEY` | `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` |

### Básicas:

| Key | Value | Descripción |
|-----|-------|-------------|
| `PROVIDER` | `openrouter` | Proveedor de modelos |
| `ZEROCLAW_MODEL` | `openrouter/free` | Modelo a usar |
| `ZEROCLAW_ALLOW_PUBLIC_BIND` | `true` | Permitir acceso público |

### Opcionales:

| Key | Value | Descripción |
|-----|-------|-------------|
| `RUST_LOG` | `info` | Nivel de logs (error, warn, info, debug) |
| `RUST_BACKTRACE` | `1` | Habilitar backtraces |

---

## Variables de Entorno Completas

### Para Modelos (IA):

| Variable | Ejemplo | Descripción |
|----------|---------|-------------|
| `OPENROUTER_API_KEY` | `sk-or-v1-...` | **Tu API key de OpenRouter** (obligatoria) |
| `PROVIDER` | `openrouter` | Proveedor: openrouter, openai, anthropic, google, etc. |
| `ZEROCLAW_MODEL` | `openrouter/free` | Modelo específico |

### Para Gateway:

| Variable | Default | Descripción |
|----------|---------|-------------|
| `ZEROCLAW_GATEWAY_PORT` | `10000` | Puerto del servidor |
| `ZEROCLAW_GATEWAY_HOST` | `[::]` | Host del servidor |
| `ZEROCLAW_ALLOW_PUBLIC_BIND` | `false` | Permitir acceso público |

### Para Telegram (opcional):

| Variable | Descripción | Cómo obtenerlo |
|----------|-------------|----------------|
| `TELEGRAM_BOT_TOKEN` | Token del bot | @BotFather en Telegram |
| `TELEGRAM_ALLOWED_USERS` | IDs autorizados (coma-separated) | @userinfobot |

### Para Memoria:

| Variable | Default | Descripción |
|----------|---------|-------------|
| `ZEROCLAW_STORAGE_PROVIDER` | `sqlite` | Backend: sqlite, memory, postgres, redis |
| `ZEROCLAW_STORAGE_DB_URL` | (auto) | URL de base de datos externa |

### Para Búsqueda Web:

| Variable | Descripción |
|----------|-------------|
| `BRAVE_API_KEY` | API key de Brave Search (para web search) |

### Para Otros Proveedores:

| Variable | Para qué sirve |
|----------|----------------|
| `ANTHROPIC_API_KEY` | Claude API |
| `GEMINI_API_KEY` | Google Gemini API |
| `OPENAI_API_KEY` | OpenAI API |
| `GITHUB_TOKEN` | GitHub API token |
| `NOTION_KEY` | Notion API key |

### Para Skills:

| Variable | Descripción |
|----------|-------------|
| `ZEROCLAW_OPEN_SKILLS_ENABLED` | Habilitar open-skills (true/false) |
| `ZEROCLAW_OPEN_SKILLS_DIR` | Directorio de skills externo |

### Para Logs:

| Variable | Valores posibles |
|----------|-------------------|
| `RUST_LOG` | `error`, `warn`, `info`, `debug`, `trace` |

---

## Paso 5: Health Check

En la sección **"Health Check"**:

| Campo | Valor |
|-------|-------|
| **Path** | `/health` |
| **Port** | `$PORT` |
| **Interval** | 30 seconds |
| **Timeout** | 10 seconds |

---

## Paso 6: Desplegar

1. Click **"Create Web Service"**
2. Esperar ~5-10 minutos (primera build)
3. Verificar en **"Logs"** que no haya errores

---

## Paso 7: Verificar que funciona

```bash
# Reemplaza TU-SERVICIO con tu URL de Render
# Ejemplo: https://zeroclaw-abc123.onrender.com

# Health check
curl https://TU-SERVICIO.onrender.com/health

# Probar el agente
curl -X POST https://TU-SERVICIO.onrender.com/webhook \
  -H "Content-Type: application/json" \
  -d '{"message":"hola"}'

# Probar terminal
curl -X POST https://TU-SERVICIO.onrender.com/api/execute \
  -H "Content-Type: application/json" \
  -d '{"command":"pwd"}'
```

---

## URLs disponibles

| Página | URL |
|--------|-----|
| Dashboard | `https://TU-SERVICIO.onrender.com/` |
| Agent Chat | `https://TU-SERVICIO.onrender.com/agent` |
| Terminal | `https://TU-SERVICIO.onrender.com/terminal` |
| Config | `https://TU-SERVICIO.onrender.com/config` |
| Health | `https://TU-SERVICIO.onrender.com/health` |

---

## Solución de Problemas

### El servicio no responde
- Verifica los logs en Render Dashboard
- Confirma que `OPENROUTER_API_KEY` está configurada correctamente

### Error "Port Already in Use"
- El puerto de Render puede haber cambiado. Revisa la variable `$PORT`

### El servicio está "Sleeping"
- Es normal en free tier después de 15 min inactividad
- Se despierta automáticamente cuando recibe una request

---

## Mantener el servicio activo (opcional)

El servicio duerme después de 15 min. Para mantenerlo activo:

### Opción 1: Google Apps Script (Gratis)
1. Ve a [script.google.com](https://script.google.com)
2. Crea un nuevo proyecto
3. Copia este código:

```javascript
function mantenerActivo() {
  var url = "https://TU-SERVICIO.onrender.com/health";
  UrlFetchApp.fetch(url);
}
```

4. Configura un trigger: cada 10 minutos
5. Despliega como "Aplicación web"

### Opción 2: UptimeRobot (Gratis)
1. Regístrate en [uptimerobot.com](https://uptimerobot.com)
2. Crea un monitor tipo "HTTP(s)"
3. Point a tu URL de Render
4. Configura interval: cada 5 minutos

---

## Notas Importantes

- ⚠️ **Cada redeploy borra la memoria y configuración**
- ⚠️ **El tier gratuito no tiene persistencia de datos**
- ⚠️ **El servicio duerme después de 15 min de inactividad**
- ✅ **El modelo por defecto es gratuito (openrouter/free)**

---

## ¿Necesitás ayuda adicional?

Revisá los logs en Render Dashboard o consultá el README.md del repositorio.