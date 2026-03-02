package com.cardeal.models

import com.google.gson.annotations.SerializedName

/**
 * Complete Vehicle Report — the main API response from POST /api/v1/vehicles/query.
 */
data class VehicleReport(
    val plate: String,
    val cached: Boolean,
    @SerializedName("generated_at") val generatedAt: String,
    val basic: BasicInfo,
    val ownership: OwnershipInfo,
    val inspection: InspectionInfo,
    val recalls: RecallInfo,
    val tax: TaxInfo,
    val risk: RiskInfo,
    @SerializedName("price_estimate") val priceEstimate: PriceEstimate,
    @SerializedName("condition_inference") val conditionInference: ConditionInference,
    val suitability: Suitability,
    @SerializedName("ai_summary") val aiSummary: String
)

data class BasicInfo(
    val plate: String,
    val vin: String? = null,
    val make: String,
    val model: String,
    val variant: String? = null,
    val year: Int,
    @SerializedName("body_type") val bodyType: String,
    @SerializedName("fuel_type") val fuelType: String,
    @SerializedName("engine_code") val engineCode: String? = null,
    @SerializedName("engine_displacement_l") val engineDisplacementL: Double? = null,
    @SerializedName("engine_power_hp") val enginePowerHp: Int? = null,
    @SerializedName("engine_torque_nm") val engineTorqueNm: Int? = null,
    @SerializedName("gearbox_type") val gearboxType: String? = null,
    @SerializedName("gearbox_gears") val gearboxGears: Int? = null,
    @SerializedName("drivetrain_type") val drivetrainType: String? = null,
    @SerializedName("is_4x4") val is4x4: Boolean = false,
    val color: String? = null,
    val seats: Int? = null,
    @SerializedName("curb_weight_kg") val curbWeightKg: Int? = null,
    @SerializedName("first_registration") val firstRegistration: String? = null,
    @SerializedName("country_origin") val countryOrigin: String? = null,
    @SerializedName("mileage_latest") val mileageLatest: Int? = null,
    @SerializedName("owner_count") val ownerCount: Int? = null
)

data class OwnershipInfo(
    @SerializedName("owner_count") val ownerCount: Int,
    val history: List<OwnershipEvent> = emptyList()
)

data class OwnershipEvent(
    val date: String,
    val event: String
)

data class InspectionInfo(
    @SerializedName("total_inspections") val totalInspections: Int,
    @SerializedName("last_inspection") val lastInspection: String? = null,
    @SerializedName("last_result") val lastResult: String? = null,
    val history: List<Inspection> = emptyList()
)

data class Inspection(
    val date: String,
    @SerializedName("mileage_km") val mileageKm: Int,
    val result: String,
    val notes: String? = null
)

data class RecallInfo(
    @SerializedName("total_recalls") val totalRecalls: Int,
    @SerializedName("open_recalls") val openRecalls: Int,
    val recalls: List<Recall> = emptyList()
)

data class Recall(
    val vin: String,
    val source: String,
    val description: String,
    val severity: String,
    @SerializedName("fix_available") val fixAvailable: Boolean
)

data class TaxInfo(
    @SerializedName("annual_tax_sek") val annualTaxSek: Int,
    @SerializedName("co2_emission_g_km") val co2EmissionGKm: Int,
    @SerializedName("euro_standard") val euroStandard: String
)

data class RiskInfo(
    val score: Int,
    val level: String,
    val factors: List<String> = emptyList(),
    val description: String
)

data class PriceEstimate(
    @SerializedName("min_sek") val minSek: Int,
    @SerializedName("max_sek") val maxSek: Int,
    @SerializedName("median_sek") val medianSek: Int,
    val currency: String,
    val source: String
)

data class ConditionInference(
    val rating: String,
    val confidence: Double,
    val indicators: List<String> = emptyList(),
    val description: String
)

data class Suitability(
    val score: Int,
    val recommendation: String,
    val pros: List<String> = emptyList(),
    val cons: List<String> = emptyList()
)

/**
 * Credit balance response from GET /api/v1/user/credits.
 */
data class CreditBalance(
    @SerializedName("user_id") val userId: Int,
    val plan: String,
    val credits: Int
)

/**
 * Query history item from GET /api/v1/user/history.
 */
data class QueryHistoryItem(
    val plate: String,
    @SerializedName("queried_at") val queriedAt: String,
    val cached: Boolean
)
