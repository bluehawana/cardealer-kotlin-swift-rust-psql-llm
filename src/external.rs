use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::config::Config;

/// External vehicle data provider — fetches real data from Swedish sources.
#[derive(Clone)]
pub struct VehicleDataProvider {
    http: reqwest::Client,
    biluppgifter_api_key: String,
    biluppgifter_base_url: String,
}

// ─── Biluppgifter.se API response types ─────────────────────────

#[derive(Debug, Deserialize)]
pub struct BiluppgifterResponse {
    pub data: Option<BiluppgifterVehicle>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BiluppgifterVehicle {
    #[serde(rename = "reg_no")]
    pub reg_no: Option<String>,
    pub vin: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub model_name: Option<String>,
    pub model_year: Option<i32>,
    pub color: Option<String>,
    pub vehicle_type: Option<String>,
    pub body_type: Option<String>,
    pub fuel: Option<String>,
    pub fuel_type: Option<String>,
    pub engine_power: Option<i32>,        // kW
    pub engine_displacement: Option<i32>, // cc
    pub gearbox: Option<String>,
    pub number_of_gears: Option<i32>,
    pub drivetrain: Option<String>,
    pub four_wheel_drive: Option<bool>,
    pub seats: Option<i32>,
    pub curb_weight: Option<i32>,
    pub gross_weight: Option<i32>,
    pub length: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub co2: Option<i32>,
    pub euro_class: Option<String>,
    pub imported: Option<bool>,
    pub first_registration_date: Option<String>,
    pub country_of_origin: Option<String>,
    pub number_of_owners: Option<i32>,
    pub annual_tax: Option<i32>,
    pub status: Option<String>,
    pub inspection: Option<BiluppgifterInspection>,
    pub inspections: Option<Vec<BiluppgifterInspectionItem>>,
    pub recalls: Option<Vec<BiluppgifterRecall>>,
    pub theft: Option<BiluppgifterTheft>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BiluppgifterInspection {
    pub latest_inspection: Option<String>,
    pub result: Option<String>,
    pub valid_until: Option<String>,
    pub odometer: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BiluppgifterInspectionItem {
    pub date: Option<String>,
    pub result: Option<String>,
    pub odometer: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BiluppgifterRecall {
    pub description: Option<String>,
    pub severity: Option<String>,
    pub fix_available: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BiluppgifterTheft {
    pub stolen: Option<bool>,
    pub date: Option<String>,
}

impl VehicleDataProvider {
    pub fn new(_cfg: &Config) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            biluppgifter_api_key: std::env::var("BILUPPGIFTER_API_KEY").unwrap_or_default(),
            biluppgifter_base_url: std::env::var("BILUPPGIFTER_BASE_URL")
                .unwrap_or_else(|_| "https://api.biluppgifter.se/api/v1".into()),
        }
    }

    /// Fetch vehicle data from external APIs and upsert into the database.
    /// Returns the vehicle row (either newly fetched or existing).
    pub async fn fetch_and_store(&self, pool: &PgPool, plate: &str) -> anyhow::Result<FetchResult> {
        let plate = plate.to_uppercase();

        // Try Biluppgifter.se API first (if key is configured)
        if !self.biluppgifter_api_key.is_empty() {
            tracing::info!(plate = %plate, "fetching from Biluppgifter.se API");
            match self.fetch_from_biluppgifter(&plate).await {
                Ok(data) => {
                    let result = self.store_biluppgifter_data(pool, &plate, &data).await?;
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!(plate = %plate, error = %e, "Biluppgifter API failed, trying fallback");
                }
            }
        }

        // Fallback: Transportstyrelsen web scraper
        tracing::info!(plate = %plate, "fetching from Transportstyrelsen (web scraper)");
        match self.fetch_from_transportstyrelsen(&plate).await {
            Ok(data) => {
                let result = self
                    .store_transportstyrelsen_data(pool, &plate, &data)
                    .await?;
                Ok(result)
            }
            Err(e) => {
                tracing::error!(plate = %plate, error = %e, "all external sources failed");
                anyhow::bail!(
                    "Could not fetch vehicle data for plate {}. Configure BILUPPGIFTER_API_KEY for reliable access, or check the plate number.",
                    plate
                )
            }
        }
    }

    /// Fetch from Biluppgifter.se API.
    async fn fetch_from_biluppgifter(&self, plate: &str) -> anyhow::Result<BiluppgifterVehicle> {
        let url = format!("{}/vehicle/regno/{}", self.biluppgifter_base_url, plate);
        let resp = self
            .http
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.biluppgifter_api_key),
            )
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Biluppgifter API error {}: {}", status, body);
        }

        let response: BiluppgifterResponse = resp.json().await?;
        response
            .data
            .ok_or_else(|| anyhow::anyhow!("No vehicle data returned for plate {}", plate))
    }

