# Code Guardian

## Descripción
Skill de protección que evita que el sistema se rompa por código defectuoso, configuraciones inválidas, o cambios problematicos. Antes de ejecutar cualquier acción que modifique archivos críticos, validará y advertirá al usuario.

## Activación
Se activa cuando:
- El usuario quiere modificar archivos de configuración (config.toml, docker-compose.yml, Dockerfile)
- Se van a ejecutar comandos de shell destructivos
- Se detectan cambios en dependencias (Cargo.toml, package.json)
- Se van a instalar paquetes o modificar el sistema
- Cualquier acción marcada como "high risk" o "medium risk" en el security policy

## Niveles de Protección

### 🔒 Nivel 1: Validación Básica
Se ejecuta siempre. Verifica:
- Sintaxis de archivos de config (TOML, YAML, JSON)
- Que los tokens/API keys no estén vacíos
- Que los servicios referenciados existan
- Que no haya errores de formato obvios

### 🔐 Nivel 2: Validación Estructural  
Para cambios en archivos críticos:
- `config.toml` - Verificar schema válido
- `docker-compose.yml` - Verificar servicios válidos
- `Cargo.toml` - Verificar dependencias y features
- `Dockerfile` - Verificar que FROM images existan

### 🚫 Nivel 3: Bloqueo
Para operaciones destructivas o de alto riesgo:
- Eliminar archivos del sistema
- Modificar dependencias críticas sin backup
- Comandos que requieran reinicio obligatorio
- Cambios en puertos reservados (42617, 6333, 6334)

## Validaciones Automáticas

### config.toml
```bash
# Verificar que sea TOML válido
# Verificar campos requeridos
# Verificar tipos de datos (puertos como integers, etc)
# Verificar que no haya duplicados
```

### docker-compose.yml
```bash
# Verificar YAML válido
# Verificar que servicios tengan image o build
# Verificar que puertos no estén en uso
# Verificar que volumes existan o se puedan crear
```

### Cargo.toml
```bash
# Verificar sintaxis TOML
# Verificar que cada dependencia tenga version
# Verificar features válidos
# No allow duplicates
```

## Flujo de Protección

```
INPUT: Usuario quiere hacer cambio [X]

1. IDENTIFICAR tipo de cambio
   - Si modifica config → aplicar validación de config
   - Si modifica código → aplicar validación de código
   - Si comando destructivo → aplicar validación de seguridad

2. EJECUTAR validaciones
   - ¿Sintaxis válida?
   - ¿Campos requeridos presentes?
   - ¿Tipos correctos?
   - ¿Conflictos con existente?

3. SI PASA TODO → PERMITIR cambio
   - Mostrar qué se hará
   - Executar
   - Verificar resultado

4. SI FALLA → BLOQUEAR y EXPLICAR
   - Mostrar error específico
   - Sugerir corrección
   - No ejecutar hasta que se arregle
```

## Comandos que Protege

### ⚠️ Precaución Extrema
| Comando | Razón |
|---------|-------|
| `rm -rf` | Puede borrar todo |
| `drop database` | Destruye datos |
| `git reset --hard` | Pierde cambios |
| `docker system prune` | Limpia todo |
| `cargo update --breaking` | Puede romper dependencias |

### ⚠️ Precaución Moderada
| Comando | Razón |
|---------|-------|
| Modificar `config.toml` | Puede afectar runtime |
| Modificar `docker-compose.yml` | Afecta servicios |
| Modificar `Cargo.toml` | Cambia dependencias |
| `docker compose down` | Detiene servicios |
| `kill` sin PID específico | Puede matar proceso errado |

### ✅ Generalmente Seguro
| Comando | Razón |
|---------|-------|
| `docker compose logs` | Solo lectura |
| `docker ps` | Solo lectura |
| `cat archivo` | Solo lectura |
| `curl` a APIs públicas | Solo lectura |

## Validación de Configuraciones

