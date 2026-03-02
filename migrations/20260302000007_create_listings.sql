-- Listing history: tracks price drops and re-listings over time
CREATE TABLE IF NOT EXISTS listings (
    id SERIAL PRIMARY KEY,
    plate VARCHAR(10) NOT NULL REFERENCES vehicles(plate) ON DELETE CASCADE,
    source VARCHAR(100),
    seller_type VARCHAR(20),        -- 'private', 'dealer', 'auction'
    price_sek INT,
    mileage_km INT,
    listed_at DATE,
    delisted_at DATE,
    url TEXT,
    notes TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_listings_plate ON listings(plate);
CREATE INDEX IF NOT EXISTS idx_listings_listed ON listings(listed_at DESC);

-- Sample listing history for ASY634 (shows price drops and re-listings)
INSERT INTO listings (plate, source, seller_type, price_sek, mileage_km, listed_at, delisted_at, notes) VALUES
('ASY634', 'blocket.se', 'dealer', 59900, 110000, '2021-03-01', '2021-04-15', 'Fristads Bilcenter listing'),
('ASY634', 'blocket.se', 'dealer', 59000, 152310, '2025-09-01', '2025-12-15', 'Idealbilar i Uddevalla — did not sell'),
('ASY634', 'blocket.se', 'dealer', 45000, 165000, '2026-02-01', NULL, 'Lowered price, still not sold'),
('ASY634', 'bytbil.com', 'private', 50000, 156960, '2026-02-28', NULL, 'Friend relisting at higher price'),
('CBG212', 'blocket.se', 'private', 54900, 273700, '2025-11-01', NULL, 'Current listing'),
('FLY734', 'blocket.se', 'private', 75000, 179650, '2025-10-15', NULL, 'Overpriced for market');
