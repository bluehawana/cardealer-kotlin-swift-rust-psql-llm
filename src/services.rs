use chrono::Utc;
use sqlx::PgPool;

use crate::ai::LlmClient;
use crate::cache::RedisCache;
use crate::models::*;
use crate::repositories;

/// Core vehicle service: orchestrates cache → DB → external API → AI enrichment.
#[derive(Clone)]
pub struct VehicleService {
    pool: PgPool,
    cache: RedisCache,
    llm: LlmClient,
}

impl VehicleService {
    pub fn new(pool: PgPool, cache: RedisCache, llm: LlmClient) -> Self {
        Self { pool, cache, llm }
    }

    /// Full vehicle query: cache → DB → external → AI → cache → return.
    pub async fn query_vehicle(&self, plate: &str, locale: &str) -> anyhow::Result<VehicleReport> {
        let plate = plate.to_uppercase().trim().to_string();

        // Step 1: Check Redis cache
        match self.cache.get(&plate).await {
            Ok(Some(cached_json)) => {
                if let Ok(mut report) = serde_json::from_str::<VehicleReport>(&cached_json) {
                    report.cached = true;
                    tracing::info!(plate = %plate, "cache hit (Redis)");
                    return Ok(report);
                }
            }
            Err(e) => tracing::warn!(plate = %plate, error = %e, "Redis cache error"),
            _ => {}
        }

        // Step 2: Check PostgreSQL for cached report
        if let Ok(Some(mut report)) = repositories::find_latest_report(&self.pool, &plate).await {
            report.cached = true;
            if let Ok(json) = serde_json::to_string(&report) {
                let _ = self.cache.set(&plate, &json).await;
            }
            tracing::info!(plate = %plate, "cache hit (PostgreSQL)");
            return Ok(report);
        }

        // Step 3: Fetch vehicle data from DB
        let vehicle = repositories::find_vehicle_by_plate(&self.pool, &plate)
            .await?
            .ok_or_else(|| anyhow::anyhow!(
                "Vehicle not found: {} (external API integration pending)", plate
            ))?;

        // Step 4: Gather related data
        let inspections = repositories::find_inspections_by_plate(&self.pool, &plate)
            .await
            .unwrap_or_default();

        let recalls = if let Some(ref vin) = vehicle.vin {
            repositories::find_recalls_by_vin(&self.pool, vin)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        // Step 5: Build report
        let mut report = self.build_report(&vehicle, &inspections, &recalls);

        // Step 6: AI enrichment
        match self.llm.evaluate(&report, locale).await {
            Ok(Some(enrichment)) => {
                report.risk = enrichment.risk;
                report.price_estimate = enrichment.price_estimate;
                report.condition_inference = enrichment.condition_inference;
                report.suitability = enrichment.suitability;
                report.ai_summary = enrichment.ai_summary;
            }
            Ok(None) => {
                tracing::info!("AI enrichment skipped (no API key)");
            }
            Err(e) => {
                tracing::warn!(error = %e, "AI enrichment failed, returning basic report");
            }
        }

        report.generated_at = Utc::now().to_rfc3339();
        report.cached = false;

        // Step 7: Save to DB and cache
        if let Err(e) = repositories::save_report(&self.pool, &report, locale).await {
            tracing::warn!(error = %e, "failed to save report to DB");
        }
        if let Ok(json) = serde_json::to_string(&report) {
            let _ = self.cache.set(&plate, &json).await;
        }

        Ok(report)
    }

    /// Get only basic vehicle info.
    pub async fn get_basic_info(&self, plate: &str) -> anyhow::Result<BasicInfo> {
        let vehicle = repositories::find_vehicle_by_plate(&self.pool, &plate.to_uppercase())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Vehicle not found: {}", plate))?;
        Ok(vehicle.to_basic_info())
    }

    /// Get inspection history for a plate.
    pub async fn get_inspections(&self, plate: &str) -> anyhow::Result<InspectionInfo> {
        let rows = repositories::find_inspections_by_plate(&self.pool, &plate.to_uppercase()).await?;
        let items: Vec<InspectionItem> = rows.iter().map(|r| r.to_item()).collect();
        Ok(InspectionInfo {
            total_inspections: items.len(),
            last_inspection: items.first().map(|i| i.date.clone()),
            last_result: items.first().map(|i| i.result.clone()),
            history: items,
        })
    }

    /// Get recalls for a VIN.
    pub async fn get_recalls(&self, vin: &str) -> anyhow::Result<RecallInfo> {
        let rows = repositories::find_recalls_by_vin(&self.pool, vin).await?;
        let items: Vec<RecallItem> = rows.iter().map(|r| r.to_item()).collect();
        let open = items.iter().filter(|r| !r.fix_available).count();
        Ok(RecallInfo {
            total_recalls: items.len(),
            open_recalls: open,
            recalls: items,
        })
    }

    fn build_report(
        &self,
        vehicle: &VehicleRow,
        inspections: &[InspectionRow],
        recalls: &[RecallRow],
    ) -> VehicleReport {
        let basic = vehicle.to_basic_info();
        let insp_items: Vec<InspectionItem> = inspections.iter().map(|r| r.to_item()).collect();
        let recall_items: Vec<RecallItem> = recalls.iter().map(|r| r.to_item()).collect();
        let open_recalls = recall_items.iter().filter(|r| !r.fix_available).count();

        VehicleReport {
            plate: vehicle.plate.clone(),
            cached: false,
            generated_at: String::new(),
            basic,
            ownership: OwnershipInfo {
                owner_count: vehicle.owner_count.unwrap_or(0),
                history: vec![],
            },
            inspection: InspectionInfo {
                total_inspections: insp_items.len(),
                last_inspection: insp_items.first().map(|i| i.date.clone()),
                last_result: insp_items.first().map(|i| i.result.clone()),
                history: insp_items,
            },
            recalls: RecallInfo {
                total_recalls: recall_items.len(),
                open_recalls,
                recalls: recall_items,
            },
            tax: TaxInfo {
                annual_tax_sek: vehicle.tax_per_year.unwrap_or(0),
                co2_emission_g_km: vehicle.co2_emission.unwrap_or(0),
                euro_standard: vehicle.euro_standard.clone().unwrap_or_default(),
            },
            risk: RiskInfo::default(),
            price_estimate: PriceEstimate::default(),
            condition_inference: ConditionInference::default(),
            suitability: Suitability::default(),
            ai_summary: String::new(),
        }
    }
}