### config.toml - Checklist
```
[ ] api_key no está vacío (si se requiere)
[ ] default_provider es válido
[ ] default_model existe
[ ] gateway.port es integer entre 1024-65535
[ ] channels_config tiene estructura válida
[ ] memory.backend es válido (sqlite, postgres, memory)
[ ] No hay secciones duplicadas
```

### docker-compose.yml - Checklist
```
[ ] version válida
[ ] services es un object
[ ] Cada servicio tiene 'image' o 'build'
[ ] Puertos no duplicados
[ ] volumes referenciados existen
[ ] depends_on referencias válidas
[ ] environment variables tienen valores (o placeholder)
```

### Cargo.toml - Checklist
```
[ ] [package] tiene name y version
[ ] edition es válida (2018, 2021)
[ ] dependencies no tiene duplicados
[ ] Cada dependency tiene version constraint
[ ] features referenciadas existen
[ ] dev-dependencies separados de dependencies
```

## Respuestas de Error

### Config Inválida
```
## ⚠️ Configuración Inválida

**Archivo:** config.toml
**Error:** Línea 15 - 'port' debe ser integer, encontró "abc"

**Validación:**
- ❌ Puerto inválido
- ⚠️ api_key vacío
- ✅ provider válido

**Sugerencia:** Cambiar `port = "abc"` a `port = 42617`

¿Arreglar automáticamente? (s/n)
```

### Dependencias Conflictivas
```
## ⚠️ Dependencias Conflictivas

**Archivo:** Cargo.toml
**Error:** 'tokio' version conflictiva
  - Requerido por: axum, tower
  - Versión 1.x incompatible con 0.5.x

**Solución:** Especificar version consistente:
```toml
tokio = { version = "1.50", features = ["full"] }
```

**Recomendación:** No hacer cargo update --breaking
```

### Servicio No Disponible
```
## ⚠️ Servicio No Disponible

**Error:** Puerto 42617 ya está en uso por otro proceso

**Procesos usando el puerto:**
- nginx (PID 1234)
- node (PID 5678)

**Opciones:**
1. Detener el proceso conflictivo: `kill 1234`
2. Cambiar puerto en config: `gateway.port = 42618`
3. Usar otro puerto en docker-compose

¿Qué prefieres?
```

## Auto-Recovery

### Si un cambio rompe el sistema:

1. **Detectar que algo está mal**
   - Health check falla
   - Logs muestran errores críticos
   - Puerto no responde

2. **Ofrecer recovery options**
   ```
   ## 🔧 Recovery Mode
   
   El sistema no está respondiendo correctamente.
   
   **Último cambio:** Modificación de config.toml (hace 5 min)
   
   **Opciones de Recovery:**
   
   1. Revertir config.toml a versión anterior
   2. Reiniciar daemon (si fue detenido)
   3. Rebuild de contenedor
   4. Limpiar y reiniciar desde cero
   
   ¿Cuál prefieres?
   ```

3. **Auto-revert si usuario aprueba**
   - Backup automático antes de cambios
   - Restore desde backup
   - Verify que funciona

## Config de Seguridad

```toml
[autonomy]
# Nivel de protección
protection_level = "strict"  # "permissive", "normal", "strict"

# Bloquear sin confirmar
confirm_destructive = true
confirm_config_changes = true
confirm_dependency_changes = true

# Auto-backup antes de cambios
auto_backup = true
backup_retention_days = 7
```

## Alertas

### 🚨 Alerta Crítica (Bloquea acción)
- Intento de modificar puerto de gateway
- Intento de eliminar volumen de datos
- Comando destructivo sin confirmación

### ⚠️ Alerta Alta (Confirma antes)
- Modificar config.toml
- docker compose down
- Cualquier cargo update

### 📝 Alerta Media (Informa nomas)
- Nuevo archivo en workspace
- Cambio en archivo no crítico
- Nuevo dependency agregado

---

*Code Guardian v1.0 — Protection & Validation Skill*