    /// Store Biluppgifter data into our database.
    async fn store_biluppgifter_data(
        &self,
        pool: &PgPool,
        plate: &str,
        data: &BiluppgifterVehicle,
    ) -> anyhow::Result<FetchResult> {
        // Convert kW to HP (1 kW ≈ 1.36 HP)
        let power_hp = data.engine_power.map(|kw| (kw as f64 * 1.36) as i32);
        // Convert cc to L
        let displacement_l = data.engine_displacement.map(|cc| cc as f64 / 1000.0);
        // Parse height from mm to m
        let height_m = data.height.map(|h| h as f64 / 1000.0);

        let raw_json = serde_json::to_value(data).ok();

        // Upsert vehicle
        sqlx::query(
            r#"INSERT INTO vehicles (
                plate, vin, make, model, variant, year, body_type, fuel_type,
                engine_displacement_l, engine_power_hp, gearbox_type, gearbox_gears,
                drivetrain_type, is_4x4, curb_weight_kg, gross_weight_kg,
                length_mm, width_mm, height_m, seats, color, imported,
                first_registration, country_origin, owner_count, mileage_latest,
                tax_per_year, co2_emission, euro_standard, raw_source_json,
                updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18, $19, $20, $21, $22,
                $23::DATE, $24, $25, $26, $27, $28, $29, $30, NOW()
            )
            ON CONFLICT (plate) DO UPDATE SET
                vin = COALESCE(EXCLUDED.vin, vehicles.vin),
                make = COALESCE(EXCLUDED.make, vehicles.make),
                model = COALESCE(EXCLUDED.model, vehicles.model),
                variant = COALESCE(EXCLUDED.variant, vehicles.variant),
                year = COALESCE(EXCLUDED.year, vehicles.year),
                body_type = COALESCE(EXCLUDED.body_type, vehicles.body_type),
                fuel_type = COALESCE(EXCLUDED.fuel_type, vehicles.fuel_type),
                engine_displacement_l = COALESCE(EXCLUDED.engine_displacement_l, vehicles.engine_displacement_l),
                engine_power_hp = COALESCE(EXCLUDED.engine_power_hp, vehicles.engine_power_hp),
                gearbox_type = COALESCE(EXCLUDED.gearbox_type, vehicles.gearbox_type),
                owner_count = COALESCE(EXCLUDED.owner_count, vehicles.owner_count),
                mileage_latest = COALESCE(EXCLUDED.mileage_latest, vehicles.mileage_latest),
                tax_per_year = COALESCE(EXCLUDED.tax_per_year, vehicles.tax_per_year),
                co2_emission = COALESCE(EXCLUDED.co2_emission, vehicles.co2_emission),
                euro_standard = COALESCE(EXCLUDED.euro_standard, vehicles.euro_standard),
                raw_source_json = COALESCE(EXCLUDED.raw_source_json, vehicles.raw_source_json),
                updated_at = NOW()
            "#,
        )
        .bind(plate)
        .bind(&data.vin)
        .bind(&data.make)
        .bind(data.model_name.as_ref().or(data.model.as_ref()))
        .bind::<Option<&str>>(None) // variant
        .bind(data.model_year)
        .bind(&data.body_type)
        .bind(data.fuel_type.as_ref().or(data.fuel.as_ref()))
        .bind(displacement_l)
        .bind(power_hp)
        .bind(&data.gearbox)
        .bind(data.number_of_gears)
        .bind(&data.drivetrain)
        .bind(data.four_wheel_drive.unwrap_or(false))
        .bind(data.curb_weight)
        .bind(data.gross_weight)
        .bind(data.length)
        .bind(data.width)
        .bind(height_m)
        .bind(data.seats)
        .bind(&data.color)
        .bind(data.imported.unwrap_or(false))
        .bind(&data.first_registration_date)
        .bind(&data.country_of_origin)
        .bind(data.number_of_owners)
        .bind(data.inspection.as_ref().and_then(|i| i.odometer))
        .bind(data.annual_tax)
        .bind(data.co2)
        .bind(&data.euro_class)
        .bind(&raw_json)
        .execute(pool)
        .await?;

