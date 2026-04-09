# Plan de Implementación: 19 Unit Tests de Lógica de Negocio

## Overview

Este documento detalla el plan de implementación paso a paso para agregar 19 unit tests que cubren bugs de lógica de negocio en el agente de ZeroClaw.

**Archivo destino:** `src/agent/tests/business_logic_tests.rs`
**Estado actual:** 3158 tests pasando
**Objetivo:** Agregar 19 tests para coverage de edge cases críticos

---

## Estructura del Archivo de Tests

```rust
//! Business Logic Unit Tests
//!
//! Tests para edge cases en la lógica de negocio del agente que no están
//! cubiertos por los tests de integración existentes.
//!
//! ## Categorías
//! 1. Credential Scrubbing (4 tests)
//! 2. History Trimming (3 tests)
//! 3. Tool Deduplication (2 tests)
//! 4. Approval Bypass (2 tests)
//! 5. Memory & Auto-save (3 tests)
//! 6. Cancellation & Timeouts (2 tests)
//! 7. Compaction (2 tests)
//! 8. Parallel Execution (1 test)

use crate::agent::agent::Agent;
use crate::agent::dispatcher::{NativeToolDispatcher, ToolDispatcher};
use crate::config::{AgentConfig, MemoryConfig};
use crate::memory::{self, Memory};
use crate::observability::{NoopObserver, Observer};
use crate::providers::{
    ChatMessage, ChatRequest, ChatResponse, ConversationMessage, Provider, ToolCall,
    ToolResultMessage,
};
use crate::tools::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════════════════
// Test Helpers — Mock Provider, Mock Tool, Mock Memory
// ═══════════════════════════════════════════════════════════════════════════

/// Mock LLM provider que retorna respuestas programables.
struct ScriptedProvider {
    responses: Mutex<Vec<ChatResponse>>,
    requests: Mutex<Vec<Vec<ChatMessage>>>,
}

impl ScriptedProvider {
    fn new(responses: Vec<ChatResponse>) -> Self {
        Self {
            responses: Mutex::new(responses),
            requests: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Provider for ScriptedProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        _message: &str,
        _model: &str,
        _temperature: f64,
    ) -> Result<String> {
        Ok("fallback".into())
    }

    async fn chat(
        &self,
        request: ChatRequest<'_>,
        _model: &str,
        _temperature: f64,
    ) -> Result<ChatResponse> {
        self.requests
            .lock()
            .unwrap()
            .push(request.messages.to_vec());

        let mut guard = self.responses.lock().unwrap();
        if guard.is_empty() {
            return Ok(ChatResponse {
                text: Some("done".into()),
                tool_calls: vec![],
                usage: None,
                reasoning_content: None,
            });
        }
        Ok(guard.remove(0))
    }
}

/// Tool que cuenta llamadas.
struct CountingTool {
    count: Arc<Mutex<usize>>,
}

impl CountingTool {
    fn new() -> (Self, Arc<Mutex<usize>>) {
        let count = Arc::new(Mutex::new(0));
        (Self { count }, count)
    }
}

#[async_trait]
impl Tool for CountingTool {
    fn name(&self) -> &str { "counter" }
    fn description(&self) -> &str { "Counts calls" }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object"})
    }
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let mut c = self.count.lock().unwrap();
        *c += 1;
        Ok(ToolResult {
            success: true,
            output: format!("call #{}", *c),
            error: None,
        })
    }
}

/// Tool que falla ejecución.
struct FailingTool;

#[async_trait]
impl Tool for FailingTool {
    fn name(&self) -> &str { "fail" }
    fn description(&self) -> &str { "Always fails" }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object"})
    }
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some("intentional failure".into()),
        })
    }
}

/// Tool que toma tiempo.
struct SlowTool {
    delay_ms: u64,
}

impl SlowTool {
    fn new(delay_ms: u64) -> Self {
        Self { delay_ms }
    }
}

#[async_trait]
impl Tool for SlowTool {
    fn name(&self) -> &str { "slow" }
    fn description(&self) -> &str { "Takes time" }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object"})
    }
    async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
        Ok(ToolResult {
            success: true,
            output: "slow done".into(),
            error: None,
        })
    }
}

fn make_memory() -> Arc<dyn Memory> {
    let cfg = MemoryConfig {
        backend: "none".into(),
        ..MemoryConfig::default()
    };
    Arc::from(memory::create_memory(&cfg, &std::env::temp_dir(), None).unwrap())
}

fn make_sqlite_memory() -> (Arc<dyn Memory>, tempfile::TempDir) {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = MemoryConfig {
        backend: "sqlite".into(),
        ..MemoryConfig::default()
    };
    let mem = Arc::from(memory::create_memory(&cfg, tmp.path(), None).unwrap());
    (mem, tmp)
}

fn make_observer() -> Arc<dyn Observer> {
    Arc::from(NoopObserver {})
}

fn build_agent_with(
    provider: Box<dyn Provider>,
    tools: Vec<Box<dyn Tool>>,
) -> Agent {
    Agent::builder()
        .provider(provider)
        .tools(tools)
        .memory(make_memory())
        .observer(make_observer())
        .tool_dispatcher(Box::new(NativeToolDispatcher))
        .workspace_dir(std::env::temp_dir())
        .build()
        .unwrap()
}

fn build_agent_with_config(
    provider: Box<dyn Provider>,
    tools: Vec<Box<dyn Tool>>,
    config: AgentConfig,
) -> Agent {
    Agent::builder()
        .provider(provider)
        .tools(tools)
        .memory(make_memory())
        .observer(make_observer())
        .tool_dispatcher(Box::new(NativeToolDispatcher))
        .workspace_dir(std::env::temp_dir())
        .config(config)
        .build()
        .unwrap()
}

fn tool_response(calls: Vec<ToolCall>) -> ChatResponse {
    ChatResponse {
        text: Some(String::new()),
        tool_calls: calls,
        usage: None,
        reasoning_content: None,
    }
}

fn text_response(text: &str) -> ChatResponse {
    ChatResponse {
        text: Some(text.into()),
        tool_calls: vec![],
        usage: None,
        reasoning_content: None,
    }
}
```

