# Auto-Connect

## Descripción
Skill para conectar servicios de forma automática. Cuando el usuario dice "conecta [servicio]", este skill maneja el flujo completo: validar token, actualizar config, verificar conexión, y reportar resultado.

## Activación
Se activa cuando el usuario dice:
- "conecta Notion"
- "conecta GitHub"
- "conecta Telegram"
- "configura OpenRouter"
- "verifica conexión"
- "prueba [servicio]"

## Servicios y Flujos

### Notion
```
Input: "conecta Notion"
Flow:
  1. Verificar si NOTION_KEY existe en config
  2. Si no existe → pedir token
  3. Si existe → verificar con API call
  4. Si OK → reportar éxito
  5. Si fail → mostrar error específico
```

### GitHub
```
Input: "conecta GitHub"
Flow:
  1. Verificar si GITHUB_TOKEN existe
  2. Si no existe → pedir token con link a generate
  3. Probar API: GET /user
  4. Si OK → reportar usuario y permisos
  5. Si fail → indicar problema
```

### Telegram
```
Input: "conecta Telegram"
Flow:
  1. Verificar bot_token en channels_config.telegram
  2. Si no existe → pedir token de @BotFather
  3. bind-telegram con user ID si no está bindeado
  4. Reiniciar daemon si fue necesario
  5. Enviar mensaje de prueba al bot
```

### OpenRouter
```
Input: "conecta OpenRouter"
Flow:
  1. Verificar API key
  2. Si no existe → link a openrouter.ai/keys
  3. Test: hacer request de modelo
  4. Mostrar modelos disponibles
```

## Herramientas disponibles

### Shell commands
```bash
# Ver estado de servicios Docker
docker ps

# Ver logs de zeroclaw
docker compose logs zeroclaw

# Reiniciar zeroclaw
docker compose restart zeroclaw

# Ver config actual
docker exec zeroclaw-enterprise zeroclaw config show

# Test API keys
curl -H "Authorization: Bearer $TOKEN" $API_URL
```

### Memory
```
# Guardar estado de conexiones
memory_store: key="connections/status", category="config"

# Recuperar estado
memory_recall: key="connections/status"
```

## Formato de Respuesta

### Éxito
```
## ✅ [Servicio] Conectado

- **Estado:** Operativo
- **Usuario:** [user si aplica]
- **Token:** ✅ verificado
- **Acción tomada:** Ninguna / Actualizado config / Reiniciado daemon

ℹ️ Ya puedes usar [servicio] desde ZeroClaw.
```

### Error
```
## ⚠️ Error conectando [Servicio]

- **Problema:** [descripción específica]
- **Token:** ✓ presente / ❌ ausente / ⚠️ inválido
- **Sugerencia:** [paso a seguir]

**Para obtener un token:**
1. Ve a [URL]
2. [Pasos específicos]
3. Copia el token y pégalo aquí
```

## Validación de Tokens

### Notion
```python
# Endpoint de prueba
GET https://api.notion.com/v1/users
Headers: Authorization: Bearer <NOTION_KEY>
```

### GitHub
```bash
# Endpoint de prueba
curl -H "Authorization: Bearer <GITHUB_TOKEN>" \
     https://api.github.com/user
```

### Telegram
```bash
# Get updates (prueba de que el bot responde)
curl "https://api.telegram.org/bot<TOKEN>/getUpdates"
```

### OpenRouter
```bash
# Test simple
curl -X POST https://openrouter.ai/api/v1/chat/completions \
  -H "Authorization: Bearer <API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{"model": "openrouter/free", "messages": [{"role": "user", "content": "test"}]}'
```

## Casos Especiales

### OAuth Requiere Login Interactivo
Si un servicio requiere OAuth flow interactivo (como gws auth login):
```
## Google Workspace requiere autenticación manual

1. Ejecuta en tu terminal local:
   gws auth login

2. Sigue las instrucciones del navegador

3. Una vez autenticado, dime "gws listo"
   para verificar la conexión.

⚠️ No puedo hacer esto automáticamente.
```

### Docker Restart Necesario
Cuando se cambia la config de un canal (Telegram, Discord):
```
## 🔄 Cambio requiere reinicio

He actualizado la configuración de Telegram.
Ahora necesito reiniciar el daemon para aplicar los cambios.

¿Reinicio ahora? (s/n)
```

## Detección de Cambios en Config

Para detectar cambios y reiniciar automáticamente:
```bash
# Watchdog: verificar cada 30 segundos
# Si config.toml cambia → restart daemon
```

## Notas
- Los tokens deben guardarse encryptados
- Usar `secrets.encrypt = true` en config
- En Docker, el config directory está mounted localmente

---

*Auto-Connect v1.0 — Service Connection Skill*
