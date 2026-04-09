# Plan: Arreglar 18 Bugs de Lógica de Negocio

## Resumen Ejecutivo

**Estado actual:** 18 tests fallando en `cargo test --lib`
**Objetivo:** Corregir los bugs y lograr 100% de tests pasando

---

## Inventario de Bugs

### 1. `channels::slack` (1 bug)

| Test | Descripción | Error |
|------|-------------|-------|
| `sanitize_attachment_filename_strips_path_traversal` | Sanitización de path traversal en archivos | Assertion falló: expected `..__..__secret.txt` but got `secret.txt` |

**Archivo:** `src/channels/slack.rs:2544`
**Causa probable:** El sanitizador está eliminando correctamente `../` pero el test espera que se transforme a `..__`

---

### 2. `channels::tests` (1 bug)

| Test | Descripción | Error |
|------|-------------|-------|
| `prompt_skills_compact_mode_omits_instructions_and_tools` | Skills en modo compact | Assertion: prompt no contiene `<location>skills/code-review/SKILL.md</location>` |

**Archivo:** `src/channels/mod.rs:5702`
**Causa probable:** En modo compact, las skills no se incluyen en el prompt

---

### 3. `config::schema` (1 bug)

| Test | Descripción | Error |
|------|-------------|-------|
| `persist_active_workspace_marker_is_cleared_for_default_config_dir` | Marker de workspace | Assertion: marker_path.exists() es false |

**Archivo:** `src/config/schema.rs:7578`
**Causa probable:** El path del marker no se crea correctamente para el config dir por defecto

---

### 4. `cron::scheduler` (2 bugs)

| Test | Descripción | Error |
|------|-------------|-------|
| `run_job_command_success` | Job command exitoso | Output no contiene `status=exit status: 0` |
| `run_job_command_failure` | Job command que falla | Mismo tipo de error |

**Archivo:** `src/cron/scheduler.rs:528`
**Causa probable:** El formato del output del comando cambió o no incluye el status

---

### 5. `providers::openai_codex` (1 bug)

| Test | Descripción | Error |
|------|-------------|-------|
| `resolve_responses_url_prefers_explicit_endpoint_env` | URL del endpoint | **Este test PASÓ** cuando se ejecutó solo |

**Nota:** Este bug puede ser intermitente o depender del orden de ejecución de tests

---

### 6. `security::policy` (2 bugs)

| Test | Descripción | Error |
|------|-------------|-------|
| `checklist_root_path_blocked` | Bloqueo de path root | Path root debería estar bloqueado |
| `checklist_workspace_only_blocks_all_absolute` | Solo workspace bloquea absolute | Absolute paths deberían bloquearse |

**Archivo:** `src/security/policy.rs`
**Causa probable:** La lógica de checklist no está bloqueando correctamente paths root/absolute

---

### 7. `tools::content_search` (7 bugs)

| Test | Descripción | Error |
|------|-------------|-------|
| `content_search_rejects_absolute_path` | Rechazo de path absoluto | Error no contiene "Absolute paths" |
| `content_search_count_mode` | Modo count | Falla |
| `content_search_context_lines` | Líneas de contexto | Falla |
| `content_search_case_insensitive` | Case insensitive | Falla |
| `content_search_basic_match` | Match básico | Falla |
| `content_search_include_filter` | Filtro include | Falla |
| `content_search_files_with_matches_mode` | Modo files with matches | Falla |
| `content_search_no_matches` | Sin matches | Falla |
| `content_search_subdirectory` | Subdirectorio | Falla |

**Archivo:** `src/tools/content_search.rs:891`
**Causa probable:** Error en la validación de path absoluto o en el output del comando grep

---

### 8. `tools::screenshot` (1 bug)

| Test | Descripción | Error |
|------|-------------|-------|
| `screenshot_command_contains_output_path` | Path de output en screenshot | unwrap() on None |

**Archivo:** `src/tools/screenshot.rs:320`
**Causa probable:** El path del screenshot no se está generando correctamente

---

## Plan de Arreglo por Categoría

### Fase 1: Bugs Críticos (Alto Impacto)

#### Bug 1.1: `content_search` - 7 tests fallando
**Prioridad:** 🔴 CRÍTICA
**Esfuerzo:** Alto

1. Investigar `src/tools/content_search.rs:891`
2. Verificar que `validate_path` rechace paths absolutos correctamente
3. Asegurar que el mensaje de error contenga "Absolute paths"
4. Verificar que todos los modos (count, context, etc.) funcionen

