CREATE TABLE IF NOT EXISTS reports (
    id SERIAL PRIMARY KEY,
    plate VARCHAR(10) NOT NULL REFERENCES vehicles(plate) ON DELETE CASCADE,
    summary_json JSONB,
    risk_score INT,
    price_min INT,
    price_max INT,
    locale VARCHAR(10) DEFAULT 'en',
    generated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reports_plate ON reports(plate);
CREATE INDEX IF NOT EXISTS idx_reports_generated ON reports(generated_at DESC);
