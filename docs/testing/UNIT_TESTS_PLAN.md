# Plan: 19 Unit Tests - Bugs de Lógica de Negocio

## Contexto

El archivo `src/agent/tests.rs` documenta 25 tests de integración del agente, pero el análisis de código revela **gaps en lógica de negocio** que requieren unit tests específicos.

**Estado actual:** 3158 tests pasando en `cargo test --lib`

---

## Tests Pendientes por Categoría

### 1. Credential Scrubbing (4 tests)

| # | Test | Bug/Edge Case | Prioridad |
|---|------|---------------|-----------|
| 1 | `scrub_short_api_key` | Keys < 8 chars no se redacted completamente | Alta |
| 2 | `scrub_nested_json_credentials` | Credentials en JSON anidado | Alta |
| 3 | `scrub_base64_lookalike_tokens` | Tokens base64 que parecen texto plano | Media |
| 4 | `scrub_unicode_credentials` | Credentials con unicode | Baja |

**Ubicación:** `src/agent/loop_.rs` - función `scrub_credentials()`

---

### 2. History Trimming (3 tests)

| # | Test | Bug/Edge Case | Prioridad |
|---|------|---------------|-----------|
| 5 | `trim_history_preserves_system_when_all_system` | Edge case: solo mensajes system | Media |
| 6 | `trim_history_multiple_system_messages` | Múltiples system prompts en historia | Media |
| 7 | `trim_compaction_summary_not_duplicated` | Summary de compaction no se duplica | Alta |

**Ubicación:** `src/agent/agent.rs` - función `trim_history()`

---

### 3. Tool Deduplication (2 tests)

| # | Test | Bug/Edge Case | Prioridad |
|---|------|---------------|-----------|
| 8 | `duplicate_tool_call_preserves_order` | Duplicados mantienen orden de resultados | Alta |
| 9 | `duplicate_tool_with_different_args_not_deduplicated` | Mismo tool, args diferentes = no dedup | Alta |

**Ubicación:** `src/agent/loop_.rs` - `seen_tool_signatures` logic

---

### 4. Approval Bypass (2 tests)

| # | Test | Bug/Edge Case | Prioridad |
|---|------|---------------|-----------|
| 10 | `approval_bypassed_for_non_cli_channels` | Canales no-CLI saltan approval | Alta |
| 11 | `approval_grants_before_hook_modification` | Hook puede modificar post-approval | Media |

**Ubicación:** `src/agent/loop_.rs` - `ApprovalManager` y hooks

---

### 5. Memory & Auto-save (3 tests)

| # | Test | Bug/Edge Case | Prioridad |
|---|------|---------------|-----------|
| 12 | `autosave_min_chars_constant_dead_code` | `AUTOSAVE_MIN_MESSAGE_CHARS` no se enforce | Alta |
| 13 | `memory_loader_filters_none_score_entries` | Entries con score: None pasan filtro | Media |
| 14 | `memory_context_empty_when_all_below_threshold` | Comportamiento cuando todo está bajo threshold | Media |

**Ubicación:** `src/agent/memory_loader.rs`, `src/agent/loop_.rs`

---

### 6. Cancellation & Timeouts (2 tests)

| # | Test | Bug/Edge Case | Prioridad |
|---|------|---------------|-----------|
| 15 | `cancellation_during_long_tool_execution` | Cancelar tool larga no causa panic | Alta |
| 16 | `provider_warmup_failure_silent` | Warmup failure no bloquea ejecución | Baja |

**Ubicación:** `src/agent/loop_.rs`, `src/providers/reliable.rs`

---

### 7. Compaction (2 tests)

| # | Test | Bug/Edge Case | Prioridad |
|---|------|---------------|-----------|
| 17 | `compaction_fallback_deterministic_truncation` | Fallback cuando summarizer falla | Alta |
| 18 | `compaction_multiple_times_compounds` | Compactaciones sucesivas no degradan | Media |

**Ubicación:** `src/agent/loop_.rs` - `auto_compact_history`

---

### 8. Parallel Execution (1 test)

| # | Test | Bug/Edge Case | Prioridad |
|---|------|---------------|-----------|
| 19 | `parallel_execution_blocked_by_approval_gate` | Approvals bloquean paralelo aunque otros no necesiten | Alta |

**Ubicación:** `src/agent/loop_.rs` - `should_execute_tools_in_parallel()`

---

## Implementación

### Estructura de Archivos

```
src/agent/
├── tests/
│   ├── mod.rs              # Exportar todos los tests
│   ├── credential_tests.rs # Tests 1-4
│   ├── history_tests.rs    # Tests 5-7
│   ├── deduplication_tests.rs # Tests 8-9
│   ├── approval_tests.rs   # Tests 10-11
│   ├── memory_tests.rs     # Tests 12-14
│   ├── cancellation_tests.rs # Tests 15-16
│   ├── compaction_tests.rs # Tests 17-18
│   └── parallel_tests.rs   # Test 19
```

### Herramientas de Test Reutilizables

El archivo `src/agent/tests.rs` ya tiene:
- `ScriptedProvider` - Mock provider con respuestas programables
- `EchoTool`, `FailingTool`, `CountingTool` - Mocks de tools
- `make_memory()`, `make_sqlite_memory()` - Helpers de memoria
- `build_agent_with_*` - Builders de agente

**Reusar estos helpers** para los nuevos tests.

---

## Orden de Implementación Sugerida

1. **Credential tests (1-4)** - Son independientes, buen punto de entrada
2. **History tests (5-7)** - Usan agentes simples
3. **Memory tests (12-14)** - Requieren sqlite memory
4. **Deduplication tests (8-9)** - Requieren CountingTool
5. **Approval tests (10-11)** - Requieren mock de ApprovalManager
6. **Cancellation tests (15-16)** - Requieren CancellationToken
7. **Compaction tests (17-18)** - Requieren LLM fallback mock
8. **Parallel test (19)** - Depende de approval gate

---

## Validación

```bash
# Ejecutar solo los nuevos tests
cargo test --lib agent::tests::credential
cargo test --lib agent::tests::history
cargo test --lib agent::tests::deduplication
# ... etc

# Suite completa
cargo test --lib
```

---

## Notas

- Tests 10 y 11 requieren mock de `ApprovalManager` - ver `src/agent/approval.rs`
- Test 16 (warmup failure) es más un warning que un bug real
- Test 18 (compaction compounds) puede requerir múltiples turns

---

## Métricas Objetivo

| Métrica | Antes | Después |
|---------|-------|---------|
| Business logic coverage | ~60% | ~90% |
| Edge case tests | 0 | 19 |
| Unknown bug surface | Alta | Baja |