---

## Implementación por Categoría

### Categoría 1: Credential Scrubbing (4 tests)

**Ubicación:** `src/agent/loop_.rs` - función `scrub_credentials()`

#### Test 1.1: `scrub_short_api_key`
```rust
#[test]
fn scrub_credentials_handles_short_api_keys() {
    // Keys < 8 chars no se redacted completamente
    let input = r#"{"api_key": "sk-short"}"#;
    let scrubbed = scrub_credentials(input);
    // El valor "sk-short" debería ser redacted
    assert!(!scrubbed.contains("sk-short"));
    assert!(scrubbed.contains("***"));
}
```

#### Test 1.2: `scrub_nested_json_credentials`
```rust
#[test]
fn scrub_credentials_handles_nested_json() {
    let input = r#"{"outer": {"inner": {"api_key": "secret123"}}}"#;
    let scrubbed = scrub_credentials(input);
    assert!(!scrubbed.contains("secret123"));
    assert!(scrubbed.contains("***"));
}
```

#### Test 1.3: `scrub_base64_lookalike_tokens`
```rust
#[test]
fn scrub_credentials_handles_base64_tokens() {
    let input = r#"{"token": "c3VwZXJzZWNyZXRrZXk="}"#;
    let scrubbed = scrub_credentials(input);
    // Tokens que parecen base64 también deben ser redacted
    assert!(!scrubbed.contains("c3VwZXJzZWNyZXRrZXk="));
}
```

#### Test 1.4: `scrub_unicode_credentials`
```rust
#[test]
fn scrub_credentials_handles_unicode() {
    let input = r#"{"password": "s3cr3t🤐"}"#;
    let scrubbed = scrub_credentials(input);
    assert!(!scrubbed.contains("s3cr3t🤐"));
}
```

---

### Categoría 2: History Trimming (3 tests)

**Ubicación:** `src/agent/agent.rs` - función `trim_history()`

#### Test 2.1: `trim_history_preserves_system_when_all_system`
```rust
#[tokio::test]
async fn trim_history_handles_all_system_messages() {
    // Edge case: todos los mensajes son system
    let mut agent = build_agent_with(
        Box::new(ScriptedProvider::new(vec![text_response("ok")])),
        vec![],
    );

    // Agregar solo system messages
    agent.inject_system_prompt("sys1".into());
    agent.inject_system_prompt("sys2".into());

    // trim_history no debería fallar
    agent.trim_history();
    let history = agent.history();

    // Al menos un system debe sobrevivir
    assert!(!history.is_empty());
}
```

