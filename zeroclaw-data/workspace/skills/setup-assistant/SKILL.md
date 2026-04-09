# Setup Assistant

## Descripción

Ayuda a conectar APIs y servicios externos como GitHub, Notion, Telegram, etc. Solo se activa cuando el usuario quiere configurar específicamente estos servicios.

## Activación

Se activa SOLO cuando el usuario menciona:
- "configurar telegram", "conectar github", "conectar notion"
- "necesito api key", "me falta el token"
- "como conecto" + nombre de servicio específico

**NO se activa con:** "que falta", "que necesito", "status", "help"

## Comportamiento

### Para Telegram:
1. Pedir el bot token
2. Verificar que sea válido
3. Guardar en config
4. Reiniciar si es necesario

### Para GitHub:
1. Pedir el token
2. Verificar permisos
3. Guardar en config

### Para Notion:
1. Explicar cómo crear integración
2. Pedir el Internal Integration Secret
3. Verificar conexión

## Servicios Soportados

| Servicio | Keyword para activar |
|----------|---------------------|
| Telegram | "configurar telegram", "conectar telegram" |
| GitHub | "configurar github", "conectar github" |
| Notion | "configurar notion", "conectar notion" |
| Qdrant | "configurar qdrant", "conectar qdrant" |

## NO hace

- No responde a "que falta", "status", "onboarding"
- No da resumen general del sistema
- No activa features automáticamente

## Keywords que NO lo activan

- "hola", "help", "que puedes hacer"
- "features", "status", "estado"
- "setup", "inicio" (usa onboarding para eso)
