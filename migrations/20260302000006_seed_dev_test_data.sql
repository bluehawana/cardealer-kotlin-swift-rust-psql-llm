-- Sample vehicles based on real-world Honda Civic examples

-- Vehicle 1: CBG212 — Best candidate (1 owner, diesel, clean history)
INSERT INTO vehicles (plate, vin, make, model, variant, year, body_type, fuel_type,
    engine_code, engine_displacement_l, engine_power_hp, engine_torque_nm,
    gearbox_type, gearbox_gears, drivetrain_type, is_4x4,
    curb_weight_kg, seats, color, imported, first_registration, country_origin,
    owner_count, mileage_latest, tax_per_year, co2_emission, euro_standard)
VALUES ('CBG212', 'SHHFK2760DU100001', 'Honda', 'Civic', '1.6 i-DTEC', 2013, 'Hatchback', 'Diesel',
    'N16A1', 1.6, 120, 300, 'Manual', 6, 'FWD', false,
    1280, 5, 'Silver', false, '2013-06-15', 'Sweden',
    1, 27370, 1238, 94, 'Euro 5');

-- Vehicle 2: FLY734 — Risky (7 owners, petrol)
INSERT INTO vehicles (plate, vin, make, model, variant, year, body_type, fuel_type,
    engine_code, engine_displacement_l, engine_power_hp, engine_torque_nm,
    gearbox_type, gearbox_gears, drivetrain_type, is_4x4,
    curb_weight_kg, seats, color, imported, first_registration, country_origin,
    owner_count, mileage_latest, tax_per_year, co2_emission, euro_standard)
VALUES ('FLY734', 'SHHFK2780AU200002', 'Honda', 'Civic', '1.8 i-VTEC', 2010, 'Sedan', 'Petrol',
    'R18A2', 1.8, 140, 174, 'Automatic', 5, 'FWD', false,
    1285, 5, 'Black', false, '2010-03-20', 'Sweden',
    7, 17965, 1876, 155, 'Euro 5');

-- Vehicle 3: ASY634 — Problematic (7 owners, multiple failed inspections, relisted)
INSERT INTO vehicles (plate, vin, make, model, variant, year, body_type, fuel_type,
    engine_code, engine_displacement_l, engine_power_hp, engine_torque_nm,
    gearbox_type, gearbox_gears, drivetrain_type, is_4x4,
    curb_weight_kg, seats, color, imported, first_registration, country_origin,
    owner_count, mileage_latest, tax_per_year, co2_emission, euro_standard)
VALUES ('ASY634', 'SHHFK2780AU300003', 'Honda', 'Civic', '1.8 i-VTEC', 2010, 'Sedan', 'Petrol',
    'R18A2', 1.8, 140, 174, 'Manual', 6, 'FWD', false,
    1270, 5, 'Grey', false, '2010-08-10', 'Sweden',
    7, 16500, 1876, 155, 'Euro 5');

-- Ownership history
INSERT INTO ownership_history (plate, date, event) VALUES
('CBG212', '2013-06-15', 'First registration — private owner'),
('FLY734', '2010-03-20', 'First registration'),
('FLY734', '2012-05-10', 'Transferred to 2nd owner'),
('FLY734', '2014-09-15', 'Transferred to 3rd owner'),
('FLY734', '2016-03-22', 'Transferred to 4th owner'),
('FLY734', '2018-01-10', 'Transferred to 5th owner'),
('FLY734', '2020-06-30', 'Transferred to 6th owner'),
('FLY734', '2022-11-15', 'Transferred to 7th owner'),
('ASY634', '2010-08-10', 'First registration'),
('ASY634', '2013-02-20', 'Transferred to 2nd owner'),
('ASY634', '2015-07-11', 'Transferred to 3rd owner'),
('ASY634', '2017-09-05', 'Transferred to 4th owner'),
('ASY634', '2019-11-20', 'Transferred to 5th owner'),
('ASY634', '2021-04-15', 'Sold to private owner (Uddevalla)'),
('ASY634', '2025-04-01', 'Acquired by Idealbilar i Uddevalla (dealer)'),
('ASY634', '2025-07-10', 'Sold to current owner (Uddevalla)');

-- Inspections — CBG212 (mostly clean, 2 remark reinspections)
INSERT INTO inspections (plate, date, mileage_km, result, notes) VALUES
('CBG212', '2015-06-20', 45000, 'PASSED', NULL),
('CBG212', '2017-06-15', 92000, 'PASSED', NULL),
('CBG212', '2019-06-10', 145000, 'PASSED', NULL),
('CBG212', '2020-06-05', 175000, 'REINSPECTION', 'Minor brake pad wear'),
('CBG212', '2020-06-25', 175200, 'PASSED', 'Brake pads replaced'),
('CBG212', '2022-06-08', 220000, 'REINSPECTION', 'Exhaust leak minor'),
('CBG212', '2022-07-01', 220300, 'PASSED', 'Exhaust repaired'),
('CBG212', '2024-06-12', 265000, 'PASSED', NULL);

-- Inspections — FLY734 (all passed, surprisingly clean for 7 owners)
INSERT INTO inspections (plate, date, mileage_km, result, notes) VALUES
('FLY734', '2012-03-15', 25000, 'PASSED', NULL),
('FLY734', '2014-03-20', 58000, 'PASSED', NULL),
('FLY734', '2016-03-18', 95000, 'PASSED', NULL),
('FLY734', '2018-03-22', 125000, 'PASSED', NULL),
('FLY734', '2020-03-10', 148000, 'PASSED', NULL),
('FLY734', '2022-03-15', 162000, 'PASSED', NULL),
('FLY734', '2024-03-20', 175000, 'PASSED', NULL);

-- Inspections — ASY634 (3 failed reinspections — red flag)
INSERT INTO inspections (plate, date, mileage_km, result, notes) VALUES
('ASY634', '2012-08-10', 22000, 'PASSED', NULL),
('ASY634', '2014-08-15', 55000, 'PASSED', NULL),
('ASY634', '2016-08-20', 82000, 'PASSED', NULL),
('ASY634', '2018-08-12', 105000, 'PASSED', NULL),
('ASY634', '2020-08-05', 118000, 'REINSPECTION', 'Suspension play, brake disc worn'),
('ASY634', '2020-09-01', 118200, 'PASSED', 'Suspension and brakes repaired'),
('ASY634', '2021-05-10', 125000, 'REINSPECTION', 'Exhaust emission failure'),
('ASY634', '2021-06-05', 125300, 'PASSED', 'Catalytic converter replaced'),
('ASY634', '2024-06-15', 155000, 'REINSPECTION', 'Steering rack play, oil leak'),
('ASY634', '2024-07-10', 155500, 'PASSED', 'Steering rack and gasket replaced'),
('ASY634', '2025-09-20', 165000, 'PASSED', NULL);

-- Recalls
INSERT INTO recalls (vin, source, description, severity, fix_available) VALUES
('SHHFK2780AU300003', 'Honda Sweden', 'Takata airbag inflator - passenger side', 'High', true),
('SHHFK2780AU200002', 'Honda Sweden', 'Takata airbag inflator - passenger side', 'High', true),
('SHHFK2780AU200002', 'Honda Sweden', 'Fuel pump impeller deformation', 'Medium', true);