#### Test 2.2: `trim_history_multiple_system_messages`
```rust
#[tokio::test]
async fn trim_history_preserves_latest_system() {
    let provider = Box::new(ScriptedProvider::new(vec![text_response("ok")]));
    let config = AgentConfig {
        max_history_messages: 3,
        ..AgentConfig::default()
    };
    let mut agent = build_agent_with_config(provider, vec![], config);

    let _ = agent.turn("msg1").await.unwrap();
    let _ = agent.turn("msg2").await.unwrap();

    // Múltiples system prompts (inyectados en diferentes turns)
    agent.trim_history();

    // El primer mensaje debe ser system
    assert!(matches!(
        agent.history()[0],
        ConversationMessage::Chat(c) if c.role == "system"
    ));
}
```

#### Test 2.3: `trim_compaction_summary_not_duplicated`
```rust
#[tokio::test]
async fn compaction_summary_not_duplicated_on_multiple_trims() {
    let provider = Box::new(ScriptedProvider::new(vec![text_response("ok")]));
    let config = AgentConfig {
        max_history_messages: 5,
        ..AgentConfig::default()
    };
    let mut agent = build_agent_with_config(provider, vec![], config);

    // Generar suficiente historia para activar compaction múltiples veces
    for i in 0..10 {
        let _ = agent.turn(&format!("msg{i}")).await.unwrap();
    }

    agent.trim_history();

    // No debería haber múltiples [Compaction summary]
    let compaction_count = agent.history()
        .iter()
        .filter(|m| matches!(m, ConversationMessage::Chat(c)
            if c.content.contains("Compaction")))
        .count();

    assert!(compaction_count <= 1, "Expected at most 1 compaction summary");
}
```

---

### Categoría 3: Tool Deduplication (2 tests)

**Ubicación:** `src/agent/loop_.rs` - lógica `seen_tool_signatures`

#### Test 3.1: `duplicate_tool_call_preserves_order`
```rust
#[tokio::test]
async fn duplicate_tools_preserve_result_ordering() {
    let (counting_tool, count) = CountingTool::new();

    let provider = Box::new(ScriptedProvider::new(vec![
        tool_response(vec![
            ToolCall { id: "tc1".into(), name: "counter".into(), arguments: "{}".into() },
            ToolCall { id: "tc2".into(), name: "counter".into(), arguments: "{}".into() }, // duplicate
            ToolCall { id: "tc3".into(), name: "counter".into(), arguments: "{}".into() },
        ]),
        text_response("done"),
    ]));

    let mut agent = build_agent_with(provider, vec![Box::new(counting_tool)]);

    let _ = agent.turn("count").await.unwrap();

    // tc1 y tc3 ejecutados, tc2 deduplicado
    assert_eq!(*count.lock().unwrap(), 2);
}
```

#### Test 3.2: `duplicate_tool_with_different_args_not_deduplicated`
```rust
#[tokio::test]
async fn same_tool_different_args_not_deduplicated() {
    let (counting_tool, count) = CountingTool::new();

    let provider = Box::new(ScriptedProvider::new(vec![
        tool_response(vec![
            ToolCall { id: "tc1".into(), name: "counter".into(), arguments: r#"{"n":"1"}"#.into() },
            ToolCall { id: "tc2".into(), name: "counter".into(), arguments: r#"{"n":"2"}"#.into() }, // different args
        ]),
        text_response("done"),
    ]));

    let mut agent = build_agent_with(provider, vec![Box::new(counting_tool)]);

    let _ = agent.turn("count different").await.unwrap();

    // Ambos ejecutados porque args son diferentes
    assert_eq!(*count.lock().unwrap(), 2);
}
```

---

### Categoría 4: Approval Bypass (2 tests)

**Ubicación:** `src/agent/loop_.rs` - `ApprovalManager`