**Commands de debug:**
```bash
cargo test --lib tools::content_search::tests -- --nocapture
RUST_BACKTRACE=1 cargo test --lib content_search_rejects_absolute_path
```

#### Bug 1.2: `screenshot` - 1 test fallando
**Prioridad:** 🔴 CRÍTICA
**Esfuerzo:** Bajo

1. Investigar `src/tools/screenshot.rs:320`
2. El unwrap() está fallando - el path es None
3. Verificar que `output_path` se genere antes del assert

---

### Fase 2: Bugs de Security (Alta Prioridad)

#### Bug 2.1: `security::policy` - 2 tests fallando
**Prioridad:** 🟠 ALTA
**Esfuerzo:** Medio

1. Investigar `src/security/policy.rs`
2. `checklist_root_path_blocked` - root `/` no está bloqueado
3. `checklist_workspace_only_blocks_all_absolute` - absolute paths no se bloquean

**Commands de debug:**
```bash
cargo test --lib security::policy::tests -- --nocapture
```

---

### Fase 3: Bugs de Channels

#### Bug 3.1: `slack` - 1 test fallando
**Prioridad:** 🟡 MEDIA
**Esfuerzo:** Bajo

1. Investigar `src/channels/slack.rs:2544`
2. Test espera `..__..__secret.txt` pero obtiene `secret.txt`
3. La sanitización está funcionando, el test tiene expectativa incorrecta O viceversa

#### Bug 3.2: `channels::tests` - 1 test fallando
**Prioridad:** 🟡 MEDIA
**Esfuerzo:** Medio

1. Investigar `src/channels/mod.rs:5702`
2. En compact mode, skills no se incluyen
3. Necesitan agregarse los `<location>` tags para skills

---

### Fase 4: Bugs de Config/Cron

#### Bug 4.1: `config::schema` - 1 test fallando
**Prioridad:** 🟡 MEDIA
**Esfuerzo:** Bajo

1. Investigar `src/config/schema.rs:7578`
2. El marker path no existe cuando debería

#### Bug 4.2: `cron::scheduler` - 2 tests fallando
**Prioridad:** 🟡 MEDIA
**Esfuerzo:** Medio

1. Investigar `src/cron/scheduler.rs:528`
2. El output del comando no contiene `status=exit status: 0`
3. Posible cambio en formato de output de comandos

---

## Orden de Implementación Sugerida

| Orden | Categoría | Tests | Prioridad |
|-------|-----------|-------|-----------|
| 1 | content_search | 7 | CRÍTICA |
| 2 | screenshot | 1 | CRÍTICA |
| 3 | security::policy | 2 | ALTA |
| 4 | slack | 1 | MEDIA |
| 5 | channels::tests | 1 | MEDIA |
| 6 | config::schema | 1 | MEDIA |
| 7 | cron::scheduler | 2 | MEDIA |
| 8 | openai_codex | 1 | BAJA (intermitente) |

---

## Comandos de Verificación

```bash
# Ejecutar todos los tests fallando
cargo test --lib 2>&1 | grep FAILED

# Ejecutar un módulo específico
cargo test --lib tools::content_search
cargo test --lib security::policy
cargo test --lib channels::slack
cargo test --lib channels::tests
cargo test --lib config::schema
cargo test --lib cron::scheduler
cargo test --lib tools::screenshot

# Ejecutar con debug
cargo test --lib tools::content_search::tests::content_search_rejects_absolute_path -- --nocapture
RUST_BACKTRACE=1 cargo test --lib tools::content_search::tests::content_search_rejects_absolute_path

# Suite completa
cargo test --lib
```

---

## Definición de Done

- [ ] `cargo test --lib` pasa 100% (0 failed)
- [ ] Los 18 bugs identificados están corregidos
- [ ] No se introdujeron nuevos bugs
- [ ] Validación: `cargo test --lib` muestra `test result: ok`

---

## Timeline Estimado

| Fase | Descripción | Tests | Tiempo |
|------|-------------|-------|--------|
| Fase 1 | content_search + screenshot | 8 | 2-3 horas |
| Fase 2 | security::policy | 2 | 1-2 horas |
| Fase 3 | channels (slack + tests) | 2 | 1-2 horas |
| Fase 4 | config + cron | 3 | 1-2 horas |
| Fase 5 | Verificación final | todos | 30 min |

**Total estimado: 6-10 horas**
