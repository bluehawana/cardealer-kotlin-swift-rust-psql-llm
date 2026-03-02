use sqlx::PgPool;

use crate::models::*;

// ─── Vehicle Repo ────────────────────────────────────────────────

pub async fn find_vehicle_by_plate(pool: &PgPool, plate: &str) -> anyhow::Result<Option<VehicleRow>> {
    let row = sqlx::query_as::<_, VehicleRow>(
        r#"SELECT id, plate, vin, make, model, variant, year, body_type, fuel_type,
            engine_code, engine_displacement_l, engine_power_hp, engine_torque_nm,
            gearbox_type, gearbox_gears, drivetrain_type, rear_diff_lock, center_diff_lock,
            is_4x4, length_mm, width_mm, height_m, clearance_mm, curb_weight_kg,
            gross_weight_kg, seats, color, imported, first_registration, country_origin,
            owner_count, mileage_latest, tax_per_year, co2_emission,
            euro_standard, raw_source_json
        FROM vehicles WHERE plate = $1"#,
    )
    .bind(plate)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn recent_plates(pool: &PgPool, days: i32) -> anyhow::Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT plate FROM vehicles WHERE updated_at > NOW() - ($1 || ' days')::INTERVAL ORDER BY updated_at DESC",
    )
    .bind(days.to_string())
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// ─── Inspection Repo ─────────────────────────────────────────────

pub async fn find_inspections_by_plate(pool: &PgPool, plate: &str) -> anyhow::Result<Vec<InspectionRow>> {
    let rows = sqlx::query_as::<_, InspectionRow>(
        "SELECT id, plate, date, mileage_km, result, notes FROM inspections WHERE plate = $1 ORDER BY date DESC",
    )
    .bind(plate)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// ─── Recall Repo ─────────────────────────────────────────────────

pub async fn find_recalls_by_vin(pool: &PgPool, vin: &str) -> anyhow::Result<Vec<RecallRow>> {
    let rows = sqlx::query_as::<_, RecallRow>(
        "SELECT id, vin, source, description, severity, fix_available FROM recalls WHERE vin = $1 ORDER BY id DESC",
    )
    .bind(vin)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// ─── Report Repo ─────────────────────────────────────────────────

pub async fn find_latest_report(pool: &PgPool, plate: &str) -> anyhow::Result<Option<VehicleReport>> {
    let row = sqlx::query_as::<_, ReportRow>(
        "SELECT id, plate, summary_json, risk_score, price_min, price_max, locale FROM reports WHERE plate = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(plate)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            if let Some(json) = r.summary_json {
                let report: VehicleReport = serde_json::from_value(json)?;
                Ok(Some(report))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

pub async fn save_report(pool: &PgPool, report: &VehicleReport, locale: &str) -> anyhow::Result<()> {
    let json = serde_json::to_value(report)?;
    sqlx::query(
        "INSERT INTO reports (plate, summary_json, risk_score, price_min, price_max, locale) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&report.plate)
    .bind(&json)
    .bind(report.risk.score)
    .bind(report.price_estimate.min_sek)
    .bind(report.price_estimate.max_sek)
    .bind(locale)
    .execute(pool)
    .await?;

    Ok(())
}
