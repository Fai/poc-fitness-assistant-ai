//! Biomarkers API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services::biomarkers::{BiomarkersService, CreateSupplementInput, LogBiomarkerInput};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use fitness_assistant_shared::types::{
    BiomarkerHistoryQuery, BiomarkerLogResponse, BiomarkerRangeResponse, CreateSupplementRequest,
    LogBiomarkerRequest, LogSupplementRequest, SupplementAdherenceQuery,
    SupplementAdherenceResponse, SupplementResponse, SupplementsListQuery,
};

/// Create biomarkers routes
pub fn biomarkers_routes() -> Router<AppState> {
    Router::new()
        .route("/ranges", get(get_ranges))
        .route("/", post(log_biomarker))
        .route("/history", get(get_history))
        .route("/:id", axum::routing::delete(delete_biomarker_log))
        .route("/supplements", post(create_supplement).get(list_supplements))
        .route("/supplements/log", post(log_supplement))
        .route("/supplements/:id", axum::routing::delete(delete_supplement))
        .route("/supplements/:id/adherence", get(get_adherence))
}

/// GET /api/v1/biomarkers/ranges - Get all biomarker ranges
async fn get_ranges(
    State(state): State<AppState>,
) -> Result<Json<Vec<BiomarkerRangeResponse>>, ApiError> {
    let ranges = BiomarkersService::get_ranges(state.db()).await?;

    Ok(Json(
        ranges
            .into_iter()
            .map(|r| BiomarkerRangeResponse {
                id: r.id.to_string(),
                name: r.name,
                display_name: r.display_name,
                category: r.category,
                unit: r.unit,
                low_threshold: r.low_threshold,
                optimal_min: r.optimal_min,
                optimal_max: r.optimal_max,
                high_threshold: r.high_threshold,
                description: r.description,
            })
            .collect(),
    ))
}

/// POST /api/v1/biomarkers - Log a biomarker value
async fn log_biomarker(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogBiomarkerRequest>,
) -> Result<Json<BiomarkerLogResponse>, ApiError> {
    let input = LogBiomarkerInput {
        biomarker_name: req.biomarker_name,
        value: req.value,
        test_date: req.test_date,
        lab_name: req.lab_name,
        notes: req.notes,
        source: req.source,
    };

    let log = BiomarkersService::log_biomarker(state.db(), auth.user_id, input).await?;

    Ok(Json(BiomarkerLogResponse {
        id: log.id.to_string(),
        biomarker_name: log.biomarker_name,
        display_name: log.display_name,
        category: log.category,
        value: log.value,
        unit: log.unit,
        classification: log.classification,
        test_date: log.test_date,
        lab_name: log.lab_name,
        notes: log.notes,
    }))
}

/// GET /api/v1/biomarkers/history - Get biomarker history
async fn get_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<BiomarkerHistoryQuery>,
) -> Result<Json<Vec<BiomarkerLogResponse>>, ApiError> {
    let logs = BiomarkersService::get_biomarker_history(
        state.db(),
        auth.user_id,
        query.biomarker_name.as_deref(),
        query.limit.clamp(1, 100),
        query.offset.max(0),
    )
    .await?;

    Ok(Json(
        logs.into_iter()
            .map(|log| BiomarkerLogResponse {
                id: log.id.to_string(),
                biomarker_name: log.biomarker_name,
                display_name: log.display_name,
                category: log.category,
                value: log.value,
                unit: log.unit,
                classification: log.classification,
                test_date: log.test_date,
                lab_name: log.lab_name,
                notes: log.notes,
            })
            .collect(),
    ))
}

/// DELETE /api/v1/biomarkers/:id - Delete a biomarker log
async fn delete_biomarker_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let log_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid log ID".to_string()))?;

    let deleted = BiomarkersService::delete_biomarker_log(state.db(), auth.user_id, log_id).await?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(ApiError::NotFound("Biomarker log not found".to_string()))
    }
}

/// POST /api/v1/biomarkers/supplements - Create a supplement
async fn create_supplement(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateSupplementRequest>,
) -> Result<Json<SupplementResponse>, ApiError> {
    let input = CreateSupplementInput {
        name: req.name,
        brand: req.brand,
        dosage: req.dosage,
        frequency: req.frequency,
        time_of_day: req.time_of_day,
        start_date: req.start_date,
        end_date: req.end_date,
        notes: req.notes,
    };

    let supplement = BiomarkersService::create_supplement(state.db(), auth.user_id, input).await?;

    Ok(Json(SupplementResponse {
        id: supplement.id.to_string(),
        name: supplement.name,
        brand: supplement.brand,
        dosage: supplement.dosage,
        frequency: supplement.frequency,
        time_of_day: supplement.time_of_day,
        start_date: supplement.start_date,
        end_date: supplement.end_date,
        is_active: supplement.is_active,
        notes: supplement.notes,
    }))
}

/// GET /api/v1/biomarkers/supplements - List supplements
async fn list_supplements(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<SupplementsListQuery>,
) -> Result<Json<Vec<SupplementResponse>>, ApiError> {
    let supplements =
        BiomarkersService::get_supplements(state.db(), auth.user_id, query.active_only).await?;

    Ok(Json(
        supplements
            .into_iter()
            .map(|s| SupplementResponse {
                id: s.id.to_string(),
                name: s.name,
                brand: s.brand,
                dosage: s.dosage,
                frequency: s.frequency,
                time_of_day: s.time_of_day,
                start_date: s.start_date,
                end_date: s.end_date,
                is_active: s.is_active,
                notes: s.notes,
            })
            .collect(),
    ))
}

/// POST /api/v1/biomarkers/supplements/log - Log supplement intake
async fn log_supplement(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogSupplementRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let supplement_id = uuid::Uuid::parse_str(&req.supplement_id)
        .map_err(|_| ApiError::Validation("Invalid supplement ID".to_string()))?;

    BiomarkersService::log_supplement(state.db(), auth.user_id, supplement_id, req.skipped, req.notes)
        .await?;

    Ok(Json(serde_json::json!({"logged": true})))
}

/// DELETE /api/v1/biomarkers/supplements/:id - Delete a supplement
async fn delete_supplement(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let supplement_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid supplement ID".to_string()))?;

    let deleted =
        BiomarkersService::delete_supplement(state.db(), auth.user_id, supplement_id).await?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(ApiError::NotFound("Supplement not found".to_string()))
    }
}

/// GET /api/v1/biomarkers/supplements/:id/adherence - Get supplement adherence
async fn get_adherence(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Query(query): Query<SupplementAdherenceQuery>,
) -> Result<Json<SupplementAdherenceResponse>, ApiError> {
    let supplement_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid supplement ID".to_string()))?;

    let adherence = BiomarkersService::get_adherence(
        state.db(),
        auth.user_id,
        supplement_id,
        query.start_date,
        query.end_date,
    )
    .await?;

    Ok(Json(SupplementAdherenceResponse {
        supplement_id: adherence.supplement_id.to_string(),
        supplement_name: adherence.supplement_name,
        total_days: adherence.total_days,
        days_taken: adherence.days_taken,
        days_skipped: adherence.days_skipped,
        adherence_percent: adherence.adherence_percent,
    }))
}
