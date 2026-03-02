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
    pub engine_power: Option<i32>,       // kW
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
                let result = self.store_transportstyrelsen_data(pool, &plate, &data).await?;
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
            .header("Authorization", format!("Bearer {}", self.biluppgifter_api_key))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Biluppgifter API error {}: {}", status, body);
        }

        let response: BiluppgifterResponse = resp.json().await?;
        response.data.ok_or_else(|| anyhow::anyhow!("No vehicle data returned for plate {}", plate))
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
    async fn fetch_from_transportstyrelsen(&self, plate: &str) -> anyhow::Result<TransportstyrelsenData> {
        let url = "https://fu-regnr.transportstyrelsen.se/externalresources/LicensePlate";

        let resp = self
            .http
            .post(url)
            .form(&[("regnr", plate)])
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("Transportstyrelsen returned status {}", resp.status());
        }

        let html = resp.text().await?;

        // Parse the HTML response for vehicle data
        let data = parse_transportstyrelsen_html(&html, plate)?;
        Ok(data)
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
                curb_weight_kg, owner_count, co2_emission, euro_standard, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            ON CONFLICT (plate) DO UPDATE SET
                make = COALESCE(EXCLUDED.make, vehicles.make),
                model = COALESCE(EXCLUDED.model, vehicles.model),
                year = COALESCE(EXCLUDED.year, vehicles.year),
                fuel_type = COALESCE(EXCLUDED.fuel_type, vehicles.fuel_type),
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
        .bind(data.owner_count)
        .bind(data.co2_emission)
        .bind(&data.euro_standard)
        .execute(pool)
        .await?;

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
    pub owner_count: Option<i32>,
    pub co2_emission: Option<i32>,
    pub euro_standard: Option<String>,
}

/// Parse Transportstyrelsen HTML response to extract vehicle data.
fn parse_transportstyrelsen_html(html: &str, _plate: &str) -> anyhow::Result<TransportstyrelsenData> {
    // The Transportstyrelsen page returns a table with vehicle data.
    // We parse key fields from the HTML using simple string matching.
    // This is a basic parser; for production, use a proper HTML parser crate.

    let extract = |label: &str| -> Option<String> {
        html.find(label).and_then(|pos| {
            let after = &html[pos + label.len()..];
            // Look for the next value in a <strong> or <td> tag
            after.find('>').and_then(|start| {
                let rest = &after[start + 1..];
                rest.find('<').map(|end| rest[..end].trim().to_string())
            })
        })
    };

    let year = extract("Årsmodell").and_then(|s| s.parse::<i32>().ok())
        .or_else(|| extract("Model year").and_then(|s| s.parse::<i32>().ok()));

    let weight = extract("Tjänstevikt").and_then(|s| {
        s.replace(" kg", "").replace('\u{a0}', "").trim().parse::<i32>().ok()
    });

    Ok(TransportstyrelsenData {
        make: extract("Fabrikat").or_else(|| extract("Make")),
        model: extract("Handelsbenämning").or_else(|| extract("Model")),
        year,
        fuel_type: extract("Drivmedel").or_else(|| extract("Fuel")),
        color: extract("Färg").or_else(|| extract("Color")),
        curb_weight_kg: weight,
        owner_count: None, // Not available from public page
        co2_emission: extract("CO2").and_then(|s| s.replace(" g/km", "").trim().parse::<i32>().ok()),
        euro_standard: extract("Miljöklass").or_else(|| extract("Euro")),
    })
}
