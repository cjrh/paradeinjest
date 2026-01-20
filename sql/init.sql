-- Initialize ParadeDB extensions
CREATE EXTENSION IF NOT EXISTS pg_search;
CREATE EXTENSION IF NOT EXISTS vector;

-- Customer registry table in public schema
CREATE TABLE IF NOT EXISTS public.customers (
    id TEXT PRIMARY KEY,
    schema_name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Function to create customer schema with bronze, silver, and gold tables
CREATE OR REPLACE FUNCTION create_customer_schema(customer_id TEXT)
RETURNS VOID AS $$
DECLARE
    schema_name TEXT;
BEGIN
    schema_name := 'customer_' || customer_id;

    -- Create the schema
    EXECUTE format('CREATE SCHEMA IF NOT EXISTS %I', schema_name);

    -- Create bronze table (raw data)
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.bronze (
            id SERIAL PRIMARY KEY,
            source_file TEXT NOT NULL,
            raw_data JSONB NOT NULL,
            ingested_at TIMESTAMPTZ DEFAULT NOW()
        )', schema_name);

    -- Create silver table (enriched with embeddings, sentiment)
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.silver (
            id SERIAL PRIMARY KEY,
            bronze_id INTEGER REFERENCES %I.bronze(id) UNIQUE,

            -- Normalized data
            normalized_data JSONB NOT NULL,

            -- Extracted fields
            primary_text TEXT,
            label TEXT,
            source_date TIMESTAMPTZ,
            data_quality_score REAL DEFAULT 0.0,

            -- ML/AI derived fields
            embedding vector(1536),
            sentiment TEXT,
            sentiment_score REAL,

            -- Metadata
            field_mapping JSONB,
            processed_at TIMESTAMPTZ DEFAULT NOW()
        )', schema_name, schema_name);

    -- Create gold table (optimized for querying)
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.gold (
            id SERIAL PRIMARY KEY,
            silver_id INTEGER REFERENCES %I.silver(id) UNIQUE,
            bronze_id INTEGER,

            -- Searchable content
            text TEXT NOT NULL,
            label TEXT NOT NULL,

            -- Analytics fields (will be fast fields in BM25 index)
            sentiment TEXT,
            sentiment_score REAL,
            text_length INTEGER,
            word_count INTEGER,

            -- Vector for semantic search
            embedding vector(1536),

            processed_at TIMESTAMPTZ DEFAULT NOW()
        )', schema_name, schema_name);

    -- Register customer if not exists
    INSERT INTO public.customers (id, schema_name)
    VALUES (customer_id, schema_name)
    ON CONFLICT (id) DO NOTHING;
END;
$$ LANGUAGE plpgsql;
