# TOOLKIT — SuperAgent Tools Reference

> **ROL:** Documentar todas las tools disponibles para SuperAgent, su propósito, parámetros y cómo usarlas.

---

## OVERVIEW

SuperAgent tiene acceso a dos categorías de tools:

1. **Tools críticas** — Sin estas, SuperAgent no puede funcionar
2. **Tools extendidas** — Mejoran las capacidades pero no son blockers

---

## TOOLS CRÍTICAS

### task_tracker

**Estado:** 🔴 FALTA — debe implementarse en `src/tools/task_tracker.rs`

**Propósito:** Tracking de tareas del plan (crear, actualizar, listar, consultar)

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "action": {
      "type": "string",
      "enum": ["create", "update", "list", "get", "delete"]
    },
    "task_id": { "type": "string" },
    "description": { "type": "string" },
    "status": {
      "type": "string",
      "enum": ["pending", "in_progress", "completed", "failed", "skipped"]
    },
    "dependencies": {
      "type": "array",
      "items": { "type": "string" }
    },
    "result": { "type": "string" },
    "duration_ms": { "type": "integer" }
  },
  "required": ["action"]
}
```

**Acciones:**

| Action | Descripción |
|--------|-------------|
| `create` | Crear nueva tarea |
| `update` | Actualizar estado/resultado |
| `list` | Listar tareas (filter por status/plan) |
| `get` | Obtener detalle de una tarea |
| `delete` | Eliminar tarea |

**Implementación en Rust:**

```rust
// src/tools/task_tracker.rs
#[derive(Clone)]
pub struct TaskTrackerTool {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub plan_id: String,
    pub description: String,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
    pub result: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}
```

**Uso en skill:**
```bash
# Crear tarea
task_tracker(action="create", task_id="plan-123/1.1", description="Verificar CI", dependencies=[])

# Actualizar estado
task_tracker(action="update", task_id="plan-123/1.1", status="completed", result="CI OK")

# Listar tareas de un plan
task_tracker(action="list", plan_id="plan-123")
```

---

### calculator

**Estado:** 🔴 FALTA — debe implementarse en `src/tools/calculator.rs`

**Propósito:** Cálculos matemáticos precisos (básicos y científicos)

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "expression": {
      "type": "string",
      "description": "Math expression: 2+2, sqrt(16), sin(pi/2), log(100, 10), 2^8"
    },
    "precision": {
      "type": "integer",
      "description": "Decimal places (default: 10)",
      "default": 10
    }
  },
  "required": ["expression"]
}
```

**Operaciones soportadas:**

| Categoría | Operadores/Funciones |
|-----------|---------------------|
| Básicos | `+`, `-`, `*`, `/`, `^`, `%` |
| Comparación | `<`, `>`, `<=`, `>=`, `==`, `!=` |
| Funciones | `sqrt`, `abs`, `floor`, `ceil`, `round` |
| Trigonométricas | `sin`, `cos`, `tan`, `asin`, `acos`, `atan` |
| Logarítmicas | `log`, `ln`, `exp` |
| Constantes | `pi`, `e` |
| Conversiones | `deg2rad`, `rad2deg` |

**Implementación sugerida:**

```rust
// src/tools/calculator.rs
// Dependencia sugerida: meval crate

use meval::Expr;

impl CalculatorTool {
    pub fn new() -> Self { Self }

    pub fn evaluate(&self, expression: &str, precision: usize) -> Result<String, CalcError> {
        let expr: Expr = expression.parse()
            .map_err(|e| CalcError::Parse(e.to_string()))?;

        let result = expr.eval()
            .map_err(|e| CalcError::Eval(e.to_string()))?;

        Ok(format!("{:.prec$}", result, prec = precision))
    }
}
```

