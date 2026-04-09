# Watchdog

## Descripción
Skill de vigilancia que monitorea cambios en la configuración y reinicia el daemon automáticamente cuando es necesario. También detecta caídas de servicios y alerta al usuario.

## Activación
Se activa cuando:
- El usuario modifica archivos de configuración
- Un servicio (Qdrant, ZeroClaw) deja de responder
- Hay cambios en el entorno Docker
- Se detectan errores de configuración que requieren restart

## Monitoreo Continuo

El watchdog verifica cada **30 segundos**:

### Health Checks
```bash
# ZeroClaw Gateway
curl http://localhost:42617/health

# Qdrant
curl http://localhost:6333/collections

# Docker containers
docker ps --format "{{.Names}}: {{.Status}}"
```

### Cambio de Config Detectado
```
SI: config.toml fue modificado (comparar timestamp)
ENTONCES:
  1. Verificar nueva config es válida
  2. Si válida → restart daemon
  3. Si inválida → rollback + alertar usuario
  4. Verificar que todo funciona post-restart
```

## Comandos de Control

| Comando | Descripción |
|---------|-------------|
| `watchdog status` | Ver estado actual de salud |
| `watchdog start` | Iniciar watchdog manual |
| `watchdog stop` | Detener watchdog |
| `watchdog restart` | Reiniciar servicios |
| `watchdog history` | Ver historial de eventos |

## Eventos y Respuestas

### Evento: Cambio en Config
```
## 🔄 Cambio detectado en config

**Archivo:** /zeroclaw-data/.zeroclaw/config.toml
**Modificado:** hace 5 segundos
**Hash anterior:** abc123
**Hash nuevo:** def456

**Validando nueva config...**
✅ Sintaxis válida
✅ Puertos OK
✅ API keys presentes

**Reiniciando daemon...**
✅ Daemon reiniciado
✅ Health check OK

ℹ️ Los cambios fueron aplicados correctamente.
```

### Evento: Servicio Caído (ZeroClaw)
```
## 🚨 Alerta: ZeroClaw no responde

**Estado:** El gateway no responde en puerto 42617
**Último check:** hace 30 segundos
**Uptime anterior:** 2 horas

**Intentando recover:**
1. Verificar Docker container status
2. Si stopped → iniciar container
3. Si error → revisar logs
4. Si persiste → alertar usuario

**Recovery en progreso...**
```

### Evento: Servicio Caído (Qdrant)
```
## 🚨 Alerta: Qdrant no responde

**Estado:** Puerto 6333 no responde
**Último check:** hace 30 segundos

**Intentando recover:**
1. Verificar container
2. Si stopped → iniciar
3. Si error → rebuild si es necesario
4. Verificar colección existe

**Recovery en progreso...**
```

### Evento: Config Inválida
```
## ⚠️ Cambio de Config Rechazado

**Archivo:** config.toml
**Error:** Línea 23 - 'port' debe ser integer, encontró "invalid"

**El cambio NO fue aplicado.**

**Opciones:**
1. Corregir manualmente y guardar
2. Revertir al archivo anterior
3. Ignorar por ahora

¿Quieres que revierta al archivo anterior? (s/n)
```

## Recovery Automático

### Nivel 1: Auto-restart
Para cambios de config válidos que requieren restart:
- Reiniciar daemon automáticamente
- Verificar health post-restart
- Si falla → hacer rollback

### Nivel 2: Service Recovery
Para servicios caídos:
- Verificar estado de containers
- Iniciar containers detenidos
- Si container corrupto → rebuild

### Nivel 3: Full Recovery
Para errores críticos:
- Backup de estado actual
- Reconstruir desde imagen
- Restaurar config de backup
- Alertar usuario con opciones

## Historial de Eventos

```json
{
  "events": [
    {
      "timestamp": "2026-04-09T12:30:00Z",
      "type": "config_change",
      "file": "config.toml",
      "action": "restart",
      "status": "success"
    },
    {
      "timestamp": "2026-04-09T12:15:00Z",
      "type": "service_down",
      "service": "zeroclaw",
      "action": "restart",
      "status": "success"
    },
    {
      "timestamp": "2026-04-09T11:00:00Z",
      "type": "config_invalid",
      "file": "config.toml",
      "error": "Invalid port value",
      "action": "rollback",
      "status": "success"
    }
  ]
}
```

## Configuración

```toml
[watchdog]
enabled = true
interval_seconds = 30
auto_restart = true
auto_rollback = true
max_retries = 3
alert_on_failure = true

# Servicios a monitorear
monitored_services = ["zeroclaw", "qdrant"]

# Puertos a verificar
health_ports = [42617, 6333]
```

## Slack/Alert Integration

Opcionalmente notify a canales externos:
```toml
[watchdog.alert]
telegram = true  # Notificar a usuario vía Telegram
email = false
webhook = false
```

## Notas de Implementación

- Usa `inotifywait` o polling para detectar cambios de archivo
- Compara hashes MD5/SHA256 para saber si hubo cambios reales
- Mantiene backup del último archivo válido
- Logs en `/zeroclaw-data/logs/watchdog.log`

## Seguridad

- Solo monitorea archivos en directorios esperados
- No permite ejecución de código externo
- Requiere confirmación para rollbacks completos
- Audit log de todas las acciones

---

*Watchdog v1.0 — Auto-Recovery & Monitoring Skill*
