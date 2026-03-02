use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde_json::json;

use crate::models::*;
use crate::services::VehicleService;

/// Build the application router with all endpoints.
pub fn build_router(service: VehicleService) -> Router {
    Router::new()
        // Health check
        .route("/healthz", get(healthz))
        // Vehicle endpoints
        .route("/api/v1/vehicles/query", post(query_vehicle))
        .route("/api/v1/vehicles/free", get(query_free))
        .route("/api/v1/vehicles/compare", post(compare_vehicles))
        .route("/api/v1/vehicles/basic", get(get_basic))
        .route("/api/v1/vehicles/inspection", get(get_inspection))
        .route("/api/v1/vehicles/recalls", get(get_recalls))
        // Valuation
        .route("/api/v1/valuation", get(get_valuation))
        // AI
        .route("/api/v1/ai/report", post(generate_ai_report))
        // User
        .route("/api/v1/user/credits", get(get_credits))
        .route("/api/v1/user/consume", post(consume_credits))
        .route("/api/v1/user/history", get(get_history))
        // Shared state
        .with_state(service)
}

// ─── Health ──────────────────────────────────────────────────────

async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok", "service": "CarDeal Vehicle Intelligence API" }))
}

// ─── Free Report ─────────────────────────────────────────────────

async fn query_free(
    State(svc): State<VehicleService>,
    Query(q): Query<PlateQuery>,
) -> impl IntoResponse {
    match svc.query_free(&q.plate).await {
        Ok(report) => (StatusCode::OK, Json(json!(report))),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "vehicle_not_found".into(),
                message: e.to_string(),
            })),
        ),
    }
}

// ─── Premium Full Report ─────────────────────────────────────────

async fn query_vehicle(
    State(svc): State<VehicleService>,
    Json(req): Json<QueryRequest>,
) -> impl IntoResponse {
    match svc.query_vehicle(&req.plate, &req.locale).await {
        Ok(report) => (StatusCode::OK, Json(json!(report))),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "vehicle_not_found".into(),
                message: e.to_string(),
            })),
        ),
    }
}

// ─── Multi-Vehicle Comparison ────────────────────────────────────

async fn compare_vehicles(
    State(svc): State<VehicleService>,
    Json(req): Json<CompareRequest>,
) -> impl IntoResponse {
    match svc.compare_vehicles(&req.plates, &req.locale).await {
        Ok(result) => (StatusCode::OK, Json(json!(result))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!(ErrorResponse {
                error: "comparison_failed".into(),
                message: e.to_string(),
            })),
        ),
    }
}

// ─── Modular Endpoints ──────────────────────────────────────────

async fn get_basic(
    State(svc): State<VehicleService>,
    Query(q): Query<PlateQuery>,
) -> impl IntoResponse {
    match svc.get_basic_info(&q.plate).await {
        Ok(info) => (StatusCode::OK, Json(json!(info))),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "vehicle_not_found".into(),
                message: e.to_string(),
            })),
        ),
    }
}

async fn get_inspection(
    State(svc): State<VehicleService>,
    Query(q): Query<PlateQuery>,
) -> impl IntoResponse {
    match svc.get_inspections(&q.plate).await {
        Ok(info) => (StatusCode::OK, Json(json!(info))),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "not_found".into(),
                message: e.to_string(),
            })),
        ),
    }
}

async fn get_recalls(
    State(svc): State<VehicleService>,
    Query(q): Query<VinQuery>,
) -> impl IntoResponse {
    match svc.get_recalls(&q.vin).await {
        Ok(info) => (StatusCode::OK, Json(json!(info))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(ErrorResponse {
                error: "internal_error".into(),
                message: e.to_string(),
            })),
        ),
    }
}

async fn get_valuation(
    State(svc): State<VehicleService>,
    Query(q): Query<PlateQuery>,
) -> impl IntoResponse {
    match svc.query_vehicle(&q.plate, "en").await {
        Ok(report) => (
            StatusCode::OK,
            Json(json!({
                "plate": report.plate,
                "price_estimate": report.price_estimate,
                "basic": report.basic,
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "not_found".into(),
                message: e.to_string(),
            })),
        ),
    }
}

// ─── AI ──────────────────────────────────────────────────────────

async fn generate_ai_report(
    State(svc): State<VehicleService>,
    Json(req): Json<QueryRequest>,
) -> impl IntoResponse {
    match svc.query_vehicle(&req.plate, &req.locale).await {
        Ok(report) => (StatusCode::OK, Json(json!(report))),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!(ErrorResponse {
                error: "generation_failed".into(),
                message: e.to_string(),
            })),
        ),
    }
}

// ─── User (placeholders) ────────────────────────────────────────

async fn get_credits() -> impl IntoResponse {
    Json(json!({
        "user_id": 0,
        "plan": "free",
        "credits": 3
    }))
}

async fn consume_credits() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "remaining": 2
    }))
}

async fn get_history() -> impl IntoResponse {
    Json(json!({
        "history": []
    }))
}
