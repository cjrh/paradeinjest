> [!WARNING]
> This is a research-level hobby project in alpha. It is not suitable for production use or any serious application.
>
> You probably don't want to use this.

# ParadeInJest

A Rust-based multi-tenant data analytics application demonstrating the **Medallion Architecture** (Bronze → Silver → Gold) using [ParadeDB](https://www.paradedb.com/) — PostgreSQL with BM25 full-text search and pgvector similarity search.

## Why This Exists

This project explores how to build a modern analytics pipeline that combines:

- **Multi-tenancy**: Each customer gets an isolated PostgreSQL schema with their own data
- **Medallion Architecture**: Progressive data refinement from raw ingestion to query-optimized storage
- **Hybrid Search**: Both traditional BM25 full-text search and semantic vector similarity search
- **ParadeDB**: Leveraging PostgreSQL extensions (`pg_search`, `pgvector`) instead of separate search infrastructure

## Architecture

This image links to the repo by [Piethein Strengholt on Building Medallion Architectures](https://github.com/pietheinstrengholt/building-medallion-architectures-book): 

![Medallion Architecture](https://raw.githubusercontent.com/pietheinstrengholt/building-medallion-architectures-book/main/assets/medallion_architecture.png)

```
CSV Upload → Bronze (raw JSONB) → Silver (enriched + embeddings) → Gold (indexed for search)
```

| Layer | Purpose | Data |
|-------|---------|------|
| **Bronze** | Raw ingestion | Original CSV rows as JSONB |
| **Silver** | Enrichment | Extracted fields, embeddings, sentiment analysis |
| **Gold** | Query-optimized | BM25 index, vector index, fast fields |



### Multi-Tenancy

Each customer gets an isolated schema (`customer_{id}`) with separate bronze/silver/gold tables. Schemas are auto-provisioned on first use.

### Search Capabilities

- **BM25 Full-Text**: ParadeDB's `pg_search` extension with scored results
- **Semantic Search**: pgvector IVFFlat index with cosine distance on 1536-dim embeddings

## Prerequisites

- [Rust](https://rustup.rs/) (for local development)
- [Podman](https://podman.io/) and podman-compose (for containerized runs)
- [just](https://github.com/casey/just) (command runner)
- `curl` and `jq` (for API testing)

## Quick Start

```bash
# Run the full demo (builds containers, starts services, loads sample data)
just rundemo

# Or step-by-step:
just build-containers   # Build Docker images
just up                 # Start ParadeDB and app
just wait-ready         # Wait for health check
just upload customer_a sample_data/customer_a/file1.csv
just process customer_a # Run full pipeline
just search customer_a "important"
just down               # Stop containers
```

## Development Workflow

### Building

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run tests
```

### Container Management

```bash
just build-containers    # Build Docker images
just up                  # Start containers (background)
just up-build            # Start with rebuild
just down                # Stop containers
just clean               # Stop and remove volumes
just logs                # Tail container logs
just health              # Check /health endpoint
```

### Data Pipeline

```bash
# Upload CSV files
just upload <customer> <file.csv>
just upload-customer-a   # Upload all sample files for customer_a
just upload-customer-b   # Upload all sample files for customer_b

# Process data through the medallion layers
just process <customer>               # Full pipeline (bronze → silver → gold)
just process-bronze-silver <customer> # Just bronze → silver
just process-silver-gold <customer>   # Just silver → gold
```

### Querying

```bash
just search <customer> "<query>"          # BM25 full-text search
just semantic-search <customer> "<query>" # Vector similarity search
just analytics <customer>                 # Get aggregations (sentiment breakdown, counts)
```

### Demos

```bash
just rundemo        # Full end-to-end demo with sample data
just demo-stepwise  # Step-by-step pipeline demonstration
just load-all       # Load and process all sample data
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/customers/{id}/upload` | Upload CSV (multipart) |
| `POST` | `/customers/{id}/process` | Run full pipeline |
| `POST` | `/customers/{id}/process/bronze-to-silver` | Bronze → Silver step |
| `POST` | `/customers/{id}/process/silver-to-gold` | Silver → Gold step |
| `GET` | `/customers/{id}/search?q=` | BM25 full-text search |
| `GET` | `/customers/{id}/search/semantic?q=` | Vector similarity search |
| `GET` | `/customers/{id}/analytics` | Aggregations |
| `GET` | `/health` | Health check |

## Configuration

Environment variables (set in `.env` or container):

```bash
DATABASE_URL=postgres://parade:parade@localhost:5432/paradeinjest
HOST=0.0.0.0
PORT=3000
RUST_LOG=paradeinjest=info,tower_http=debug
```

## Project Structure

```
src/
├── handlers/     # HTTP request handlers
├── services/     # Business logic (ingestion, embedding, sentiment, search)
├── models/       # Data structures (bronze, silver, gold)
├── db/           # Connection pool and schema management
├── routes.rs     # API routing
└── main.rs       # Entry point

sql/
└── init.sql      # Database initialization (extensions, functions)

sample_data/      # Example CSVs for testing
```

## Notes

- The embedding provider is currently mocked (returns deterministic 1536-dim vectors). Swap `MockEmbeddingProvider` for a real implementation (e.g., OpenAI) in production.
- Sentiment analysis uses a simple rule-based approach for demonstration.