**Uso en skill:**
```bash
# Cálculo básico
calculator(expression="2 + 2")  # → "4"

# Científica
calculator(expression="sqrt(16) * sin(pi/2)")  # → "4"

# Precisión personalizada
calculator(expression="10 / 3", precision=4)  # → "3.3333"
```

---

## TOOLS EXTENDIDAS

### code_runner

**Estado:** 🟡 FALTA — debe implementarse en `src/tools/code_runner.rs`

**Propósito:** Ejecutar código Python/JavaScript en sandbox

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "language": {
      "type": "string",
      "enum": ["python", "javascript"]
    },
    "code": { "type": "string" },
    "timeout_ms": {
      "type": "integer",
      "description": "Max execution time (default: 30000)",
      "default": 30000
    }
  },
  "required": ["language", "code"]
}
```

**Implementación sugerida:**

```rust
// src/tools/code_runner.rs
// Python: usar rlua o pyo3 para sandboxed execution
// JS: usar deno_core o quickjs para sandboxed execution

// IMPORTANTE: El sandbox es crítico por seguridad
// - Sin acceso a filesystem fuera del workspace
// - Sin acceso a network
// - Sin acceso a system commands
```

**Uso en skill:**
```bash
code_runner(language="python", code="print(sum(range(1, 101)))")
# → "5050"

code_runner(language="javascript", code="Array.from({length: 5}, (_, i) => i * 2)")
# → "[0, 2, 4, 6, 8]"
```

---

### decision

**Estado:** 🟡 FALTA — debe implementarse en `src/tools/decision.rs`

**Propósito:** Evaluar condiciones y hacer branching lógico

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "condition": { "type": "string" },
    "context": {
      "type": "object",
      "description": "Variables disponibles para la evaluación"
    }
  },
  "required": ["condition"]
}
```

**Uso en skill:**
```bash
# Evaluar condición
decision(
  condition="disk_usage > 90 AND cpu_usage > 80",
  context={"disk_usage": 95, "cpu_usage": 85}
)
# → {"result": true, "action": "escalate"}

# Routing
decision(
  condition="user_tier == 'premium'",
  context={"user_tier": "premium"}
)
# → {"result": true, "route": "fast_path"}
```

---

### validator

**Estado:** 🟡 FALTA — debe implementarse en `src/tools/validator.rs`

**Propósito:** Validar output contra schema esperado

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "data": { "type": "any" },
    "schema": { "type": "object" }
  },
  "required": ["data", "schema"]
}
```

**Uso en skill:**
```bash
validator(
  data={"name": "test", "value": 42},
  schema={
    "type": "object",
    "properties": {
      "name": {"type": "string"},
      "value": {"type": "number", "minimum": 0}
    },
    "required": ["name", "value"]
  }
)
# → {"valid": true}
```

---

### time_tracker

**Estado:** 🟡 FALTA — debe implementarse en `src/tools/time_tracker.rs`

**Propósito:** Medir duración de tareas y operaciones

**Schema:**
```json
{
  "type": "object",
  "properties": {
    "action": {
      "type": "string",
      "enum": ["start", "stop", "elapsed"]
    },
    "timer_id": { "type": "string" },
    "task_id": { "type": "string" }
  },
  "required": ["action"]
}
```

**Uso en skill:**
```bash
time_tracker(action="start", timer_id="t1", task_id="plan-123/1.1")
# → {"timer_id": "t1", "started": "2026-04-07T10:00:00Z"}

# ... ejecutar tarea ...