        // Store inspection history
        if let Some(inspections) = &data.inspections {
            for insp in inspections {
                sqlx::query(
                    "INSERT INTO inspections (plate, date, mileage_km, result, notes) VALUES ($1, $2::DATE, $3, $4, $5) ON CONFLICT DO NOTHING"
                )
                .bind(plate)
                .bind(&insp.date)
                .bind(insp.odometer)
                .bind(&insp.result)
                .bind(&insp.notes)
                .execute(pool)
                .await
                .ok();
            }
        }

        // Store recalls
        if let Some(recalls) = &data.recalls {
            if let Some(vin) = &data.vin {
                for recall in recalls {
                    sqlx::query(
                        "INSERT INTO recalls (vin, source, description, severity, fix_available) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING"
                    )
                    .bind(vin)
                    .bind("biluppgifter.se")
                    .bind(&recall.description)
                    .bind(&recall.severity)
                    .bind(recall.fix_available.unwrap_or(false))
                    .execute(pool)
                    .await
                    .ok();
                }
            }
        }

        let stolen = data.theft.as_ref().and_then(|t| t.stolen).unwrap_or(false);

        Ok(FetchResult {
            source: "biluppgifter.se".into(),
            plate: plate.to_string(),
            found: true,
            stolen,
        })
    }

    /// Fetch basic data from Transportstyrelsen public web page.
    /// Uses the public "Fordonsuppgifter" service — free, no API key needed.
    async fn fetch_from_transportstyrelsen(
        &self,
        plate: &str,
    ) -> anyhow::Result<TransportstyrelsenData> {
        // Transportstyrelsen has a public form at this URL
        let url = "https://fordonsuppgifter.transportstyrelsen.se/api/vehicle";

        // Try the API endpoint first
        let resp = self
            .http
            .get(format!("{}?registrationNumber={}", url, plate))
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            )
            .header("Accept", "text/html,application/xhtml+xml,application/json")
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let body = r.text().await?;
                // Try JSON parse first (if they have a JSON endpoint)
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                    return Ok(parse_json_response(&data, plate));
                }
                // Fall back to HTML parsing
                return parse_transportstyrelsen_html(&body, plate);
            }
            _ => {}
        }

        // Fallback: try the classic form submission
        let form_url = "https://fu-regnr.transportstyrelsen.se/externalresources/LicensePlate";
        let resp = self
            .http
            .post(form_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            )
            .form(&[("regnr", plate)])
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "Transportstyrelsen returned status {}. The public lookup may be temporarily unavailable. \
                Configure BILUPPGIFTER_API_KEY for reliable access.",
                resp.status()
            );
        }

        let html = resp.text().await?;

        if html.len() < 200 || html.contains("Inga uppgifter") || html.contains("hittades inte") {
            anyhow::bail!("No vehicle found for plate {} on Transportstyrelsen", plate);
        }

        parse_transportstyrelsen_html(&html, plate)
    }

    /// Store Transportstyrelsen data into database.
    async fn store_transportstyrelsen_data(
        &self,
        pool: &PgPool,
        plate: &str,
        data: &TransportstyrelsenData,
    ) -> anyhow::Result<FetchResult> {
        sqlx::query(
            r#"INSERT INTO vehicles (plate, make, model, year, fuel_type, color,
                curb_weight_kg, gross_weight_kg, owner_count, co2_emission, euro_standard,
                tax_per_year, engine_power_hp, engine_displacement_l,
                first_registration, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15::DATE, NOW())
            ON CONFLICT (plate) DO UPDATE SET
                make = COALESCE(EXCLUDED.make, vehicles.make),
                model = COALESCE(EXCLUDED.model, vehicles.model),
                year = COALESCE(EXCLUDED.year, vehicles.year),
                fuel_type = COALESCE(EXCLUDED.fuel_type, vehicles.fuel_type),
                color = COALESCE(EXCLUDED.color, vehicles.color),
                curb_weight_kg = COALESCE(EXCLUDED.curb_weight_kg, vehicles.curb_weight_kg),
                gross_weight_kg = COALESCE(EXCLUDED.gross_weight_kg, vehicles.gross_weight_kg),
                owner_count = COALESCE(EXCLUDED.owner_count, vehicles.owner_count),
                co2_emission = COALESCE(EXCLUDED.co2_emission, vehicles.co2_emission),
                euro_standard = COALESCE(EXCLUDED.euro_standard, vehicles.euro_standard),
                tax_per_year = COALESCE(EXCLUDED.tax_per_year, vehicles.tax_per_year),
                engine_power_hp = COALESCE(EXCLUDED.engine_power_hp, vehicles.engine_power_hp),
                engine_displacement_l = COALESCE(EXCLUDED.engine_displacement_l, vehicles.engine_displacement_l),
                first_registration = COALESCE(EXCLUDED.first_registration, vehicles.first_registration),
                updated_at = NOW()
            "#,
        )
        .bind(plate)
        .bind(&data.make)
        .bind(&data.model)
        .bind(data.year)
        .bind(&data.fuel_type)
        .bind(&data.color)
        .bind(data.curb_weight_kg)
        .bind(data.gross_weight_kg)
        .bind(data.owner_count)
        .bind(data.co2_emission)
        .bind(&data.euro_standard)
        .bind(data.tax_per_year)
        .bind(data.engine_power_hp)
        .bind(data.engine_displacement_l)
        .bind(&data.first_registration)
        .execute(pool)
        .await?;

        // Store inspection if available
        if let Some(ref result) = data.inspection_result {
            if let Some(ref date) = data.inspection_date {
                sqlx::query(
                    "INSERT INTO inspections (plate, date, mileage_km, result, notes) VALUES ($1, $2::DATE, $3, $4, $5) ON CONFLICT DO NOTHING"
                )
                .bind(plate)
                .bind(date)
                .bind(data.inspection_mileage)
                .bind(result)
                .bind(&data.inspection_notes)
                .execute(pool)
                .await
                .ok();
            }
        }

        Ok(FetchResult {
            source: "transportstyrelsen.se".into(),
            plate: plate.to_string(),
            found: true,
            stolen: false,
        })
    }
}

