# AI Agent

A Rust AI agent with RAG capabilities using [rig](https://rig.rs/). API queues jobs to Redis, Worker processes with LLM + vector search.

## Architecture

Uses **ReAct (Reasoning + Acting) pattern**:

```
User Message → LLM Reasoning → Need context?
                                   ↓
                 Yes → knowledge_base tool → Search → LLM Response
                 No  → Direct Response
```

The LLM decides when to use tools, enabling flexible multi-step reasoning.

## Quick Start

```bash
# Setup
cp .env.example .env  # Add ANTHROPIC_API_KEY and OPENAI_API_KEY

# Run
docker compose up -d
cargo run --bin api     # Terminal 1
cargo run --bin worker  # Terminal 2
```

## API

```bash
# Chat
curl -X POST http://localhost:8080/api/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello"}'
# Returns: {"job_id": "...", "status": "queued"}

# Check result
curl http://localhost:8080/api/v1/chat/jobs/{job_id}

# Documents
curl -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" \
  -d '{"title": "Doc", "content": "..."}'

curl http://localhost:8080/api/v1/documents
curl -X POST http://localhost:8080/api/v1/documents/search \
  -d '{"query": "term", "limit": 5}'
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ANTHROPIC_API_KEY` | Claude API key | Required |
| `OPENAI_API_KEY` | Embeddings API key | Required |
| `REDIS_URL` | Redis connection | `redis://localhost:6379` |
| `QDRANT_URL` | Qdrant URL | `http://localhost:6334` |
| `SERVER_PORT` | API port | `8080` |

### YAML Config Files

Settings and prompts are in `config/`:

- `config/agent.yaml` - LLM, embedding, RAG, worker, CORS settings
- `config/prompts.yaml` - System prompts and tool descriptions

```yaml
# config/agent.yaml
llm:
  model: "claude-sonnet-4-20250514"
embedding:
  model: "text-embedding-3-small"
  dimension: 1536
rag:
  top_k: 5
  chunk_size: 1000
cors:
  allowed_origins:
    - "http://localhost:3000"
    - "https://yourdomain.com"
```

```yaml
# config/prompts.yaml
agent:
  system: |
    You are a helpful assistant...
```

## Development

```bash
cargo test
cargo fmt
cargo clippy
```

## License

MIT