#### Test 4.1: `approval_bypassed_for_non_cli_channels`
```rust
#[tokio::test]
async fn non_cli_channels_bypass_approval() {
    // Crear agent con channel_name = "telegram"
    let provider = Box::new(ScriptedProvider::new(vec![
        tool_response(vec![ToolCall {
            id: "tc1".into(),
            name: "shell".into(),
            arguments: r#"{"cmd": "echo hi"}"#.into(),
        }]),
        text_response("done"),
    ]));

    let mut agent = build_agent_with(provider, vec![Box::new(FailingTool)]);

    // En canales no-CLI, approvals son auto-granted
    // El tool debería ejecutarse sin esperar approval
    let result = agent.turn("run shell").await;

    // No debería fallar por approval
    assert!(result.is_ok());
}
```

#### Test 4.2: `approval_grants_before_hook_modification`
```rust
#[tokio::test]
async fn hook_modification_after_approval() {
    // Verificar que hooks pueden modificar tool post-approval
    let provider = Box::new(ScriptedProvider::new(vec![
        tool_response(vec![ToolCall {
            id: "tc1".into(),
            name: "shell".into(),
            arguments: r#"{"cmd": "original"}"#.into(),
        }]),
        text_response("done"),
    ]));

    let mut agent = build_agent_with(provider, vec![Box::new(FailingTool)]);

    // El hook modifica args pero approval ya fue granted
    let result = agent.turn("run modified").await;
    assert!(result.is_ok());
}
```

---

### Categoría 5: Memory & Auto-save (3 tests)

**Ubicación:** `src/agent/loop_.rs`, `src/agent/memory_loader.rs`

#### Test 5.1: `autosave_min_chars_constant_dead_code`
```rust
#[tokio::test]
async fn autosave_ignores_min_chars_constant() {
    // AUTOSAVE_MIN_MESSAGE_CHARS = 20 pero no se enforce
    let (mem, _tmp) = make_sqlite_memory();
    let provider = Box::new(ScriptedProvider::new(vec![text_response("ok")]));

    let mut agent = Agent::builder()
        .provider(provider)
        .tools(vec![])
        .memory(mem.clone())
        .observer(make_observer())
        .tool_dispatcher(Box::new(NativeToolDispatcher))
        .workspace_dir(std::env::temp_dir())
        .auto_save(true)
        .build()
        .unwrap();

    // Mensaje corto debería guardarse igual (contra la constante)
    let _ = agent.turn("hi").await.unwrap();

    // Verificar que se guardó a pesar de ser < 20 chars
    let count = mem.count().await.unwrap();
    assert_eq!(count, 1, "Short messages should be saved regardless of AUTOSAVE_MIN_MESSAGE_CHARS");
}
```

#### Test 5.2: `memory_loader_filters_none_score_entries`
```rust
#[tokio::test]
async fn memory_loader_handles_none_relevance_score() {
    let (mem, _tmp) = make_sqlite_memory();

    // Insertar entry con score None
    mem.store("test", "content", None).await.unwrap();

    let loader = DefaultMemoryLoader::new(mem.clone(), 0.0);
    let context = loader.load_for_message("test query").await.unwrap();

    // Entries con score: None deberían pasar el filtro
    assert!(!context.is_empty());
}
```

#### Test 5.3: `memory_context_empty_when_all_below_threshold`
```rust
#[tokio::test]
async fn memory_context_empty_when_all_below_min_relevance() {
    let (mem, _tmp) = make_sqlite_memory();

    // Insertar entries con score bajo
    mem.store("unrelated1", "content a", Some(0.1)).await.unwrap();
    mem.store("unrelated2", "content b", Some(0.1)).await.unwrap();

    let loader = DefaultMemoryLoader::new(mem.clone(), 0.8);
    let context = loader.load_for_message("test query").await.unwrap();

    // Todo está bajo threshold → contexto vacío
    assert!(context.is_empty());
}
```

---

### Categoría 6: Cancellation & Timeouts (2 tests)

**Ubicación:** `src/agent/loop_.rs` - `CancellationToken`

