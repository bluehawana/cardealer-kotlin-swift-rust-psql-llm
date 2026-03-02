CREATE TABLE IF NOT EXISTS ownership_history (
    id SERIAL PRIMARY KEY,
    plate VARCHAR(10) NOT NULL REFERENCES vehicles(plate) ON DELETE CASCADE,
    date DATE,
    event VARCHAR(200),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ownership_plate ON ownership_history(plate);
