CREATE TABLE IF NOT EXISTS inspections (
    id SERIAL PRIMARY KEY,
    plate VARCHAR(10) NOT NULL REFERENCES vehicles(plate) ON DELETE CASCADE,
    date DATE,
    mileage_km INT,
    result VARCHAR(20),
    notes TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inspections_plate ON inspections(plate);
