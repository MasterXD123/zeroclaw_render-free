# Ejemplos de Adaptación de Skills

## Ejemplo 1: OpenAI GPT Action → ZeroClaw

### Original (OpenAI):
```json
{
  "type": "function",
  "function": {
    "name": "get_stock_price",
    "description": "Get the current stock price for a ticker symbol",
    "parameters": {
      "type": "object",
      "properties": {
        "symbol": {
          "type": "string",
          "description": "Stock ticker symbol (e.g., AAPL, GOOGL)"
        }
      },
      "required": ["symbol"]
    }
  }
}
```

### Adaptado (ZeroClaw):
```toml
[[tools]]
name = "get_stock_price"
description = "Get the current stock price for a ticker symbol"
kind = "shell"
command = "curl -s \"https://api.example.com/quote?symbol=$SYMBOL\""
```

---

## Ejemplo 2: LangChain Tool → ZeroClaw

### Original (LangChain Python):
```python
from langchain.tools import tool

@tool
def calculate(expression: str) -> str:
    """Evaluate a mathematical expression."""
    return str(eval(expression))
```

### Adaptado (ZeroClaw):
```toml
[[tools]]
name = "calculate"
description = "Evaluate a mathematical expression"
kind = "shell"
command = "python3 -c \"print($EXPRESSION)\""
```

---

## Ejemplo 3: n8n Node → ZeroClaw

### Original (n8n):
```json
{
  "nodeType": "n8n-nodes-base.httpRequest",
  "parameters": {
    "url": "https://api.telegram.org/bot{{$credentials.botToken}}/sendMessage",
    "method": "POST",
    "bodyParameters": {
      "parameters": [
        {"name": "chat_id", "value": "={{$json.chat_id}}"},
        {"name": "text", "value": "={{$json.message}}"}
      ]
    }
  }
}
```

### Adaptado (ZeroClaw):
```toml
[[tools]]
name = "telegram_send"
description = "Send a message via Telegram"
kind = "shell"
command = "curl -s -X POST \"https://api.telegram.org/bot$TELEGRAM_BOT/sendMessage\" -d \"chat_id=$CHAT_ID&text=$TEXT\""
```

---

## Ejemplo 4: AutoGen Tool → ZeroClaw

### Original (AutoGen):
```python
from autogen import tool

@tool
def run_python(code: str) -> str:
    """Execute Python code and return the output."""
    exec(code, globals())
    return result
```

### Adaptado (ZeroClaw):
```toml
[[tools]]
name = "run_python"
description = "Execute Python code and return output"
kind = "shell"
command = "python3 -c \"$CODE\""
```

---

## Ejemplo 5: Vercel AI SDK Tool → ZeroClaw

### Original (TypeScript):
```typescript
const calculatorTool = {
  name: 'calculator',
  description: 'Perform basic math operations',
  parameters: z.object({
    operation: z.enum(['add', 'subtract', 'multiply', 'divide']),
    a: z.number(),
    b: z.number()
  }),
  execute: async ({ operation, a, b }) => {
    // ... logic
  }
}
```

### Adaptado (ZeroClaw):
```toml
[[tools]]
name = "calculator"
description = "Perform basic math operations (add, subtract, multiply, divide)"
kind = "shell"
command = "python3 -c \"print($A $OP $B)\""
```

---

## Ejemplo 6: Personalidad / System Prompt → Skill

### Original (GPT Persona):
```
You are a helpful Python tutor. You explain concepts clearly 
and provide code examples. You use simple language for beginners.
```

### Adaptado (ZeroClaw - SKILL.md):
```markdown
# Python Tutor

## Descripción
Tutor de Python que explica conceptos de forma clara con ejemplos de código.

## Activación
Se activa cuando el usuario:
- Quiere aprender Python
- Pregunta sobre código Python
- Dice "enséñame Python"

## Comportamiento
1. Explico el concepto con lenguaje simple
2. Proporciono ejemplos de código
3. Sugiero ejercicios prácticos

## Estilo
- Lenguaje accesible para principiantes
- Ejemplos prácticos y relevantes
- Paciente y encouraging
```

---

## Resumen de Adaptación

| Plataforma | Tipo de Origen | Cómo Adaptar |
|------------|----------------|---------------|
| OpenAI | JSON Action | Convertir a shell command con curl |
| LangChain | Python @tool | Convertir a shell command |
| n8n | Node JSON | Convertir HTTP request a curl |
| AutoGen | Python @tool | Convertir a shell command |
| Vercel AI | TypeScript | Convertir a shell command |
| GPTs | System Prompt | Convertir a SKILL.md |

---

## Notas Importantes

1. **API Keys**: Usar variables de entorno (`$API_KEY`) en lugar de hardcodear
2. **Rate Limits**: Considerar límites de API al diseñar la tool
3. **Errores**: Manejar errores con mensajes claros en la skill
4. **Testing**: Probar cada tool antes de installar