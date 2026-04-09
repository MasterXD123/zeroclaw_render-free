# Self-Healer v2.1

## Descripción

Sistema de auto-curación autónomo para ZeroClaw. Un solo script maestro maneja salud, healing, auto-tuning y predicción de problemas.

## Arquitectura

```
CRON (cada 3 horas)
└─> health-master.sh (UN SOLO SCRIPT)
      ├─ Health check completo
      ├─ SI crítico (DOWN o >90%) → AUTO-HEAL inmediato
      ├─ SI warning (80-90%) → reporta sugerencia
      └─ Predictive → analiza tendencias
```

## Un Solo Script

`health-master.sh` hace todo:

| Frecuencia | Acción |
|------------|--------|
| Cada 60s | Health check completo |
| Cada 5min | Auto-tune (si memory > 80%) |
| Cada 10min | Predictive (analiza tendencias) |

## Tools (8 únicamente)

| Tool | Uso |
|------|-----|
| `check_health` | Health check completo en un comando |
| `docker_stats` | Ver uso de recursos |
| `docker_restart` | Reiniciar si algo falla |
| `docker_start` | Iniciar si está detenido |
| `read_state` | Ver estado actual |
| `clean_logs` | Limpiar logs problemáticos |
| `run_health_master` | Ejecutar script maestro |
| `get_logs_tail` | Ver últimos logs |

## Flujo de Healing

```
1. check_health → GATEWAY:DOWN QDRANT:UP MEM:45 DISK:32
2. SI GATEWAY DOWN → docker_restart (si pasaron >10 min desde último)
3. SI QDRANT DOWN → restart_qdrant (si pasaron >10 min)
4. SI memory > 90% → clean_logs (si pasaron >10 min)
5. SI disk > 90% → clean_logs (si pasaron >10 min)
6. SI memory 80-90% → solo reporta sugerencia
7. Log del healing
```

**Anti-loop**: Si algo se cae justo después de healear, espera 10 min antes de healear de nuevo.

## Auto-Tune

Cada 5 minutos (cada 5ta ejecución):
- Si memory > 80% → reporta sugerencia
- No cambia nada automáticamente (solo reporta)

## Predictive Monitor

Cada 10 minutos (cada 10ma ejecución):
- Compara memory actual con hace 10 min
- Si memory subió > 10% → "OOM预计在30分钟内"
- Ejecuta cleanup proactivo si detecta problema

## Estado

El estado se guarda en `state.json`:
```
gateway=UP
qdrant=UP
memory=45
disk=32
last_check=2026-04-09T18:00:00Z
exec_count=150
```

## Comandos

```bash
# Activar - cada 3 horas (10800000 ms)
zeroclaw cron add-every 10800000 "bash /zeroclaw-data/workspace/skills/self-healer/commands/health-master.sh"

# Ver estado
zeroclaw skill tool self-healer read_state

# Forzar health check ahora
zeroclaw skill tool self-healer run_health_master

# Ver logs
zeroclaw skill tool self-healer get_logs_tail
```

## Diferencia con v2.0

| v2.0 | v2.1 |
|------|------|
| 25 tools | 8 tools |
| 3 scripts separados | 1 script maestro |
| 3 cron jobs | 1 cron job |
| 10+ docker exec/ejecución | 1-3 docker exec/ejecución |
| ~2000 tokens/ejecución | ~500 tokens/ejecución |

---

*Self-Healer v2.1 — Optimizado para tokens*