time_tracker(action="stop", timer_id="t1")
# → {"timer_id": "t1", "duration_ms": 45230, "task_id": "plan-123/1.1"}
```

---

## TOOLS EXISTENTES DE ZEROCLAW

ZeroClaw ya tiene estas tools que SuperAgent puede usar directamente:

### Shell & Files

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `shell` | Ejecutar comandos del sistema |
| ✅ | `file_read` | Leer archivos |
| ✅ | `file_write` | Escribir archivos |
| ✅ | `file_edit` | Editar partes de archivos |
| ✅ | `glob_search` | Buscar archivos por patrón |

### Git & Code

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `git_operations` | Git add/commit/push/pull/etc |
| ✅ | `github` | Issues, PRs, repos |

### Web & Network

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `http_request` | Hacer requests HTTP |
| ✅ | `web_search` | Búsquedas web |
| ✅ | `web_fetch` | Obtener contenido de URLs |

### Browser

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `browser` | Automatización de navegador |
| ✅ | `browser_open` | Abrir URLs |
| ✅ | `screenshot` | Capturar pantalla |

### Memory

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `memory_store` | Guardar en memoria |
| ✅ | `memory_recall` | Recuperar de memoria |
| ✅ | `memory_forget` | Borrar de memoria |

### Scheduling

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `cron_add` | Crear cron job |
| ✅ | `cron_list` | Listar cron jobs |
| ✅ | `cron_run` | Ejecutar cron job |
| ✅ | `schedule` | Programar tareas |

### Integrations

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `google_workspace` | Gmail, Drive, Calendar |
| ✅ | `pushover` | Notificaciones |
| ✅ | `composio` | Integraciones |

### Hardware

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `hardware_board_info` | Info de boards |
| ✅ | `hardware_memory_read` | Leer memoria de hardware |

### Utilities

| Tool | Name | Uso |
|------|------|-----|
| ✅ | `pdf_read` | Leer PDFs |
| ✅ | `image_info` | Info de imágenes |
| ✅ | `delegate` | Delegar a otro agente |
| ✅ | `schema` | Schema de tools |

---

## TOOLKIT STATUS

### Implementación requerida:

| Tool | Status | Archivo | Prioridad | Dependencias |
|------|--------|---------|-----------|--------------|
| `task_tracker` | 🔴 FALTA | `src/tools/task_tracker.rs` | CRÍTICA | ninguna |
| `calculator` | 🔴 FALTA | `src/tools/calculator.rs` | CRÍTICA | `meval` crate |
| `code_runner` | 🟡 FALTA | `src/tools/code_runner.rs` | ALTA | `rlua` o `wasmer` |
| `decision` | 🟡 FALTA | `src/tools/decision.rs` | ALTA | ninguna |
| `validator` | 🟡 FALTA | `src/tools/validator.rs` | MEDIA | `jsonschema` crate |
| `time_tracker` | 🟡 FALTA | `src/tools/time_tracker.rs` | MEDIA | `chrono` crate |

### Registration en ZeroClaw:

```rust
// En src/tools/mod.rs, agregar:
pub mod task_tracker;
pub mod calculator;
pub mod code_runner;
pub mod decision;
pub mod validator;
pub mod time_tracker;

// En all_tools_with_runtime(), agregar:
Arc::new(TaskTrackerTool::new()),
Arc::new(CalculatorTool::new()),
Arc::new(CodeRunnerTool::new()),
Arc::new(DecisionTool::new()),
Arc::new(ValidatorTool::new()),
Arc::new(TimeTrackerTool::new()),
```

---

## ROADMAP DE IMPLEMENTACIÓN

### Fase 1: Críticas (Semana 1)
1. `calculator.rs` — Math básico y científico
2. `task_tracker.rs` — CRUD de tareas

### Fase 2: Extendidas (Semana 2)
3. `decision.rs` — Branching logic
4. `time_tracker.rs` — Medición de duración

### Fase 3: Sandbox (Semana 3)
5. `code_runner.rs` — Python/JS sandboxed

### Fase 4: Validation (Semana 4)
6. `validator.rs` — JSON schema validation

---

## REFERENCIAS

- Trait de Tool: `src/tools/traits.rs`
- Ejemplo de tool existente: `src/tools/shell.rs`, `src/tools/cron_add.rs`
- Registry de tools: `src/tools/mod.rs`

---

*TOOLKIT v1.0 — SuperAgent Tools Reference*