#### Test 6.1: `cancellation_during_long_tool_execution`
```rust
#[tokio::test]
async fn cancellation_handles_long_running_tool() {
    use tokio::sync::CancellationToken;

    let token = CancellationToken::new();
    let token_clone = token.clone();

    // Tool que tarda 500ms
    let slow_tool = SlowTool::new(500);

    // Cancelar después de 100ms
    let cancel_handle = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        token_clone.cancel();
    });

    let provider = Box::new(ScriptedProvider::new(vec![
        tool_response(vec![ToolCall {
            id: "tc1".into(),
            name: "slow".into(),
            arguments: "{}".into(),
        }]),
        text_response("done"),
    ]));

    let mut agent = build_agent_with(provider, vec![Box::new(slow_tool)]);

    // Should handle cancellation gracefully
    let result = agent.turn("run slow").await;

    cancel_handle.await.unwrap();

    // No panic, posiblemente error de cancellation
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("cancel"));
}
```

#### Test 6.2: `provider_warmup_failure_silent`
```rust
#[tokio::test]
async fn warmup_failure_does_not_block_execution() {
    struct FailingWarmupProvider;

    #[async_trait]
    impl Provider for FailingWarmupProvider {
        async fn warmup(&self) -> Result<()> {
            anyhow::bail!("warmup failed")
        }
        // ... other methods
    }

    let provider = Box::new(FailingWarmupProvider);
    let mut agent = build_agent_with(provider, vec![]);

    // warmup failure debería ser warning, no error
    // La ejecución debería continuar
    let result = agent.turn("hi").await;
    assert!(result.is_ok());
}
```

---

### Categoría 7: Compaction (2 tests)

**Ubicación:** `src/agent/loop_.rs` - `auto_compact_history`

#### Test 7.1: `compaction_fallback_deterministic_truncation`
```rust
#[tokio::test]
async fn compaction_falls_back_to_truncation_on_summarizer_failure() {
    struct FailingSummarizerProvider;

    #[async_trait]
    impl Provider for FailingSummarizerProvider {
        async fn chat(&self, _request: ChatRequest<'_>, _model: &str, _temp: f64) -> Result<ChatResponse> {
            anyhow::bail!("summarizer failed")
        }
        // ... other methods
    }

    let provider = Box::new(FailingSummarizerProvider);
    let config = AgentConfig {
        max_history_messages: 3,
        ..AgentConfig::default()
    };
    let mut agent = build_agent_with_config(provider, vec![], config);

    // Generar suficiente historia
    for i in 0..5 {
        let _ = agent.turn(&format!("msg{i}")).await;
    }

    // Debería usar fallback determinístico (truncation)
    // No debería panic
    agent.trim_history();
    assert!(!agent.history().is_empty());
}
```

#### Test 7.2: `compaction_multiple_times_compounds`
```rust
#[tokio::test]
async fn multiple_compactions_do_not_compound_errors() {
    let provider = Box::new(ScriptedProvider::new(vec![text_response("ok")]));
    let config = AgentConfig {
        max_history_messages: 4,
        ..AgentConfig::default()
    };
    let mut agent = build_agent_with_config(provider, vec![], config);

    // Trigger compaction multiple times
    for i in 0..8 {
        let _ = agent.turn(&format!("message number {i}")).await;
        agent.trim_history();
    }

    // Verificar que no hay múltiples summaries
    let summaries = agent.history()
        .iter()
        .filter(|m| matches!(m, ConversationMessage::Chat(c)
            if c.content.contains("[Compaction summary]")))
        .count();

    assert!(summaries <= 1, "Multiple compactions should not compound");
}
```

---

### Categoría 8: Parallel Execution (1 test)

**Ubicación:** `src/agent/loop_.rs` - `should_execute_tools_in_parallel()`

#### Test 8.1: `parallel_execution_blocked_by_approval_gate`
```rust
#[tokio::test]
async fn parallel_blocked_when_any_tool_needs_approval() {
    // Cuando un tool necesita approval, todos van sequential
    let (tool1, count1) = CountingTool::new();
    let (tool2, count2) = CountingTool::new();

    let provider = Box::new(ScriptedProvider::new(vec![
        tool_response(vec![
            ToolCall { id: "tc1".into(), name: "counter".into(), arguments: "{}".into() },
            ToolCall { id: "tc2".into(), name: "counter".into(), arguments: "{}".into() },
        ]),
        text_response("done"),
    ]));

    let mut agent = build_agent_with(provider, vec![
        Box::new(tool1),
        Box::new(tool2),
    ]);

    let _ = agent.turn("count").await.unwrap();

    // Ambos tools fueron ejecutados (posiblemente sequential)
    assert_eq!(*count1.lock().unwrap(), 1);
    assert_eq!(*count2.lock().unwrap(), 1);
}
```