// ─── Result types ────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FetchResult {
    pub source: String,
    pub plate: String,
    pub found: bool,
    pub stolen: bool,
}

#[derive(Debug)]
pub struct TransportstyrelsenData {
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub fuel_type: Option<String>,
    pub color: Option<String>,
    pub curb_weight_kg: Option<i32>,
    pub gross_weight_kg: Option<i32>,
    pub owner_count: Option<i32>,
    pub co2_emission: Option<i32>,
    pub euro_standard: Option<String>,
    pub tax_per_year: Option<i32>,
    pub engine_power_hp: Option<i32>,
    pub engine_displacement_l: Option<f64>,
    pub first_registration: Option<String>,
    pub inspection_result: Option<String>,
    pub inspection_date: Option<String>,
    pub inspection_mileage: Option<i32>,
    pub inspection_notes: Option<String>,
}

/// Parse JSON response from Transportstyrelsen API.
fn parse_json_response(data: &serde_json::Value, _plate: &str) -> TransportstyrelsenData {
    let s = |key: &str| {
        data.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    let i = |key: &str| data.get(key).and_then(|v| v.as_i64()).map(|n| n as i32);
    let f = |key: &str| data.get(key).and_then(|v| v.as_f64());

    TransportstyrelsenData {
        make: s("make").or_else(|| s("fabrikat")),
        model: s("model").or_else(|| s("handelsbenamning")),
        year: i("modelYear").or(i("arsmodell")),
        fuel_type: s("fuelType").or_else(|| s("drivmedel")),
        color: s("color").or_else(|| s("farg")),
        curb_weight_kg: i("curbWeight").or(i("tjanstevikt")),
        gross_weight_kg: i("grossWeight").or(i("totalvikt")),
        owner_count: i("numberOfOwners"),
        co2_emission: i("co2"),
        euro_standard: s("euroClass").or_else(|| s("miljoklass")),
        tax_per_year: i("annualTax").or(i("fordonsskatt")),
        engine_power_hp: i("enginePower"),
        engine_displacement_l: f("engineDisplacement"),
        first_registration: s("firstRegistrationDate"),
        inspection_result: s("inspectionResult"),
        inspection_date: s("lastInspectionDate"),
        inspection_mileage: i("inspectionOdometer"),
        inspection_notes: s("inspectionNotes"),
    }
}

/// Parse Transportstyrelsen HTML response to extract vehicle data.
fn parse_transportstyrelsen_html(
    html: &str,
    _plate: &str,
) -> anyhow::Result<TransportstyrelsenData> {
    let extract = |labels: &[&str]| -> Option<String> {
        for label in labels {
            if let Some(pos) = html.find(label) {
                let after = &html[pos + label.len()..];
                // Try different value patterns: <strong>, <td>, <span>, <dd>, plain text after >
                for tag_end in ['>', ':'] {
                    if let Some(start) = after.find(tag_end) {
                        let rest = &after[start + 1..];
                        // Skip whitespace and nested tags
                        let rest = rest.trim_start();
                        if rest.starts_with('<') {
                            if let Some(inner_start) = rest.find('>') {
                                let inner = &rest[inner_start + 1..];
                                if let Some(end) = inner.find('<') {
                                    let val = inner[..end].trim();
                                    if !val.is_empty() && val != "-" {
                                        return Some(val.to_string());
                                    }
                                }
                            }
                        } else if let Some(end) = rest.find('<') {
                            let val = rest[..end].trim();
                            if !val.is_empty() && val != "-" {
                                return Some(val.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    };

    let parse_int = |labels: &[&str]| -> Option<i32> {
        extract(labels).and_then(|s| {
            let cleaned: String = s
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '-')
                .collect();
            cleaned.parse::<i32>().ok()
        })
    };

    let make = extract(&["Fabrikat", "Make", "Tillverkare"]);
    let model = extract(&["Handelsbenämning", "Handelsbeteckning", "Model", "Modell"]);

    if make.is_none() && model.is_none() {
        anyhow::bail!("Could not parse vehicle data from Transportstyrelsen response");
    }

    // Convert kW to HP if power is in kW
    let power_kw = parse_int(&["Motoreffekt", "Engine power", "Effekt"]);
    let power_hp = power_kw.map(|kw| (kw as f64 * 1.36) as i32);

    // Engine displacement
    let displacement_cc = parse_int(&["Slagvolym", "Displacement", "Cylindervolym"]);
    let displacement_l = displacement_cc.map(|cc| cc as f64 / 1000.0);

    Ok(TransportstyrelsenData {
        make,
        model,
        year: parse_int(&["Årsmodell", "Model year", "Arsmodell"]),
        fuel_type: extract(&["Drivmedel", "Fuel", "Bränsle"]),
        color: extract(&["Färg", "Color", "Kulör"]),
        curb_weight_kg: parse_int(&["Tjänstevikt", "Curb weight", "Tjanstevikt"]),
        gross_weight_kg: parse_int(&["Totalvikt", "Gross weight"]),
        owner_count: parse_int(&["Antal ägare", "Number of owners", "Antal agare"]),
        co2_emission: parse_int(&["CO2", "Koldioxid"]),
        euro_standard: extract(&["Miljöklass", "Euro class", "Miljoklass"]),
        tax_per_year: parse_int(&["Fordonsskatt", "Vehicle tax", "Skatt"]),
        engine_power_hp: power_hp,
        engine_displacement_l: displacement_l,
        first_registration: extract(&[
            "Första registrering",
            "First registration",
            "Forsta registrering",
        ]),
        inspection_result: extract(&[
            "Besiktningsresultat",
            "Inspection result",
            "Senaste besiktning",
        ]),
        inspection_date: extract(&["Besiktningsdatum", "Inspection date", "Senast godkänd"]),
        inspection_mileage: parse_int(&["Mätarställning", "Odometer"]),
        inspection_notes: extract(&["Anmärkningar", "Remarks"]),
    })
}
