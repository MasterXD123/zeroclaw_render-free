# ZeroClaw-render-free

> ZeroClaw fork optimizado para deployment en **Render free tier**

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/MasterXD123/zeroclaw_render-free)

---

## Tabla de Contenidos

1. [Qué es ZeroClaw-render-free](#qué-es-zeroclaw-render-free)
2. [Requisitos Previos](#requisitos-previos)
3. [Configuración de OpenRouter](#configuración-de-openrouter)
4. [Deploy en Render - Paso a Paso](#deploy-en-render---paso-a-paso)
5. [Configuración de Variables de Entorno](#configuración-de-variables-de-entorno)
6. [Verificación del Deployment](#verificación-del-deployment)
7. [Cómo Usar ZeroClaw](#cómo-usar-zeroclaw)
8. [Solución de Problemas](#solución-de-problemas)
9. [Costos](#costos)
10. [Diferencias con el Fork Original](#diferencias-con-el-fork-original)
11. [Recursos Adicionales](#recursos-adicionales)

---

## Qué es ZeroClaw-render-free

Este es un fork de [ZeroClaw-free](https://github.com/MasterXD123/zeroclaw-free) específicamente optimizado para deploy en **Render free tier**.

### Características Principales

| Feature | Estado | Descripción |
|---------|--------|-------------|
| Gateway HTTP | ✅ | Interfaz web en puerto dinámico |
| SQLite Memory | ✅ | Memoria persistente por defecto |
| Skills System | ✅ | Sistema extensible de habilidades |
| Telegram Bot | ✅ | Bot de Telegram (opcional) |
| Cron Jobs | ✅ | Tareas programadas |
| Agent Loop | ✅ | Orquestación de agentes Rust |
| Agentic-Bridge | ✅ | Mejora de prompts |

### Diferencias vs Original

- **Sin Qdrant**: Usa SQLite como backend de memoria
- **Puerto dinámico**: Usa `$PORT` de Render en lugar de 42617
- **Skills adaptadas**: Sin comandos `docker exec`
- **Optimizado para Render**: Build y run commands específicos

### Limitaciones (Render Free Tier)

- ❌ Sin persistencia de datos entre redeploys
- ❌ Container duerme después de 15 min inactividad
- ❌ Sin acceso a Docker daemon interno

Ver [RENDER_DIFF.md](./RENDER_DIFF.md) para detalles completos.

---

## Requisitos Previos

### Cuentas Necesarias

| Servicio | Enlace | Costo |
|----------|--------|-------|
| GitHub | [github.com](https://github.com) | Gratis |
| Render | [render.com](https://render.com) | Gratis |
| OpenRouter | [openrouter.ai](https://openrouter.ai) | $5 gratis/mes |

### Conocimiento Básico
- Uso básico de terminal/línea de comandos
- Saber navegar GitHub y Render Dashboard

---

## Configuración de OpenRouter

### Paso 1: Crear Cuenta

1. Ve a [openrouter.ai](https://openrouter.ai)
2. Click **Sign Up** → usa tu cuenta Google o GitHub
3. Verifica tu email

### Paso 2: Obtener tu API Key

1. En OpenRouter, navega a **Keys** en el menú lateral
2. Click **Create New Key**
3. Nombre: `zeroclaw-render`
4. **Copia la key** (comienza con `sk-or-v1-xxxxxxxx`)
5. ⚠️ **Importante**: La key solo se muestra una vez

### Paso 3: Verificar Créditos

1. Ve a **Credits** en el menú
2. Deberías tener $5 gratis mensuales
3. Los créditos se renuevan automáticamente

### Modelos Recomendados

| Modelo | Calidad | Velocidad | Costo |
|--------|---------|-----------|-------|
| `openrouter/google/gemma-4-27b-it` | ⭐⭐⭐⭐⭐ | Media | Bajo |
| `openrouter/anthropic/claude-3-haiku` | ⭐⭐⭐⭐ | Rápida | Medio |
| `openrouter/mistral/mistral-7b-instruct` | ⭐⭐⭐ | Rápida | Muy bajo |

---

## Deploy en Render - Paso a Paso

### Paso 1: Preparar el Repositorio

Este repositorio ya está configurado para Render. Solo necesitas:

1. Fork de este repositorio o clonar y subir a tu GitHub
2. Conectar tu cuenta de GitHub con Render

### Paso 2: Crear Web Service en Render

1. Ve a [render.com](https://render.com) e inicia sesión
2. Click **"New"** → **"Web Service"**
3. Busca y conecta tu repositorio `zeroclaw_render-free`
4. Click **"Connect"**

### Paso 3: Configurar el Servicio

En la página de configuración:

| Campo | Valor |
|-------|-------|
| **Name** | `zeroclaw` |
| **Environment** | `Docker` |
| **Region** | `Frankfurt` (o la más cercana) |
| **Branch** | `master` |

### Paso 4: Configurar Build Command

En el campo **Build Command**:

```bash
docker build -t zeroclaw -f Dockerfile.render .
```

### Paso 5: Configurar Run Command

En el campo **Start Command**:

```bash
docker run -p $PORT:$PORT -e OPENAI_API_KEY -e PROVIDER -e ZEROCLAW_MODEL -e ZEROCLAW_ALLOW_PUBLIC_BIND zeroclaw
```

### Paso 6: Añadir Environment Variables

En la sección **Environment Variables**, añade:

| Key | Value | Descripción |
|-----|-------|-------------|
| `OPENAI_API_KEY` | `sk-or-v1-xxxxxxxxxxxxx` | **TU API KEY DE OPENROUTER** |
| `PROVIDER` | `openrouter` | Proveedor de modelos |
| `ZEROCLAW_MODEL` | `openrouter/google/gemma-4-27b-it` | Modelo a usar |
| `ZEROCLAW_ALLOW_PUBLIC_BIND` | `true` | Permitir acceso público |
| `RUST_LOG` | `info` | Nivel de logs |

**⚠️ IMPORTANTE**: Reemplaza `sk-or-v1-xxxxxxxxxxxxx` con tu API key real de OpenRouter.

### Paso 7: Desplegar

1. Click **"Create Web Service"**
2. Espera a que termine el build (~10-15 minutos la primera vez)
3. Verifica en los logs que no haya errores

---

## Configuración de Variables de Entorno

### Variables Obligatorias

| Variable | Ejemplo | Descripción |
|----------|---------|-------------|
| `OPENAI_API_KEY` | `sk-or-v1-aqui-va-tu-key...` | **Tu clave de OpenRouter** |
| `PROVIDER` | `openrouter` | Proveedor de IA |
| `ZEROCLAW_MODEL` | `openrouter/google/gemma-4-27b-it` | Modelo de IA |

### Variables Opcionales

| Variable | Default | Descripción |
|----------|---------|-------------|
| `ZEROCLAW_ALLOW_PUBLIC_BIND` | `false` | Permitir acceso público |
| `RUST_LOG` | `info` | Nivel de logs (error, warn, info, debug) |
| `RUST_BACKTRACE` | `1` | Habilitar backtraces |
| `TELEGRAM_BOT_TOKEN` | (vacío) | Token de bot de Telegram |
| `HOST_PORT` | `42617` | Puerto local (no necesario en Render) |

### Variables de Render (Automáticas)

| Variable | Descripción |
|----------|-------------|
| `PORT` | Puerto asignado por Render (ej: 10000) |
| `RENDER_SERVICE_ID` | ID del servicio |
| `RENDER_JOB_ID` | ID del job de build |

---

## Verificación del Deployment

### Método 1: Health Endpoint

```bash
curl https://tu-servicio.onrender.com/health
```

Debería responder:
```json
{"status":"ok","version":"..."}
```

### Método 2: Interfaz Web

1. Copia la URL de tu servicio (ej: `https://zeroclaw-abc123.onrender.com`)
2. Ábrela en tu navegador
3. Deberías ver la interfaz de ZeroClaw

### Método 3: API

```bash
# Ver estado general
curl https://tu-servicio.onrender.com/api/status

# Enviar mensaje
curl -X POST https://tu-servicio.onrender.com/api/message \
  -H "Content-Type: application/json" \
  -d '{"message": "hello"}'
```

---

## Cómo Usar ZeroClaw

### Interfaz Web

1. Accede a `https://tu-servicio.onrender.com`
2. Envía mensajes al agente
3. El agente ejecutará acciones y responderá

### Telegram (Opcional)

1. Obtén un token de @BotFather en Telegram
2. Añade `TELEGRAM_BOT_TOKEN` en las environment variables de Render
3. Busca tu bot en Telegram y envíale `/start`

### API REST

```bash
# Health check
curl https://tu-servicio.onrender.com/health

# Chat
curl -X POST https://tu-servicio.onrender.com/api/message \
  -H "Content-Type: application/json" \
  -d '{"message": "your message here"}'
```

---

## Solución de Problemas

### Error: "Invalid API Key"

**Causa**: Tu API key de OpenRouter es incorrecta o expiró

**Solución**:
1. Ve a [openrouter.ai/keys](https://openrouter.ai/keys)
2. Crea una nueva key si es necesario
3. Actualiza la variable `OPENAI_API_KEY` en Render

### Error: "Model Not Found"

**Causa**: El modelo especificado no existe

**Solución**: Usa uno de los modelos recomendados:
```bash
ZEROCLAW_MODEL=openrouter/anthropic/claude-3-haiku
```

### Error: "Port Already in Use"

**Causa**: Render asignó un puerto ya en uso

**Solución**: El comando ya usa `$PORT`, contacta a soporte de Render

### Servicio en Estado "Sleeping"

**Causa**: Render free tier pone a dormir después de 15 min inactividad

**Solución**: Envía una request y espera ~30 segundos, se despertará automáticamente

### Build Fallido

**Causa**: Error en el Dockerfile o dependencias

**Solución**:
1. Revisa los logs de build en Render Dashboard
2. Verifica que el branch sea `master`
3. Asegúrate que `Dockerfile.render` existe

### Sin Respuesta del Gateway

**Causa**: El servicio no inició correctamente

**Solución**:
1. Revisa los logs de runtime
2. Verifica que `OPENAI_API_KEY` esté configurada
3. Revisa que el modelo sea válido

---

## Costos

| Servicio | Costo | Notas |
|----------|-------|-------|
| **Render Free** | $0 | 750 horas/mes, duerme después de 15 min |
| **OpenRouter** | $0-5/mes | $5 gratis mensuales, solo se cobra si superas |
| **GitHub** | $0 | Gratis |
| **TOTAL** | **~$0-5/mes** | |

### Consejos para Ahorrar

1. Usa `claude-3-haiku` o `mistral-7b-instruct` (más económicos)
2. Monitorea uso en [openrouter.ai/credits](https://openrouter.ai/credits)
3. Configura alertas de uso en OpenRouter

---

## Diferencias con el Fork Original

Este fork difiere del original [zeroclaw-free](https://github.com/MasterXD123/zeroclaw-free) en:

| Aspecto | Original | Este Fork |
|---------|----------|-----------|
| Qdrant | ✅ Incluido | ❌ Eliminado |
| Memoria | qdrant o sqlite | **sqlite** (default) |
| Puerto | 42617 (hardcoded) | **$PORT** (dinámico) |
| Skills | docker exec | **adaptadas** (sin docker) |
| Dockerfile | Dockerfile | **Dockerfile.render** |
| Documentación | General | **Render-optimizada** |

Ver [RENDER_DIFF.md](./RENDER_DIFF.md) para lista completa.

---

## Recursos Adicionales

### Documentación
- [RENDER_DEPLOY_GUIDE.md](./RENDER_DEPLOY_GUIDE.md) - Guía detallada de deployment
- [RENDER_DIFF.md](./RENDER_DIFF.md) - Diferencias técnicas con el original
- [CLAUDE.md](./CLAUDE.md) - Guía para desarrolladores

### Enlaces Útiles
- [Render Docs](https://render.com/docs) - Documentación oficial
- [OpenRouter Docs](https://openrouter.ai/docs) - API de modelos
- [ZeroClaw Original](https://github.com/zeroclaw-labs/zeroclaw) - Repositorio oficial

### Comunidad
- ¿Encontraste un bug? [Abrir Issue](https://github.com/MasterXD123/zeroclaw_render-free/issues)
- ¿Tienes preguntas? [Discussions](https://github.com/MasterXD123/zeroclaw_render-free/discussions)

---

## Changelog

### v1.0.0 (2024)
- Initial release for Render deployment
- Qdrant removed, SQLite as default
- Skills adapted for Render (no docker exec)
- Dockerfile.render created
- Complete deployment guide added

---

## Licencia

MIT o Apache 2.0 (mismo que upstream ZeroClaw)

---

## Notas

⚠️ **Advertencia**: El tier gratuito de Render no garantiza persistencia de datos. Cada redeploy borrará la memoria y configuraciones. Para persistencia, considera actualizar a un plan de pago o usar una base de datos externa.

⚠️ **Seguridad**: Nunca expongas tu API key en código público. Las variables de entorno en Render son seguras, pero no hagas commit de keys en tu código.

---

*Hecho con ❤️ para la comunidad de ZeroClaw*