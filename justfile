# ParadeDB CSV Ingestion POC with Silver Layer

set dotenv-load

base_url := "http://localhost:3000"

# List available commands
default:
    @just --list

# Build the Rust application locally
build:
    cargo build

# Build the container images
build-containers:
    podman-compose build

# Start ParadeDB and app containers (background)
up:
    podman-compose up --no-build &

# Start with build (background)
up-build:
    podman-compose up --build &

# Stop containers
down:
    podman-compose down

# View container logs
logs:
    podman-compose logs -f

# Check health endpoint
health:
    @curl -s {{base_url}}/health && echo ""

# Wait for the app to be ready
wait-ready:
    @echo "Waiting for app to be ready..."
    @for i in $(seq 1 30); do \
        if curl -s {{base_url}}/health > /dev/null 2>&1; then \
            echo "App is ready!"; \
            exit 0; \
        fi; \
        sleep 1; \
    done; \
    echo "Timeout waiting for app"; exit 1

# Upload a CSV file for a customer
upload customer file:
    @echo "Uploading {{file}} for {{customer}}..."
    @curl -s -X POST {{base_url}}/customers/{{customer}}/upload \
        -F "file=@{{file}}" | jq .

# Process bronze -> silver for a customer
process-bronze-silver customer:
    @echo "Processing bronze -> silver for {{customer}}..."
    @curl -s -X POST {{base_url}}/customers/{{customer}}/process/bronze-to-silver | jq .

# Process silver -> gold for a customer
process-silver-gold customer:
    @echo "Processing silver -> gold for {{customer}}..."
    @curl -s -X POST {{base_url}}/customers/{{customer}}/process/silver-to-gold | jq .

# Full pipeline: bronze -> silver -> gold for a customer
process customer:
    @echo "Processing full pipeline for {{customer}}..."
    @curl -s -X POST {{base_url}}/customers/{{customer}}/process | jq .

# Search a customer's gold layer (BM25 full-text search)
search customer query:
    @echo "Searching {{customer}} for '{{query}}'..."
    @curl -s "{{base_url}}/customers/{{customer}}/search?q={{query}}" | jq .

# Semantic search a customer's gold layer (vector similarity)
semantic-search customer query:
    @echo "Semantic search {{customer}} for '{{query}}'..."
    @curl -s "{{base_url}}/customers/{{customer}}/search/semantic?q={{query}}" | jq .

# Get analytics for a customer
analytics customer:
    @echo "Getting analytics for {{customer}}..."
    @curl -s {{base_url}}/customers/{{customer}}/analytics | jq .

# Upload all sample files for customer_a
upload-customer-a:
    @just upload customer_a sample_data/customer_a/file1.csv
    @just upload customer_a sample_data/customer_a/file2.csv
    @just upload customer_a sample_data/customer_a/file3.csv

# Upload all sample files for customer_b
upload-customer-b:
    @just upload customer_b sample_data/customer_b/file1.csv
    @just upload customer_b sample_data/customer_b/file2.csv
    @just upload customer_b sample_data/customer_b/file3.csv

# Demo: Upload and process all sample data
load-all: upload-customer-a upload-customer-b
    @just process customer_a
    @just process customer_b

# Run the full demo with silver layer
rundemo: build-containers
    @echo "Starting containers in background..."
    podman-compose up &
    @sleep 2
    @just wait-ready
    @echo ""
    @echo "=========================================="
    @echo "  ParadeDB Medallion Architecture Demo"
    @echo "  Bronze -> Silver -> Gold Pipeline"
    @echo "=========================================="
    @echo ""

    @echo ">>> Step 1: Uploading CSVs for Customer A"
    @echo "-------------------------------------------"
    @just upload-customer-a
    @echo ""

    @echo ">>> Step 2: Uploading CSVs for Customer B"
    @echo "-------------------------------------------"
    @just upload-customer-b
    @echo ""

    @echo ">>> Step 3: Processing Customer A (bronze -> silver -> gold)"
    @echo "-------------------------------------------"
    @just process customer_a
    @echo ""

    @echo ">>> Step 4: Processing Customer B (bronze -> silver -> gold)"
    @echo "-------------------------------------------"
    @just process customer_b
    @echo ""

    @echo ">>> Step 5: BM25 Search for 'important' in Customer A"
    @echo "-------------------------------------------"
    @just search customer_a important
    @echo ""

    @echo ">>> Step 6: Semantic Search for 'important' in Customer A"
    @echo "-------------------------------------------"
    @just semantic-search customer_a important
    @echo ""

    @echo ">>> Step 7: Analytics for Customer A (with sentiment)"
    @echo "-------------------------------------------"
    @just analytics customer_a
    @echo ""

    @echo ">>> Step 8: Analytics for Customer B (with sentiment)"
    @echo "-------------------------------------------"
    @just analytics customer_b
    @echo ""

    @echo "=========================================="
    @echo "  Demo Complete!"
    @echo "=========================================="
    @echo ""
    @echo "Try these commands:"
    @echo "  just search customer_a meeting"
    @echo "  just semantic-search customer_a meeting"
    @echo "  just analytics customer_a"
    @echo "  just process-bronze-silver customer_a  # Step-by-step"
    @echo "  just process-silver-gold customer_a    # Step-by-step"
    @echo "  just down  # to stop containers"

# Demo: Step-by-step processing
demo-stepwise: build-containers
    @echo "Starting containers in background..."
    podman-compose up &
    @sleep 2
    @just wait-ready
    @echo ""
    @echo "=========================================="
    @echo "  Step-by-Step Processing Demo"
    @echo "=========================================="
    @echo ""

    @echo ">>> Step 1: Upload CSV"
    @just upload customer_a sample_data/customer_a/file1.csv
    @echo ""

    @echo ">>> Step 2: Process Bronze -> Silver (embeddings, sentiment)"
    @just process-bronze-silver customer_a
    @echo ""

    @echo ">>> Step 3: Process Silver -> Gold (optimized for queries)"
    @just process-silver-gold customer_a
    @echo ""

    @echo ">>> Step 4: Search and Analytics"
    @just search customer_a important
    @just analytics customer_a

# Clean up: stop containers and remove volumes
clean:
    podman-compose down -v
