# Skill Creator - Creador de Skills

## Descripción
Esta skill permite crear nuevas skills para ZeroClaw directamente desde conversación. El usuario proporciona el código/config de una skill y yo la adapto y creo los archivos.

## Activación
Se activa cuando el usuario dice:
- "crea una skill"
- "nueva skill"
- "quiero crear una skill"
- "make skill"
- "crear herramienta"
- "nueva herramienta"

## Comportamiento

### Paso 1: Pedir información
El usuario describe la skill que quiere crear. Puede ser:
- Una función/herramienta (con código o descripción)
- Una personalidad/comportamiento
- Un workflow de pasos

### Paso 2: Adaptar al formato ZeroClaw

Si es una **herramienta/API**:
- Convertir a comando shell con curl/python/etc
- Definir parámetros como variables ($PARAM)
- Crear SKILL.toml con tool definido

Si es una **personalidad**:
- Crear SKILL.md con instrucciones claras
- Definir keywords de activación
- Describir comportamiento

### Paso 3: Crear archivos

Ubicación: `/zeroclaw-data/workspace/skills/[nombre]/

Archivos a crear:
- `SKILL.toml` - Metadatos + herramientas
- `SKILL.md` - Descripción + comportamiento

### Paso 4: Confirmar

Mostrar al usuario:
- Nombre de la skill creada
- Keywords para activarla
- Cómo usarla

## Herramientas que uso

Para crear archivos uso las herramientas disponibles:
- `file_write` - Para crear SKILL.toml y SKILL.md

## Ejemplo de conversación

**Usuario**: "crea una skill que busque en Google"

**Yo**:
1. "Dale, necesitás darme más detalles. La skill es para buscar en Google y devolver resultados?"

2. (El usuario confirma) "Ok, voy a crear una skill 'google_search' que use curl a una API de búsqueda."

3. Creo los archivos:
   - SKILL.toml con tool google_search
   - SKILL.md con activación y comportamiento

4. "✅ Skill creada: google_search
   - Keywords: 'busca en google', 'search google'
   - Archivos: /zeroclaw-data/workspace/skills/google_search/
   - Para activarla, decí 'busca en google [consulta]'"

## Errores comunes

- "No puedo crear la skill porque..." - Explicar el error
- "Necesito más detalles sobre..." - Pedir información faltante
- "Esa skill ya existe" - Ofrecer sobrescribir o elegir otro nombre

## Notas

- Solo creo skills, no las instalo automáticamente
- El usuario puede instalar con: `zeroclaw skills install ./mi-skill/`
- Validar que el nombre no tenga caracteres especiales