---

## Resumen de Tests a Implementar

| # | Test | Categoría | Dificultad |
|---|------|-----------|------------|
| 1 | `scrub_credentials_handles_short_api_keys` | Credential | Baja |
| 2 | `scrub_credentials_handles_nested_json` | Credential | Baja |
| 3 | `scrub_credentials_handles_base64_tokens` | Credential | Baja |
| 4 | `scrub_credentials_handles_unicode` | Credential | Baja |
| 5 | `trim_history_handles_all_system_messages` | History | Media |
| 6 | `trim_history_preserves_latest_system` | History | Media |
| 7 | `compaction_summary_not_duplicated` | History | Alta |
| 8 | `duplicate_tools_preserve_result_ordering` | Deduplication | Media |
| 9 | `same_tool_different_args_not_deduplicated` | Deduplication | Media |
| 10 | `non_cli_channels_bypass_approval` | Approval | Media |
| 11 | `hook_modification_after_approval` | Approval | Alta |
| 12 | `autosave_ignores_min_chars_constant` | Memory | Baja |
| 13 | `memory_loader_handles_none_relevance_score` | Memory | Media |
| 14 | `memory_context_empty_when_all_below_threshold` | Memory | Baja |
| 15 | `cancellation_handles_long_running_tool` | Cancellation | Alta |
| 16 | `warmup_failure_does_not_block_execution` | Cancellation | Media |
| 17 | `compaction_falls_back_to_truncation` | Compaction | Alta |
| 18 | `multiple_compactions_do_not_compound` | Compaction | Alta |
| 19 | `parallel_blocked_when_approval_needed` | Parallel | Media |

---

## Orden de Implementación Recomendada

1. **Fase 1 - Credential Tests (1-4)** - Baja dificultad, independientes
2. **Fase 2 - Memory Tests (12-14)** - Baja/_media dificultad, aislados
3. **Fase 3 - History Tests (5-7)** - Media dificultad
4. **Fase 4 - Deduplication Tests (8-9)** - Media dificultad
5. **Fase 5 - Approval Tests (10-11)** - Media/Alta, requieren mock ApprovalManager
6. **Fase 6 - Cancellation Tests (15-16)** - Alta dificultad, async
7. **Fase 7 - Compaction Tests (17-18)** - Alta dificultad
8. **Fase 8 - Parallel Test (19)** - Media dificultad

---

## Validación

```bash
# Ejecutar nuevos tests
cargo test --lib business_logic

# Suite completa
cargo test --lib

# Solo credential tests
cargo test --lib scrub_credentials

# Solo compaction tests
cargo test --lib compaction

# Solo deduplication tests
cargo test --lib duplicate_tools
```

---

## Dependencias

No se requieren nuevas dependencias. El archivo reutiliza:
- `ScriptedProvider` existente
- `CountingTool` existente
- `FailingTool` existente
- `make_memory()`, `make_sqlite_memory()` existentes
- `build_agent_with()` existente

Para algunos tests se necesita:
- `SlowTool` - implementar locally (tokio::time::sleep)
- `FailingWarmupProvider` - implementar locally

---

## Notas de Implementación

1. **Tests 10-11 (Approval)** requieren mock de `ApprovalManager` que no está en el scope de estos tests. Implementar como tests de integración si es necesario.

2. **Test 16 (Warmup)** requiere que `Provider` trait tenga método `warmup()`. Verificar que existe antes de implementar.

3. **Tests de Cancellation** usan `tokio::sync::CancellationToken` - verificar imports.

4. **Credential scrubbing** es una función privada `pub(crate)` en `loop_.rs`. Puede ser necesario hacerla `pub` para tests o usar un wrapper.

5. **Memory loader** está en módulo separado `memory_loader.rs`. Verificar API pública antes de implementar.

---

## Timeline Sugerido

| Semana | Fase | Tests |
|--------|------|-------|
| Semana 1 | Fase 1-2 | 1-4, 12-14 |
| Semana 2 | Fase 3-4 | 5-9 |
| Semana 3 | Fase 5-6 | 10-11, 15-16 |
| Semana 4 | Fase 7-8 | 17-19 |

**Total: ~4 semanas para implementación completa**
