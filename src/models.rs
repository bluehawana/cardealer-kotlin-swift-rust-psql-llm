use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ─── Database Models ─────────────────────────────────────────────

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)] // Fields read by SQLx at runtime
pub struct VehicleRow {
    pub id: i32,
    pub plate: String,
    pub vin: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub variant: Option<String>,
    pub year: Option<i32>,
    pub body_type: Option<String>,
    pub fuel_type: Option<String>,
    pub engine_code: Option<String>,
    pub engine_displacement_l: Option<f64>,
    pub engine_power_hp: Option<i32>,
    pub engine_torque_nm: Option<i32>,
    pub gearbox_type: Option<String>,
    pub gearbox_gears: Option<i32>,
    pub drivetrain_type: Option<String>,
    pub rear_diff_lock: bool,
    pub center_diff_lock: bool,
    pub is_4x4: bool,
    pub length_mm: Option<i32>,
    pub width_mm: Option<i32>,
    pub height_m: Option<f64>,
    pub clearance_mm: Option<i32>,
    pub curb_weight_kg: Option<i32>,
    pub gross_weight_kg: Option<i32>,
    pub seats: Option<i32>,
    pub color: Option<String>,
    pub imported: bool,
    pub first_registration: Option<NaiveDate>,
    pub country_origin: Option<String>,
    pub owner_count: Option<i32>,
    pub mileage_latest: Option<i32>,
    pub tax_per_year: Option<i32>,
    pub co2_emission: Option<i32>,
    pub euro_standard: Option<String>,
    pub raw_source_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct InspectionRow {
    pub id: i32,
    pub plate: String,
    pub date: Option<NaiveDate>,
    pub mileage_km: Option<i32>,
    pub result: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct RecallRow {
    pub id: i32,
    pub vin: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub fix_available: bool,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ReportRow {
    pub id: i32,
    pub plate: String,
    pub summary_json: Option<serde_json::Value>,
    pub risk_score: Option<i32>,
    pub price_min: Option<i32>,
    pub price_max: Option<i32>,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ListingRow {
    pub id: i32,
    pub plate: String,
    pub source: Option<String>,
    pub seller_type: Option<String>,
    pub price_sek: Option<i32>,
    pub mileage_km: Option<i32>,
    pub listed_at: Option<NaiveDate>,
    pub delisted_at: Option<NaiveDate>,
    pub url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct OwnershipRow {
    pub id: i32,
    pub plate: String,
    pub date: Option<NaiveDate>,
    pub event: Option<String>,
}

// ─── API Response Models ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleReport {
    pub plate: String,
    pub cached: bool,
    pub generated_at: String,
    pub basic: BasicInfo,
    pub ownership: OwnershipInfo,
    pub inspection: InspectionInfo,
    pub recalls: RecallInfo,
    pub tax: TaxInfo,
    pub risk: RiskInfo,
    pub price_estimate: PriceEstimate,
    pub condition_inference: ConditionInference,
    pub suitability: Suitability,
    pub ai_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicInfo {
    pub plate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vin: Option<String>,
    pub make: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub year: i32,
    pub body_type: String,
    pub fuel_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_displacement_l: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_power_hp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_torque_nm: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gearbox_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gearbox_gears: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drivetrain_type: Option<String>,
    pub is_4x4: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seats: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curb_weight_kg: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_registration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mileage_latest: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipInfo {
    pub owner_count: i32,
    pub history: Vec<OwnershipEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipEvent {
    pub date: String,
    pub event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionInfo {
    pub total_inspections: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_inspection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_result: Option<String>,
    pub history: Vec<InspectionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionItem {
    pub date: String,
    pub mileage_km: i32,
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallInfo {
    pub total_recalls: usize,
    pub open_recalls: usize,
    pub recalls: Vec<RecallItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallItem {
    pub vin: String,
    pub source: String,
    pub description: String,
    pub severity: String,
    pub fix_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxInfo {
    pub annual_tax_sek: i32,
    pub co2_emission_g_km: i32,
    pub euro_standard: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskInfo {
    pub score: i32,
    pub level: String,
    pub factors: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceEstimate {
    pub min_sek: i32,
    pub max_sek: i32,
    pub median_sek: i32,
    pub currency: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConditionInference {
    pub rating: String,
    pub confidence: f64,
    pub indicators: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Suitability {
    pub score: i32,
    pub recommendation: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

// ─── Request Models ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub plate: String,
    #[serde(default = "default_locale")]
    pub locale: String,
}

fn default_locale() -> String {
    "en".into()
}

#[derive(Debug, Deserialize)]
pub struct PlateQuery {
    pub plate: String,
}

#[derive(Debug, Deserialize)]
pub struct VinQuery {
    pub vin: String,
}

#[derive(Debug, Deserialize)]
pub struct CompareRequest {
    pub plates: Vec<String>,
    #[serde(default = "default_locale")]
    pub locale: String,
}

/// Free-tier report: shows enough to convince, locks the premium analysis.
#[derive(Debug, Clone, Serialize)]
pub struct FreeReport {
    pub plate: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub fuel_type: String,
    pub mileage_latest: Option<i32>,
    pub owner_count: Option<i32>,
    pub color: Option<String>,
    pub last_inspection_result: Option<String>,
    pub last_inspection_date: Option<String>,
    pub recall_count: usize,
    pub annual_tax_sek: i32,
    pub unlock_message: String,
}

// ─── User Models (TODO: implement when user auth is added) ──────

// ─── Error Response ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// ─── Conversions ─────────────────────────────────────────────────

impl VehicleRow {
    pub fn to_basic_info(&self) -> BasicInfo {
        BasicInfo {
            plate: self.plate.clone(),
            vin: self.vin.clone(),
            make: self.make.clone().unwrap_or_default(),
            model: self.model.clone().unwrap_or_default(),
            variant: self.variant.clone(),
            year: self.year.unwrap_or(0),
            body_type: self.body_type.clone().unwrap_or_default(),
            fuel_type: self.fuel_type.clone().unwrap_or_default(),
            engine_code: self.engine_code.clone(),
            engine_displacement_l: self.engine_displacement_l,
            engine_power_hp: self.engine_power_hp,
            engine_torque_nm: self.engine_torque_nm,
            gearbox_type: self.gearbox_type.clone(),
            gearbox_gears: self.gearbox_gears,
            drivetrain_type: self.drivetrain_type.clone(),
            is_4x4: self.is_4x4,
            color: self.color.clone(),
            seats: self.seats,
            curb_weight_kg: self.curb_weight_kg,
            first_registration: self.first_registration.map(|d| d.to_string()),
            country_origin: self.country_origin.clone(),
            mileage_latest: self.mileage_latest,
            owner_count: self.owner_count,
        }
    }
}

impl InspectionRow {
    pub fn to_item(&self) -> InspectionItem {
        InspectionItem {
            date: self.date.map(|d| d.to_string()).unwrap_or_default(),
            mileage_km: self.mileage_km.unwrap_or(0),
            result: self.result.clone().unwrap_or_default(),
            notes: self.notes.clone(),
        }
    }
}

impl RecallRow {
    pub fn to_item(&self) -> RecallItem {
        RecallItem {
            vin: self.vin.clone(),
            source: self.source.clone().unwrap_or_default(),
            description: self.description.clone().unwrap_or_default(),
            severity: self.severity.clone().unwrap_or_default(),
            fix_available: self.fix_available,
        }
    }
}
