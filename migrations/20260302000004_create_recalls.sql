CREATE TABLE IF NOT EXISTS recalls (
    id SERIAL PRIMARY KEY,
    vin VARCHAR(32) NOT NULL,
    source VARCHAR(50),
    description TEXT,
    severity VARCHAR(20),
    fix_available BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_recalls_vin ON recalls(vin);
