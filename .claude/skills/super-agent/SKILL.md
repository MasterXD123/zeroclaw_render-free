---
name: super-agent
description: "SuperAgent autonomous workflow engine for ZeroClaw. Activa cuando el usuario quiere ejecución 100% autónoma, sin supervisión, planificando y ejecutando tareas complejas de principio a fin. Indicios: 'hazlo solo', 'no me molestes hasta que termines', 'resuélvelo', 'autonomous', 'plan and execute', 'delegar', 'complejo', 'múltiples pasos', 'objetivo largo'. NO activar para preguntas simples o tareas de un solo paso."
---

# SuperAgent — Autonomous Agent Skill

## Overview

SuperAgent es un sistema de **inteligencia estructural autónoma** que descompone objetivos complejos en tareas ejecutables, las ejecuta con monitoreo, y reporta resultados — sin depender del LLM para razonar paso a paso.

La inteligencia viene de la **arquitectura**, no del modelo.

---

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────┐
│                     SUPERAGENT ORCHESTRATOR                 │
│                                                             │
│   USER INPUT ──→ [PLANNER] ──→ [EXECUTOR] ──→ [MONITOR]   │
│                      ↓               ↓            ↓         │
│                   PLAN           TASK LOOP         │        │
│                   ↓                  ↓            │        │
│              ¿APROBADO?          resultados  ───→ [REFLECTOR] │
│                  ↓                   ↓                 ↓    │
│              SI → EXECUTE        ¿error?          EVALUACIÓN  │
│              NO → FEEDBACK      SI → RETRY              ↓    │
│                                NO → NEXT            REPORTE  │
└─────────────────────────────────────────────────────────────┘
```

---

## Roles del Sistema

### PLANNER
- Recibe el objetivo
- Descompone en tareas atómicas
- Resuelve dependencias
- Genera plan estructurado
- **Archivos:** `PLANNER.md`

### EXECUTOR
- Recibe plan del PLANNER
- Ejecuta tareas en orden
- Usa tools de ZeroClaw
- Reporta resultados
- **Archivos:** `EXECUTOR.md`

### MONITOR
- Valida cada output de tarea
- Detecta errores
- Decide: retry, skip, o abort
- **Archivos:** `MONITOR.md`

### REFLECTOR
- Evalúa resultado final
- Compara con objetivo original
- Genera reporte de ejecución
- Sugiere mejoras
- **Archivos:** `REFLECTOR.md`

---

## Diagrama de Flujo Completo

```
INICIO: Usuario entrega objetivo
    │
    ▼
┌──────────────────────┐
│      PLANNER         │
│  ¿Información OK?    │
│  NO → PREGUNTAR ────→│
│  SI → GENERAR PLAN   │
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│    MOSTRAR PLAN      │
│  Usuario APPROVES?   │
│  NO → FEEDBACK ─────→│
│  SI → CONTINUAR      │
└──────────┬───────────┘
           ▼
┌──────────────────────────────────────┐
│           EXECUTOR LOOP              │
│                                      │
│  ┌─ get next task from plan         │
│  └─ execute with tools               │
│         ▼                            │
│  ┌─ MONITOR validates output        │
│  │  │                              │
│  │  ├─ OK? → next task            │
│  │  ├─ RETRY? → retry task        │
│  │  └─ FAIL? → escalate to REFLECT│
│  │                                  │
│  └─ ¿more tasks? NO → EXIT LOOP    │
└──────────┬───────────────────────────┘
           ▼
