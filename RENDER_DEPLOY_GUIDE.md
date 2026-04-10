# GUÍA COMPLETA: Deploy ZeroClaw-free en Render

## Tabla de Contenidos
1. [Requisitos Previos](#requisitos-previos)
2. [Configuración de OpenRouter](#configuración-de-openrouter)
3. [Configuración del Proyecto](#configuración-del-proyecto)
4. [Deploy en Render](#deploy-en-render)
5. [Variables de Entorno](#variables-de-entorno)
6. [Verificación](#verificación)
7. [Solución de Problemas](#solución-de-problemas)
8. [Costos](#costos)

---

## 1. REQUISITOS PREVIOS

### Cuentas necesarias:
| Servicio | Enlace | Costo |
|----------|--------|-------|
| GitHub | [github.com](https://github.com) | Gratis |
| Render | [render.com](https://render.com) | Gratis |
| OpenRouter | [openrouter.ai](https://openrouter.ai) | $5 gratis/mes |

### Software necesario:
- Git instalado
- Cuenta de GitHub

---

## 2. CONFIGURACIÓN DE OPENROUTER

### Paso 1: Crear cuenta
1. Ve a [openrouter.ai](https://openrouter.ai)
2. Click "Sign Up" → usa tu cuenta Google o GitHub
3. Verifica tu email

### Paso 2: Obtener API Key
1. En OpenRouter, ve a **Keys** (en el menú lateral)
2. Click **Create Key**
3. Dale un nombre: `zeroclaw-render`
4. Copia la key (comienza con `sk-or-v1-xxxxxxxx`)
5. **IMPORTANTE**: Guarda esta key, solo se muestra una vez

### Paso 3: Verificar créditos
1. Ve a **Credits** en el menú
2. Deberías tener $5 gratis disponibles
3. Si no hay créditos, agrega un método de pago (solo se cobra si usas más de $5)

---

## 3. CONFIGURACIÓN DEL PROYECTO

### Paso 1: Clonar el repositorio
```bash
git clone https://github.com/MasterXD123/zeroclaw-free.git
cd zeroclaw-free
```

### Paso 2: Crear archivo .env
```bash
cp .env.example .env
```

### Paso 3: Editar .env
Abre el archivo `.env` y configura:

```env
# REQUERIDO - Tu API key de OpenRouter (la que copié antes)
OPENAI_API_KEY=sk-or-v1-aquí-va-tu-key-enterita

# Proveedor (dejar como está)
PROVIDER=openrouter

# Modelo recomendado
ZEROCLAW_MODEL=openrouter/google/gemma-4-27b-it

# Seguridad (false = solo localhost, true = accesible desde internet)
ZEROCLAW_ALLOW_PUBLIC_BIND=true

# Puerto (para local, en Render se usa $PORT)
HOST_PORT=42617

# Telegram (OPCIONAL, dejar en blanco si no tienes)
TELEGRAM_BOT_TOKEN=

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
```

### Paso 4: Subir a GitHub
```bash
# En la terminal, dentro de la carpeta zeroclaw-free:

git add .
git commit -m "Ready for Render deployment"
git push origin master
```

---

## 4. DEPLOY EN RENDER

### Paso 1: Crear cuenta en Render
1. Ve a [render.com](https://render.com)
2. Click "Sign Up with GitHub"
3. Autoriza a Render acceso a tu GitHub

### Paso 2: Crear Web Service
1. En el dashboard de Render, click **"New"**
2. Selecciona **"Web Service"**
3. Busca tu repositorio `zeroclaw-free`
4. Click **"Connect"**

### Paso 3: Configurar el servicio

En la página de configuración:

| Campo | Valor |
|-------|-------|
| **Name** | `zeroclaw` |
| **Environment** | `Docker` |
| **Region** | `Frankfurt` (o la más cercana a ti) |
| **Branch** | `master` |

### Paso 4: Configure Build Command

En el campo **Build Command**, escribe:
```bash
docker build -t zeroclaw -f Dockerfile.render .
```

### Paso 5: Configure Run Command

En el campo **Start Command**, escribe:
```bash
docker run -p $PORT:$PORT -e OPENAI_API_KEY -e PROVIDER -e ZEROCLAW_MODEL -e ZEROCLAW_ALLOW_PUBLIC_BIND zeroclaw
```

### Paso 6: Añadir Environment Variables

En la sección **Environment Variables**, añade estas variables:

| Key | Value | Description |
|-----|-------|-------------|
| `OPENAI_API_KEY` | `sk-or-v1-aqui-tu-key` | **TU API KEY DE OPENROUTER** |
| `PROVIDER` | `openrouter` | Proveedor de modelos |
| `ZEROCLAW_MODEL` | `openrouter/google/gemma-4-27b-it` | Modelo a usar |
| `ZEROCLAW_ALLOW_PUBLIC_BIND` | `true` | Permitir acceso público |
| `RUST_LOG` | `info` | Nivel de logs |

**⚠️ IMPORTANTE**: En `OPENAI_API_KEY` debes poner tu key real de OpenRouter (la que empieza con `sk-or-v1-...`)

### Paso 7: Desplegar

1. Click **"Create Web Service"**
2. Espera a que compile (puede tomar 10-15 minutos la primera vez)
3. Verifica en los logs que no haya errores

---

## 5. VARIABLES DE ENTORNO EXPLICADAS

| Variable | Ejemplo | Para qué sirve |
|----------|---------|----------------|
| `OPENAI_API_KEY` | `sk-or-v1-xxxx...` | **Tu clave de OpenRouter** - Obligatoria |
| `PROVIDER` | `openrouter` | Qué proveedor de IA usas |
| `ZEROCLAW_MODEL` | `openrouter/google/gemma-4-27b-it` | Qué modelo de IA |
| `ZEROCLAW_ALLOW_PUBLIC_BIND` | `true` | Si=true, accesible desde internet |
| `PORT` | `10000` | Render lo asigna automáticamente |
| `TELEGRAM_BOT_TOKEN` | (vacío o tu token) | Para usar Telegram (opcional) |
| `RUST_LOG` | `info` | Cuánto detalle en logs |

---

## 6. VERIFICACIÓN

### Verificar que el servicio está corriendo:

1. Espera unos 2-3 minutos después del deploy
2. Busca la URL de tu servicio (algo como `https://zeroclaw-xxxxx.onrender.com`)
3. Abre esa URL en tu navegador

### Probar el health endpoint:
```bash
curl https://tu-servicio.onrender.com/health
```

Debería responder con algo como:
```json
{"status":"ok","version":"..."}
```

### Probar el gateway:
```bash
curl https://tu-servicio.onrender.com/
```

Debería mostrar una página web o interfaz.

---

## 7. SOLUCIÓN DE PROBLEMAS

### El servicio no inicia:

**Problema**: Error en logs
**Solución**: 
1. Revisa los logs en Render Dashboard
2. Verifica que `OPENAI_API_KEY` sea correcta
3. Asegúrate que el modelo exista

### "Invalid API Key":

**Problema**: Tu key de OpenRouter es incorrecta
**Solución**:
1. Ve a [openrouter.ai/keys](https://openrouter.ai/keys)
2. Crea una nueva key
3. Actualiza la variable en Render

### "Model Not Found":

**Problema**: El modelo especificado no existe
**Solución**: Cambia `ZEROCLAW_MODEL` a uno válido como:
- `openrouter/anthropic/claude-3-haiku`
- `openrouter/mistral/mistral-7b-instruct`

### Servicio dormido (sleeping):

**Problema**: Renderfree tier puts services to sleep after 15 min
**Solución**: 
- Espera ~30 segundos, se despierta automáticamente al recibir una request
- O actualiza a paid plan

### Puerto ya en uso:

**Problema**: Render asignó un puerto diferente
**Solución**: El comando ya usa `$PORT` que Render asigna automáticamente

---

## 8. COSTOS

| Servicio | Costo | Notas |
|----------|-------|-------|
| Render Free | $0 | 750 horas/mes, duerme después de 15 min inactividad |
| OpenRouter | ~$0-5/mes | $5 gratis mensuales |
| GitHub | $0 | Gratis |
| **TOTAL** | **~$0-5/mes** | |

---

## 9. MODELOS RECOMENDADOS

| Modelo | Calidad | Velocidad | Costo |
|--------|---------|-----------|-------|
| `openrouter/google/gemma-4-27b-it` | ⭐⭐⭐⭐⭐ | Media | Bajo |
| `openrouter/anthropic/claude-3-haiku` | ⭐⭐⭐⭐ | Rápida | Medio |
| `openrouter/mistral/mistral-7b-instruct` | ⭐⭐⭐ | Rápida | Muy bajo |

---

## 10. ACCESO DESDE TELEGRAM (OPCIONAL)

Si quieres usar Telegram:

1. Habla con @BotFather en Telegram
2. Crea un nuevo bot con /newbot
3. Copia el token (algo like `123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`)
4. Añádelo en las variables de entorno de Render:
   - `TELEGRAM_BOT_TOKEN` = tu token
5. Busca tu bot en Telegram y envíale /start

---

## RESUMEN RÁPIDO

1. ✅ Crear cuenta en OpenRouter → obtener API key
2. ✅ Clonar repo → editar .env con tu API key
3. ✅ Subir a GitHub
4. ✅ En Render: New → Web Service → conectar repo
5. ✅ Build: `docker build -t zeroclaw -f Dockerfile.render .`
6. ✅ Run: `docker run -p $PORT:$PORT -e OPENAI_API_KEY -e PROVIDER -e ZEROCLAW_MODEL zeroclaw`
7. ✅ Añadir `OPENAI_API_KEY=tu-key-real` en Environment Variables
8. ✅ Click Create → esperar ~10 min → verificar con /health

---

¿Tienes alguna duda o necesitas más ayuda?