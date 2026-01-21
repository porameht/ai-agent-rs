# AI Agent

A Rust AI agent with RAG capabilities using [rig](https://rig.rs/). API queues jobs to Redis, Worker processes with LLM + vector search.

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

| Variable | Description | Default |
|----------|-------------|---------|
| `ANTHROPIC_API_KEY` | Claude API key | Required |
| `OPENAI_API_KEY` | Embeddings API key | Required |
| `REDIS_URL` | Redis connection | `redis://localhost:6379` |
| `QDRANT_URL` | Qdrant URL | `http://localhost:6334` |
| `SERVER_PORT` | API port | `8080` |

## Development

```bash
cargo test
cargo fmt
cargo clippy
```

## License

MIT