┌──────────────────────┐
│      REFLECTOR      │
│  Evalúa resultados  │
│  Compara con meta   │
│  Genera reporte     │
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│      REPORTE         │
│  Mostrar al usuario │
│  Fin del proceso    │
└──────────────────────┘
```

---

## Cómo Activar

### Triggers que activan SuperAgent:

- "hazlo solo"
- "no me molestes"
- "resuélvelo"
- "autonomous"
- "plan and execute"
- "delegar"
- "complejo"
- "múltiples pasos"
- "objetivo largo"
- Cualquier objetivo que requiera más de 3 pasos

### Triggers que NO deben activar:

- Preguntas simples ("qué hora es")
- Tareas de un solo paso ("léeme este archivo")
- Configuraciones triviales
- Consultas de información

---

## Uso del Sistema

### Paso 1: Recibir objetivo del usuario

El usuario describe el objetivo de forma libre.

**PLANNER** recibe este input y comienza el ciclo.

### Paso 2: PLANNER genera plan

Ver `PLANNER.md` para el proceso completo.

El plan se muestra al usuario para aprobación.

### Paso 3: Usuario approves o da feedback

```
A) APRUEBA → El plan se guarda en memoria y se pasa a EXECUTOR
B) FEEDBACK → PLANNER ajusta el plan según comentarios
```

### Paso 4: EXECUTOR ejecuta

Ver `EXECUTOR.md` para detalles de ejecución.

Cada tarea es ejecutada y monitoreada por MONITOR.

### Paso 5: REFLECTOR evalúa

Cuando el loop termina, REFLECTOR genera el reporte final.

### Paso 6: Reporte al usuario

El usuario recibe un reporte estructurado con:
- Qué se hizo
- Qué resultó
- Cuánto tomó
- Próximos pasos (si hay)

---

## Tools Disponibles

Ver `TOOLKIT.md` para la lista completa.

### Tools críticas para SuperAgent:

| Tool | Propósito |
|------|-----------|
| `task_tracker` | Crear y trackear tareas del plan |
| `calculator` | Cálculos si el objetivo requiere math |
| `shell` | Ejecución general |
| `file_read/write/edit` | Manipulación de archivos |
| `memory_store/recall` | Guardar estado y contexto |

### Tools extendidas:

| Tool | Propósito |
|------|-----------|
| `code_runner` | Ejecutar Python/JS sandboxed |
| `decision` | Evaluar branching logic |
| `validator` | Validar outputs contra schema |
| `time_tracker` | Medir duración de tareas |

---

## Configuración de Autonomy

Para que SuperAgent funcione al máximo:

```toml
[autonomy]
level = "full"          # "read_only" | "supervised" | "full"

[security]
# En level "full", todas las tools están habilitadas
# excepto las marked como dangerous en security policy
```

### Niveles de autonomía:

| Level | Comportamiento |
|-------|----------------|
| `read_only` | Solo lectura, no ejecuta actions |
| `supervised` | Ejecuta pero pide confirmación para actions críticas |
| `full` | Ejecuta todo sin confirmación, reporta después |

---

## Memory Usage

El sistema usa la memoria de ZeroClaw para persistir:

- **Planes activos** — `key: "plan/[uuid]"`, category: `"planning"`
- **Estado de tareas** — `key: "task/[task_id]"`, category: `"execution"`
- **Contexto compartido** — `key: "context/[plan_id]"`, category: `"context"`

Ver `REFLECTOR.md` para cleanup de memoria post-ejecución.

---

## Manejo de Errores

### Errores en PLANNING:
- Falta de información → Preguntar al usuario
- Objetivo ambiguo → Pedir clarificación
- No ejecutable → Explicar por qué, sugerir alternativas

### Errores en EXECUTION:
- Task falla → MONITOR decide: retry, skip, o abort
- Tool no disponible → Usar tool alternativa o escalar a REFLECTOR
- Timeout → Cancelar tarea, reportar, continuar si es posible

### Errores en REFLECTION:
- Objetivo no alcanzado → Explicar qué pasó, sugerir retry
- Resultado parcial → Mostrar qué se hizo y qué falta

---

## Integración con ZeroClaw

SuperAgent usa la arquitectura existente de ZeroClaw:

- **Tools** → Implementadas como `dyn Tool` en `src/tools/`
- **Memory** → Backend existente de ZeroClaw
- **Security** → Security policy de ZeroClaw
- **Channels** → Notificaciones opcionales al usuario

### Skills relacionadas:

- `zeroclaw` — Si necesitas consultar estado del sistema
- `github-issue` — Para crear issues de seguimiento automático

---

## Metadata

- **Versión:** 1.0
- **Fecha:** 2026-04-07
- **Autor:** SuperAgent Architecture
- **Dependencias:** task_tracker, calculator (críticas)

---

## Archivos de la Skill

| Archivo | Propósito |
|---------|----------|
| `SKILL.md` | Este archivo — activación y coordinación |
| `SUPER-AGENT.md` | System prompt principal |
| `PLANNER.md` | Lógica de descomposición de objetivos |
| `EXECUTOR.md` | Orquestación de ejecución de tareas |
| `MONITOR.md` | Validación y manejo de errores |
| `REFLECTOR.md` | Evaluación y reporte final |
| `TOOLKIT.md` | Mapeo de tools disponibles |

---

*SuperAgent v1.0 — Autonomous Workflow Engine*
