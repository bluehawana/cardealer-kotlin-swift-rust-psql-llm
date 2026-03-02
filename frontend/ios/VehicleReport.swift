import Foundation

// MARK: - Main Report

/// Complete Vehicle Report — the main API response from POST /api/v1/vehicles/query.
struct VehicleReport: Codable {
    let plate: String
    let cached: Bool
    let generatedAt: String
    let basic: BasicInfo
    let ownership: OwnershipInfo
    let inspection: InspectionInfo
    let recalls: RecallInfo
    let tax: TaxInfo
    let risk: RiskInfo
    let priceEstimate: PriceEstimate
    let conditionInference: ConditionInference
    let suitability: Suitability
    let aiSummary: String

    enum CodingKeys: String, CodingKey {
        case plate, cached, basic, ownership, inspection, recalls, tax, risk, suitability
        case generatedAt = "generated_at"
        case priceEstimate = "price_estimate"
        case conditionInference = "condition_inference"
        case aiSummary = "ai_summary"
    }
}

// MARK: - Basic Info

struct BasicInfo: Codable {
    let plate: String
    let vin: String?
    let make: String
    let model: String
    let variant: String?
    let year: Int
    let bodyType: String
    let fuelType: String
    let engineCode: String?
    let engineDisplacementL: Double?
    let enginePowerHp: Int?
    let engineTorqueNm: Int?
    let gearboxType: String?
    let gearboxGears: Int?
    let drivetrainType: String?
    let is4x4: Bool
    let color: String?
    let seats: Int?
    let curbWeightKg: Int?
    let firstRegistration: String?
    let countryOrigin: String?
    let mileageLatest: Int?
    let ownerCount: Int?

    enum CodingKeys: String, CodingKey {
        case plate, vin, make, model, variant, year, color, seats
        case bodyType = "body_type"
        case fuelType = "fuel_type"
        case engineCode = "engine_code"
        case engineDisplacementL = "engine_displacement_l"
        case enginePowerHp = "engine_power_hp"
        case engineTorqueNm = "engine_torque_nm"
        case gearboxType = "gearbox_type"
        case gearboxGears = "gearbox_gears"
        case drivetrainType = "drivetrain_type"
        case is4x4 = "is_4x4"
        case curbWeightKg = "curb_weight_kg"
        case firstRegistration = "first_registration"
        case countryOrigin = "country_origin"
        case mileageLatest = "mileage_latest"
        case ownerCount = "owner_count"
    }
}

// MARK: - Ownership

struct OwnershipInfo: Codable {
    let ownerCount: Int
    let history: [OwnershipEvent]

    enum CodingKeys: String, CodingKey {
        case ownerCount = "owner_count"
        case history
    }
}

struct OwnershipEvent: Codable {
    let date: String
    let event: String
}

// MARK: - Inspection

struct InspectionInfo: Codable {
    let totalInspections: Int
    let lastInspection: String?
    let lastResult: String?
    let history: [Inspection]

    enum CodingKeys: String, CodingKey {
        case totalInspections = "total_inspections"
        case lastInspection = "last_inspection"
        case lastResult = "last_result"
        case history
    }
}

struct Inspection: Codable {
    let date: String
    let mileageKm: Int
    let result: String
    let notes: String?

    enum CodingKeys: String, CodingKey {
        case date, result, notes
        case mileageKm = "mileage_km"
    }
}

// MARK: - Recalls

struct RecallInfo: Codable {
    let totalRecalls: Int
    let openRecalls: Int
    let recalls: [Recall]

    enum CodingKeys: String, CodingKey {
        case totalRecalls = "total_recalls"
        case openRecalls = "open_recalls"
        case recalls
    }
}

struct Recall: Codable {
    let vin: String
    let source: String
    let description: String
    let severity: String
    let fixAvailable: Bool

    enum CodingKeys: String, CodingKey {
        case vin, source, description, severity
        case fixAvailable = "fix_available"
    }
}

// MARK: - Tax

struct TaxInfo: Codable {
    let annualTaxSek: Int
    let co2EmissionGKm: Int
    let euroStandard: String

    enum CodingKeys: String, CodingKey {
        case annualTaxSek = "annual_tax_sek"
        case co2EmissionGKm = "co2_emission_g_km"
        case euroStandard = "euro_standard"
    }
}

// MARK: - Risk

struct RiskInfo: Codable {
    let score: Int
    let level: String
    let factors: [String]
    let description: String
}

// MARK: - Price Estimate

struct PriceEstimate: Codable {
    let minSek: Int
    let maxSek: Int
    let medianSek: Int
    let currency: String
    let source: String

    enum CodingKeys: String, CodingKey {
        case currency, source
        case minSek = "min_sek"
        case maxSek = "max_sek"
        case medianSek = "median_sek"
    }
}

// MARK: - Condition Inference

struct ConditionInference: Codable {
    let rating: String
    let confidence: Double
    let indicators: [String]
    let description: String
}

// MARK: - Suitability

struct Suitability: Codable {
    let score: Int
    let recommendation: String
    let pros: [String]
    let cons: [String]
}

// MARK: - User

struct CreditBalance: Codable {
    let userId: Int
    let plan: String
    let credits: Int

    enum CodingKeys: String, CodingKey {
        case userId = "user_id"
        case plan, credits
    }
}

struct QueryHistoryItem: Codable {
    let plate: String
    let queriedAt: String
    let cached: Bool

    enum CodingKeys: String, CodingKey {
        case plate, cached
        case queriedAt = "queried_at"
    }
}
