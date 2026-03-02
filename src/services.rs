use chrono::Utc;
use sqlx::PgPool;

use crate::ai::{ComparisonResult, LlmClient};
use crate::cache::RedisCache;
use crate::external::VehicleDataProvider;
use crate::models::*;
use crate::repositories;

/// Core vehicle service: orchestrates cache → DB → external API → AI enrichment.
#[derive(Clone)]
pub struct VehicleService {
    pool: PgPool,
    cache: RedisCache,
    llm: LlmClient,
    external: VehicleDataProvider,
}

impl VehicleService {
    pub fn new(pool: PgPool, cache: RedisCache, llm: LlmClient, external: VehicleDataProvider) -> Self {
        Self { pool, cache, llm, external }
    }

    /// Full premium vehicle query: cache → DB → external API → AI enrichment → cache → return.
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

        // Step 3: If not in DB, fetch from external APIs (Biluppgifter / Transportstyrelsen)
        let vehicle = match repositories::find_vehicle_by_plate(&self.pool, &plate).await? {
            Some(v) => v,
            None => {
                tracing::info!(plate = %plate, "vehicle not in DB, fetching from external API");
                let fetch_result = self.external.fetch_and_store(&self.pool, &plate).await?;

                if fetch_result.stolen {
                    tracing::warn!(plate = %plate, "⚠️ VEHICLE REPORTED STOLEN");
                }

                repositories::find_vehicle_by_plate(&self.pool, &plate)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Vehicle {} not found after external fetch", plate))?
            }
        };

        // Step 4: Gather all related data
        let inspections = repositories::find_inspections_by_plate(&self.pool, &plate)
            .await
            .unwrap_or_default();

        let ownership = repositories::find_ownership_by_plate(&self.pool, &plate)
            .await
            .unwrap_or_default();

        let recalls = if let Some(ref vin) = vehicle.vin {
            repositories::find_recalls_by_vin(&self.pool, vin)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        let listings = repositories::find_listings_by_plate(&self.pool, &plate)
            .await
            .unwrap_or_default();

        // Step 5: Build base report
        let mut report = self.build_report(&vehicle, &ownership, &inspections, &recalls);

        // Step 6: AI enrichment — deep investigative analysis
        match self.llm.evaluate(&report, &listings, locale).await {
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

    /// Free-tier report: basic info only, enough to hook but not the full analysis.
    pub async fn query_free(&self, plate: &str) -> anyhow::Result<FreeReport> {
        let plate = plate.to_uppercase().trim().to_string();

        let vehicle = match repositories::find_vehicle_by_plate(&self.pool, &plate).await? {
            Some(v) => v,
            None => {
                self.external.fetch_and_store(&self.pool, &plate).await?;
                repositories::find_vehicle_by_plate(&self.pool, &plate)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Vehicle not found: {}", plate))?
            }
        };

        let inspections = repositories::find_inspections_by_plate(&self.pool, &plate)
            .await
            .unwrap_or_default();

        let recall_count = if let Some(ref vin) = vehicle.vin {
            repositories::find_recalls_by_vin(&self.pool, vin)
                .await
                .map(|r| r.len())
                .unwrap_or(0)
        } else {
            0
        };

        Ok(FreeReport {
            plate: vehicle.plate,
            make: vehicle.make.unwrap_or_default(),
            model: vehicle.model.unwrap_or_default(),
            year: vehicle.year.unwrap_or(0),
            fuel_type: vehicle.fuel_type.unwrap_or_default(),
            mileage_latest: vehicle.mileage_latest,
            owner_count: vehicle.owner_count,
            color: vehicle.color,
            last_inspection_result: inspections.first().and_then(|i| i.result.clone()),
            last_inspection_date: inspections.first().and_then(|i| i.date.map(|d| d.to_string())),
            recall_count,
            annual_tax_sek: vehicle.tax_per_year.unwrap_or(0),
            unlock_message: "🔒 Unlock the full report for AI analysis, risk score, price estimate, negotiation strategy, and expert recommendation.".into(),
        })
    }

    /// Compare multiple vehicles side-by-side with AI recommendation.
    pub async fn compare_vehicles(&self, plates: &[String], locale: &str) -> anyhow::Result<ComparisonResult> {
        if plates.is_empty() || plates.len() > 5 {
            anyhow::bail!("Provide 2-5 plates for comparison");
        }

        let mut reports = Vec::new();
        let mut all_listings = Vec::new();

        for plate in plates {
            let report = self.query_vehicle(plate, locale).await?;
            let listings = repositories::find_listings_by_plate(&self.pool, &plate.to_uppercase())
                .await
                .unwrap_or_default();
            reports.push(report);
            all_listings.push(listings);
        }

        match self.llm.compare(&reports, &all_listings, locale).await? {
            Some(result) => Ok(result),
            None => {
                // Fallback: basic comparison without AI
                let comparison = reports.iter().enumerate().map(|(i, r)| {
                    crate::ai::ComparisonItem {
                        plate: r.plate.clone(),
                        rank: (i + 1) as i32,
                        score: 0,
                        verdict: format!("{} {} {}", r.basic.make, r.basic.model, r.basic.year),
                    }
                }).collect();

                Ok(ComparisonResult {
                    comparison,
                    best_pick: crate::ai::BestPick {
                        plate: reports.first().map(|r| r.plate.clone()).unwrap_or_default(),
                        reason: "AI analysis unavailable — configure AI_API_KEY for recommendations".into(),
                    },
                    ai_summary: "AI comparison unavailable. Configure AI_API_KEY for detailed analysis.".into(),
                })
            }
        }
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
        ownership: &[OwnershipRow],
        inspections: &[InspectionRow],
        recalls: &[RecallRow],
    ) -> VehicleReport {
        let basic = vehicle.to_basic_info();
        let insp_items: Vec<InspectionItem> = inspections.iter().map(|r| r.to_item()).collect();
        let recall_items: Vec<RecallItem> = recalls.iter().map(|r| r.to_item()).collect();
        let open_recalls = recall_items.iter().filter(|r| !r.fix_available).count();

        let ownership_events: Vec<OwnershipEvent> = ownership.iter().map(|o| OwnershipEvent {
            date: o.date.map(|d| d.to_string()).unwrap_or_default(),
            event: o.event.clone().unwrap_or_default(),
        }).collect();

        VehicleReport {
            plate: vehicle.plate.clone(),
            cached: false,
            generated_at: String::new(),
            basic,
            ownership: OwnershipInfo {
                owner_count: vehicle.owner_count.unwrap_or(0),
                history: ownership_events,
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
