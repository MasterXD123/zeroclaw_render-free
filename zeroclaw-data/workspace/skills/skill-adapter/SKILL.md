# Guía de Adaptación de Skills para ZeroClaw

## Estructura de una Skill ZeroClaw

```
skills/
├── mi-skill/
│   ├── SKILL.toml    ← Obligatorio: Metadatos + herramientas
│   └── SKILL.md     ← Obligatorio: Descripción + comportamiento
```

---

## Formato SKILL.toml

```toml
[skill]
name = "nombre-de-la-skill"
description = "Qué hace esta skill en una frase"
version = "1.0.0"
author = "tu-nombre"
tags = ["tag1", "tag2", "categoria"]

# Herramientas que define la skill (opcional)
[[tools]]
name = "nombre_del_comando"
description = "Qué hace este comando"
kind = "shell"  # Tipos: shell, http, script
command = "echo hola"
```

---

## Formato SKILL.md

```markdown
# Nombre de la Skill

## Descripción
Explicación más detallada de qué hace la skill.

## Activación
Se activa SOLO cuando el usuario menciona:
- "palabra clave 1"
- "palabra clave 2"

**NO se activa con:** "hola", "ayuda", "que puedes hacer"

## Comportamiento

### Cuando se activa:
1. Paso 1 - hace esto
2. Paso 2 - hace esto otro
3. Paso 3 - devuelve resultado

### Ejemplo de uso:
Usuario: "palabra clave"
Yo: "Respuesta..."

## Notas
- Cosas importantes a saber
- Limitaciones
```

---

## Guía de Adaptación por Plataforma

### 1. De OpenAI GPTs / Actions

**Original (JSON/Action):**
```json
{
  "name": "weather_tool",
  "description": "Get weather for a location",
  "parameters": {
    "type": "object",
    "properties": {
      "location": {"type": "string"}
    }
  }
}
```

**A ZeroClaw:**
```toml
[[tools]]
name = "weather"
description = "Get weather for a location"
kind = "shell"
command = "curl \"https://api.weather.com?q=$LOCATION\""
```

### 2. De LangChain Tools

**Original (Python):**
```python
@tool
def search(query: str) -> str:
    """Search for information."""
    return search_api(query)
```

**A ZeroClaw:**
```toml
[[tools]]
name = "search"
description = "Search for information on the web"
kind = "shell"
command = "curl -s \"https://api.search.com?q=$QUERY\""
```

### 3. De AutoGen / CrewAI

**Original (YAML/Config):**
```yaml
tools:
  - name: code_executor
    description: Execute Python code
    type: function
```

**A ZeroClaw:**
```toml
[[tools]]
name = "code_executor"
description = "Execute Python code in sandbox"
kind = "shell"
command = "python3 -c \"$CODE\""
```

### 4. De n8n Nodes

**Original (Node JSON):**
```json
{
  "name": "Slack",
  "description": "Send message to Slack",
  "properties": [
    {"name": "webhook", "type": "string"}
  ]
}
```

**A ZeroClaw:**
```toml
[[tools]]
name = "slack_send"
description = "Send message to Slack via webhook"
kind = "http"
command = "curl -X POST $SLACK_WEBHOOK -d '{\"text\":\"$MESSAGE\"}'"
```

### 5. De CustomGPT / Cursor Rules

**Original (Instructions):**
```
You are a code reviewer. When user asks to review code:
1. Read the file
2. Check for bugs
3. Suggest improvements
```

**A ZeroClaw:**
```markdown
# Code Reviewer Skill

## Descripción
Revisa código y sugiere mejoras.

## Activación
Se activa cuando el usuario dice:
- "revisa este código"
- "review code"
- "analiza el archivo"

## Comportamiento
1. Leo el archivo que me pasen
2. Identifico bugs y issues
3. Sugiero mejoras con ejemplos
```

---

## Comandos Shell Permitidos

En ZeroClaw podés usar:

| Categoría | Comandos |
|-----------|----------|
| **Archivos** | ls, cat, grep, find, wc, head, tail |
| **Red** | curl, wget |
| **Git** | git (status, clone, pull, add, commit) |
| **Dev** | npm, node, python, pip, cargo |
| **Sistema** | date, whoami, hostname, ps, df, free |

**⚠️ No permitido:** `rm -rf /`, `dd`, `mkfs`, etc. (bloqueado por security policy)

---

## Ejemplo Completo: Adaptar "Weather Skill"

### Original (supongamos que es de un GPT):
```json
{
  "name": "get_weather",
  "description": "Get current weather for a city",
  "parameters": {"city": "string"}
}
```

### Adaptado a ZeroClaw:

**SKILL.toml:**
```toml
[skill]
name = "weather"
description = "Obtiene el clima de una ciudad"
version = "1.0.0"
author = "adapted"
tags = ["weather", "api", "utilidad"]

[[tools]]
name = "get_weather"
description = "Obtiene el clima actual de una ciudad"
kind = "shell"
command = "curl -s \"wttr.in/$CITY?format=%c%t+%w\""
```

**SKILL.md:**
```markdown
# Weather - Obtener Clima

## Descripción
Muestra el clima actual de una ciudad usando wttr.in (no requiere API key).

## Activación
Se activa cuando el usuario dice:
- "qué clima hace en [ciudad]"
- "clima [ciudad]"
- "weather in [city]"

## Ejemplos
- Usuario: "clima en Buenos Aires"
- Yo: "☁️ 18°C con viento suave en Buenos Aires"

## Notas
- No requiere API key
- Usa wttr.in (gratuito)
- Funciona offline con cached data
```

---

## Tips para Adaptación

1. **Usa variables de entorno** para API keys: `$API_KEY`
2. **Prefiere APIs gratuitas** cuando sea posible
3. **Usa shell chaining con precaución** - algunos están bloqueados
4. **Mantén descripciones claras** en SKILL.md
5. **Testea localmente** antes de instalar

---

## Instalar una Skill Adaptada

```bash
# Desde archivo local
zeroclaw skills install ./mi-skill/

# Desde GitHub
zeroclaw skills install https://github.com/usuario/mi-skill
```