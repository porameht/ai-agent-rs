# AI Agent

A Rust AI agent with RAG (Retrieval-Augmented Generation) capabilities using the [rig](https://rig.rs/) framework. The architecture separates API and Worker for scalable async processing.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      API Server (Axum)                          │
│              POST /chat  →  Queue Job  →  Return job_id         │
│              GET /jobs/:id  →  Check Status  →  Return result   │
└─────────────────────────────────┬───────────────────────────────┘
                                  │
                                  ▼
                        ┌─────────────────┐
                        │   Redis Queue   │
                        └────────┬────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────┐
│                         Worker                                   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                    ChatAgent (rig)                        │   │
│  │                                                           │   │
│  │   User Message                                            │   │
│  │        │                                                  │   │
│  │        ▼                                                  │   │
│  │   ┌─────────────────┐                                     │   │
│  │   │  LLM (Claude)   │  ←── decides to call tool           │   │
│  │   └────────┬────────┘                                     │   │
│  │            │                                              │   │
│  │            ▼                                              │   │
│  │   ┌─────────────────┐    ┌─────────────────┐              │   │
│  │   │ knowledge_base  │ →  │   RAG Service   │              │   │
│  │   │     (tool)      │    │  (embed+search) │              │   │
│  │   └─────────────────┘    └─────────────────┘              │   │
│  │            │                                              │   │
│  │            ▼                                              │   │
│  │   LLM generates final response with context               │   │
│  └───────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### How It Works

1. **API receives chat request** → Queues job to Redis → Returns `job_id`
2. **Worker picks up job** → Uses ChatAgent with rig tool calling
3. **LLM decides** if it needs context → Calls `knowledge_base` tool
4. **Tool queries RAG** → Returns relevant documents
5. **LLM generates response** with retrieved context
6. **Result stored in Redis** → Client polls for result

### Key Design Decisions

- **API is stateless** - Only queues jobs, no AI processing
- **Worker handles all AI** - LLM calls, RAG, tool execution
- **Tool calling pattern** - LLM decides when to search knowledge base (not hardcoded)
- **Async processing** - Client polls for results via job_id

## Project Structure

```
src/
├── lib.rs                  # Library entry point
├── main.rs                 # API server binary
├── worker.rs               # Worker binary
├── domain/                 # Domain layer (entities, ports)
│   ├── entities/           # Core domain types
│   ├── ports/              # Traits (interfaces)
│   └── errors.rs           # Domain errors
├── application/            # Application layer (services)
│   └── services/
│       ├── document.rs     # Document management
│       └── rag.rs          # RAG operations
├── infrastructure/         # Infrastructure layer
│   ├── agent.rs            # ChatAgent with rig tool calling
│   ├── embedding/          # Text embedding service
│   ├── vector_store/       # Vector storage (Qdrant)
│   ├── tools/              # Agent tools
│   │   └── knowledge_base.rs  # RAG tool for agent
│   └── queue/              # Redis job queue
└── api/                    # API layer (HTTP handlers)
    ├── routes/
    │   ├── chat.rs         # Chat endpoints
    │   ├── documents.rs    # Document endpoints
    │   └── health.rs       # Health checks
    └── state.rs            # Application state
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker (for Redis and Qdrant)
- Anthropic API key (Claude)
- OpenAI API key (for embeddings)

### Setup

```bash
# Clone and enter directory
git clone <repository-url>
cd agentic-rust

# Copy environment template
cp .env.example .env

# Edit .env with your API keys:
# ANTHROPIC_API_KEY=your-key
# OPENAI_API_KEY=your-key  (for embeddings)

# Build
cargo build
```

### Running

```bash
# Terminal 1: Start Redis and Qdrant
docker compose up -d

# Terminal 2: Start API server
cargo run --bin api

# Terminal 3: Start Worker
cargo run --bin worker
```

## API Endpoints

### Chat

```bash
# Send chat message (queued for worker)
curl -X POST http://localhost:8080/api/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "What is in the knowledge base?"}'

# Response: {"job_id": "uuid", "status": "queued"}

# Check job status
curl http://localhost:8080/api/v1/chat/jobs/{job_id}

# Response when complete:
# {"job_id": "uuid", "status": "completed", "result": {"response": "..."}}
```

### Documents

```bash
# Create document
curl -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" \
  -d '{"title": "My Doc", "content": "Document content..."}'

# List documents
curl http://localhost:8080/api/v1/documents

# Search documents
curl -X POST http://localhost:8080/api/v1/documents/search \
  -H "Content-Type: application/json" \
  -d '{"query": "search term", "limit": 5}'
```

### Health

```bash
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `ANTHROPIC_API_KEY` | Anthropic API key for Claude | Required |
| `OPENAI_API_KEY` | OpenAI API key for embeddings | Required |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `QDRANT_URL` | Qdrant gRPC URL | `http://localhost:6334` |
| `SERVER_HOST` | API server host | `0.0.0.0` |
| `SERVER_PORT` | API server port | `8080` |
| `WORKER_CONCURRENCY` | Worker threads | `4` |

## Tool Calling Pattern

The agent uses rig's tool calling pattern. The LLM decides when to call the knowledge base:

```rust
// ChatAgent creates a rig agent with tools
let agent = client
    .agent("claude-sonnet-4-20250514")
    .preamble("You are a helpful assistant...")
    .tool(KnowledgeBaseTool::new(rag, 5))  // RAG as a tool
    .build();

// LLM decides when to search knowledge base
let response = agent.prompt(message).await?;
```

The `knowledge_base` tool:
- **Name**: `knowledge_base`
- **Description**: Search the knowledge base for relevant information
- **Args**: `{"query": "search query"}`
- **Returns**: Top-k relevant document chunks

## Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Check without building
cargo check
```

## License

MIT
