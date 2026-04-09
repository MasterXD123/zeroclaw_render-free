# ZeroClaw Enterprise Docker Build

## Build

```bash
cd zeroclaw
docker build -t zeroclaw:enterprise .
```

## Run

```bash
# Con docker-compose (recomendado)
docker compose up -d

# Solo zeroclaw (requiere Qdrant corriendo)
docker run -d \
  --name zeroclaw-enterprise \
  -p 42617:42617 \
  -e OPENAI_API_KEY=sk-... \
  -e QDRANT_HOST=localhost \
  -e QDRANT_PORT=6333 \
  zeroclaw:enterprise
```

## Verificar

```bash
# Ver contenedores
docker compose ps

# Logs
docker compose logs -f zeroclaw

# Health check
curl http://localhost:42617/health
```

## Variables de Entorno

| Variable | Descripción | Default |
|----------|-------------|---------|
| OPENAI_API_KEY | OpenAI API key | - |
| NOTION_KEY | Notion integration secret | - |
| GITHUB_TOKEN | GitHub personal token | - |
| QDRANT_HOST | Qdrant host | qdrant |
| QDRANT_PORT | Qdrant port | 6333 |
| RUST_LOG | Log level | info |
| ZEROCLAW_PORT | ZeroClaw port | 42617 |