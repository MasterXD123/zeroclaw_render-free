# Adaptación de Skills - Guía de Referencia

## ¿Qué es skill-adapter?

Es una skill que te ayuda a convertir skills de otras plataformas al formato ZeroClaw.

## ¿Cómo usarla?

### Paso 1: Identifica la skill a adaptar
Tenés el código/formato de la skill original (de OpenAI, LangChain, n8n, etc.)

### Paso 2: Determina el tipo de skill

| Tipo | Origen | Ejemplo |
|------|--------|---------|
| **Tool** | API call | "Get weather", "Search web" |
| **Action** | Function call | "Send email", "Create file" |
| **Persona** | System prompt | "Actúa como tutor" |
| **Workflow** | Multi-step | "Code review completo" |

### Paso 3: Adapta según la guía

**Para Tools/Actions:**
1. Copiar la lógica a un comando shell
2. Usar `$VAR` para parámetros
3. Preferir APIs gratuitas cuando sea posible

**Para Personas:**
1. Crear SKILL.md con instrucciones claras
2. Definir keywords de activación
3. Describir comportamiento esperado

### Paso 4: Crea los archivos

```
skills/
├── tu-skill/
│   ├── SKILL.toml    # Metadatos + tools
│   └── SKILL.md      # Descripción
```

### Paso 5: Instala

```bash
zeroclaw skills install ./tu-skill/
```

---

## Formatos Soportados

### ✅ Soportados
- OpenAI GPT Actions (JSON)
- LangChain Tools (Python decorators)
- n8n Nodes (JSON)
- AutoGen Tools (Python decorators)
- Vercel AI SDK (TypeScript)
- CustomGPT Instructions
- Cursor Rules
- System Prompts

### 🔄 Requiere Adaptación
- AWS Lambda functions
- Azure Functions
- Google Cloud Functions
- Webhooks genéricos

---

## Estructura Obligatoria

### SKILL.toml (mínimo)
```toml
[skill]
name = "nombre"
description = "descripción"
version = "1.0.0"
```

### SKILL.md (mínimo)
```markdown
# Nombre

## Descripción
Qué hace la skill.

## Activación
Keywords que la activan.
```

---

## Tips de Seguridad

1. **No hardcodear API keys** - Usar `$ENV_VAR`
2. **No usar comandos destructivos** - `rm -rf` está bloqueado
3. **Validar inputs** - No confiar en datos del usuario
4. **Manejar errores** - Devolver mensajes claros

---

## Obtener Ayuda

Si tenés dudas sobre cómo adaptar una skill específica:

1. Mirá los ejemplos en `EXAMPLE.md`
2. Consultá la guía en `ADAPTATION_GUIDE.md`
3. Preguntame directamente con el formato original

## Ejemplo de Pedido

**Vos decís:**
"Tengo esta skill de LangChain, cómo la adapto?"

```python
@tool
def search_wikipedia(query: str) -> str:
    '''Search Wikipedia for a query.'''
    return wikipedia.summary(query)
```

**Yo respondo:**
"Acá está adaptada:

**SKILL.toml:**
```toml
[skill]
name = "wikipedia_search"
description = "Busca en Wikipedia"
version = "1.0.0"

[[tools]]
name = "search"
description = "Search Wikipedia for a query"
kind = "shell"
command = "curl -s \"https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch=$QUERY&format=json\" | jq '.query.search[0].snippet'"
```

**SKILL.md:**
```markdown
# Wikipedia Search

## Descripción
Busca artículos en Wikipedia.

## Activación
- \"busca en wikipedia\"
- \"wikipedia\"
- \"buscar [tema]\"
```"