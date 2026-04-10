# Diferencias: ZeroClaw-render-free vs ZeroClaw-free (Original)

## Resumen
Este fork está específicamente optimizado para deploy en **Render free tier** sin dependencias externas adicionales.

---

## Cambios Realizados

### 1. Eliminación de Qdrant

| Aspecto | Original | Este Fork |
|---------|----------|-----------|
| Qdrant (vector DB) | ✅ Incluido como servicio Docker | ❌ Eliminado |
| Memoria | qdrant o sqlite | **sqlite** (por defecto) |
| Colección | configurable | no aplica |

**Archivos modificados:**
- `docker-compose.yml` - eliminado servicio qdrant
- `.env.example` - eliminadas variables QDRANT_*

---

### 2. Skills Adaptados para Render

| Skill | Original | Este Fork |
|-------|----------|-----------|
| Watchdog | `docker exec`, `docker ps`, `docker restart` | `curl`, `ps aux`, sin restart automático |
| Self-Healer | `docker exec`, `docker restart` | `curl`, diagnóstico, Render maneja restart |
| Code-Guardian | `docker exec`, check_qdrant | Sin docker, sin qdrant |
| Auto-Connect | `docker restart`, test_qdrant | Sin docker, test de gateway y API |

**Archivos modificados:**
- `zeroclaw-data/workspace/skills/watchdog/SKILL.toml`
- `zeroclaw-data/workspace/skills/self-healer/SKILL.toml`
- `zeroclaw-data/workspace/skills/code-guardian/SKILL.toml`
- `zeroclaw-data/workspace/skills/auto-connect/SKILL.toml`

---

### 3. Docker para Render

| Aspecto | Original | Este Fork |
|---------|----------|-----------|
| Dockerfile | Dockerfile (local) | `Dockerfile.render` (optimizado) |
| Puerto | 42617 (hardcoded) | `$PORT` (dinámico) |
| Config | qdrant por defecto | **sqlite** por defecto |
| Puerto host | 0.0.0.0:42617 | $PORT desde Render |

**Archivos nuevos:**
- `Dockerfile.render` - usa variable $PORT de Render
- `render.yaml` - blueprint de Render (opcional)
- `RENDER_DEPLOY_GUIDE.md` - guía completa de deployment

---

### 4. Documentación

| Aspecto | Original | Este Fork |
|---------|----------|-----------|
| README | General, para Docker local | Actualizado con sección Render |
| Guía de Deploy | DOCKER-BUILD.md | `RENDER_DEPLOY_GUIDE.md` (detallada) |
| Estado de features | Qdrant como opción | SQLite como default, Qdrant eliminado |

---

## Features Soportados en Render

| Feature | Estado | Notas |
|---------|--------|-------|
| Gateway HTTP | ✅ | Puerto dinámico $PORT |
| SQLite Memory | ✅ | Por defecto, sin config adicional |
| Skills System | ✅ | Adaptados, sin docker exec |
| Telegram Bot | ✅ | Si se configura token |
| Cron Jobs | ✅ | Funciona igual |
| Agent Loop | ✅ | Igual que original |
| Agentic-Bridge | ✅ | Igual que original |

---

## Features NO Soportados (no disponibles en Render free tier)

| Feature | Razón |
|---------|-------|
| Qdrant (vector DB) | Requiere servicio externo o Docker daemon |
| Auto-restart de containers | Render maneja esto automáticamente |
| docker exec para monitoreo | Sin acceso a Docker daemon |
| Volúmenes persistentes | Requiere plan de pago |

---

## Variables de Entorno

### Original (Docker Compose)
```env
OPENAI_API_KEY=...
PROVIDER=openrouter
ZEROCLAW_MODEL=...
QDRANT_HOST=qdrant
QDRANT_PORT=6333
```

### Este Fork (Render)
```env
OPENAI_API_KEY=...        # Obligatorio
PROVIDER=openrouter
ZEROCLAW_MODEL=openrouter/google/gemma-4-27b-it
PORT=10000               # Render asigna automáticamente
```

---

## Próximos Pasos Sugeridos

1. **Migrar a este fork si:**
   - Quieres deploy en Render free tier
   - No necesitas búsqueda semántica avanzada
   - Prefieres simplicidad (SQLite)

2. **Permanecer con el original si:**
   - Necesitas Qdrant para memoria vectorial
   - Tienes tu propio servidor/Docker
   - Necesitas persistencia de datos

---

## Changelog de Adaptación

- `docker-compose.yml` - Eliminado Qdrant
- `.env.example` - Eliminadas variables QDRANT_*
- `config.toml.example` - SQLite ya es default (sin cambios)
- Skills - Adaptados para funcionar sin Docker daemon
- `Dockerfile.render` - Nuevo, optimizado para Render
- `README.md` - Actualizado con sección de Render
- `RENDER_DEPLOY_GUIDE.md` - Nueva guía completa

---

*Para detalles de deployment, ver `RENDER_DEPLOY_GUIDE.md`*