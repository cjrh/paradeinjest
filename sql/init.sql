-- Initialize ParadeDB extensions
CREATE EXTENSION IF NOT EXISTS pg_search;

-- Customer registry table in public schema
CREATE TABLE IF NOT EXISTS public.customers (
    id TEXT PRIMARY KEY,
    schema_name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Function to create customer schema with bronze and gold tables
CREATE OR REPLACE FUNCTION create_customer_schema(customer_id TEXT)
RETURNS VOID AS $$
DECLARE
    schema_name TEXT;
BEGIN
    schema_name := 'customer_' || customer_id;

    -- Create the schema
    EXECUTE format('CREATE SCHEMA IF NOT EXISTS %I', schema_name);

    -- Create bronze table
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.bronze (
            id SERIAL PRIMARY KEY,
            source_file TEXT NOT NULL,
            raw_data JSONB NOT NULL,
            ingested_at TIMESTAMPTZ DEFAULT NOW()
        )', schema_name);

    -- Create gold table
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.gold (
            id SERIAL PRIMARY KEY,
            bronze_id INTEGER REFERENCES %I.bronze(id),
            text TEXT NOT NULL,
            label TEXT NOT NULL,
            processed_at TIMESTAMPTZ DEFAULT NOW()
        )', schema_name, schema_name);

    -- Register customer if not exists
    INSERT INTO public.customers (id, schema_name)
    VALUES (customer_id, schema_name)
    ON CONFLICT (id) DO NOTHING;
END;
$$ LANGUAGE plpgsql;
