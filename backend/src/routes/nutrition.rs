//! Nutrition API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::repositories::FoodItemRepository;
use crate::services::NutritionService;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use fitness_assistant_shared::types::{
    AddIngredientRequest, CreateRecipeRequest, DailyNutritionResponse, DateQuery,
    FoodItemResponse, FoodLogResponse, FoodSearchQuery, LogFoodRequest, RecipeDetailResponse,
    RecipeIngredientResponse, RecipeResponse,
};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Create nutrition routes
pub fn nutrition_routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_foods))
        .route("/barcode/:code", get(lookup_barcode))
        .route("/log", post(log_food))
        .route("/log/:id", delete(delete_food_log))
        .route("/daily", get(get_daily_summary))
        .route("/recipes", post(create_recipe).get(list_recipes))
        .route("/recipes/:id", get(get_recipe).delete(delete_recipe))
        .route("/recipes/:id/ingredients", post(add_ingredient))
        .route("/recipes/:id/ingredients/:food_id", delete(remove_ingredient))
}

/// Helper to convert Decimal to f64
fn dec_to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

/// Helper to convert f64 to Decimal
fn f64_to_dec(f: f64) -> Decimal {
    Decimal::try_from(f).unwrap_or(Decimal::ZERO)
}

/// GET /api/v1/nutrition/search - Search food database
async fn search_foods(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<FoodSearchQuery>,
) -> Result<Json<Vec<FoodItemResponse>>, ApiError> {
    let items = NutritionService::search_foods(state.db(), &query.q, query.limit).await?;

    let response: Vec<FoodItemResponse> = items
        .into_iter()
        .map(|item| FoodItemResponse {
            id: item.id.to_string(),
            name: item.name,
            brand: item.brand,
            barcode: item.barcode,
            serving_size: dec_to_f64(item.serving_size),
            serving_unit: item.serving_unit,
            calories: dec_to_f64(item.calories),
            protein_g: dec_to_f64(item.protein_g),
            carbohydrates_g: dec_to_f64(item.carbohydrates_g),
            fat_g: dec_to_f64(item.fat_g),
            fiber_g: dec_to_f64(item.fiber_g),
            sugar_g: dec_to_f64(item.sugar_g),
            source: item.source,
            verified: item.verified,
        })
        .collect();

    Ok(Json(response))
}

/// GET /api/v1/nutrition/barcode/:code - Lookup food by barcode
async fn lookup_barcode(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<Option<FoodItemResponse>>, ApiError> {
    let item = NutritionService::lookup_barcode(state.db(), &code).await?;

    let response = item.map(|item| FoodItemResponse {
        id: item.id.to_string(),
        name: item.name,
        brand: item.brand,
        barcode: item.barcode,
        serving_size: dec_to_f64(item.serving_size),
        serving_unit: item.serving_unit,
        calories: dec_to_f64(item.calories),
        protein_g: dec_to_f64(item.protein_g),
        carbohydrates_g: dec_to_f64(item.carbohydrates_g),
        fat_g: dec_to_f64(item.fat_g),
        fiber_g: dec_to_f64(item.fiber_g),
        sugar_g: dec_to_f64(item.sugar_g),
        source: item.source,
        verified: item.verified,
    });

    Ok(Json(response))
}

/// POST /api/v1/nutrition/log - Log a food entry
async fn log_food(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogFoodRequest>,
) -> Result<Json<FoodLogResponse>, ApiError> {
    let food_item_id = req
        .food_item_id
        .map(|id| Uuid::parse_str(&id))
        .transpose()
        .map_err(|_| ApiError::Validation("Invalid food_item_id".to_string()))?;

    let log = NutritionService::log_food(
        state.db(),
        auth.user_id,
        food_item_id,
        None, // custom_name not supported yet
        f64_to_dec(req.servings),
        req.meal_type,
        req.consumed_at,
        req.notes,
    )
    .await?;

    // Get food name if we have a food_item_id
    let food_name = if let Some(item_id) = log.food_item_id {
        FoodItemRepository::find_by_id(state.db(), item_id)
            .await
            .ok()
            .flatten()
            .map(|item| item.name)
    } else {
        log.custom_name.clone()
    };

    Ok(Json(FoodLogResponse {
        id: log.id.to_string(),
        food_item_id: log.food_item_id.map(|id| id.to_string()),
        food_name,
        servings: dec_to_f64(log.servings),
        calories: dec_to_f64(log.calories),
        protein_g: dec_to_f64(log.protein_g),
        carbohydrates_g: dec_to_f64(log.carbohydrates_g),
        fat_g: dec_to_f64(log.fat_g),
        fiber_g: dec_to_f64(log.fiber_g),
        meal_type: log.meal_type,
        consumed_at: log.consumed_at,
        notes: log.notes,
    }))
}

/// DELETE /api/v1/nutrition/log/:id - Delete a food log entry
async fn delete_food_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<()>, ApiError> {
    let log_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid log ID".to_string()))?;

    NutritionService::delete_log(state.db(), auth.user_id, log_id).await?;

    Ok(Json(()))
}

/// GET /api/v1/nutrition/daily - Get daily nutrition summary
async fn get_daily_summary(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<DateQuery>,
) -> Result<Json<DailyNutritionResponse>, ApiError> {
    let summary = NutritionService::get_daily_summary(state.db(), auth.user_id, query.date).await?;
    let logs = NutritionService::get_logs_by_date(state.db(), auth.user_id, query.date).await?;

    let log_responses: Vec<FoodLogResponse> = logs
        .into_iter()
        .map(|log| FoodLogResponse {
            id: log.id.to_string(),
            food_item_id: log.food_item_id.map(|id| id.to_string()),
            food_name: log.custom_name,
            servings: dec_to_f64(log.servings),
            calories: dec_to_f64(log.calories),
            protein_g: dec_to_f64(log.protein_g),
            carbohydrates_g: dec_to_f64(log.carbohydrates_g),
            fat_g: dec_to_f64(log.fat_g),
            fiber_g: dec_to_f64(log.fiber_g),
            meal_type: log.meal_type,
            consumed_at: log.consumed_at,
            notes: log.notes,
        })
        .collect();

    Ok(Json(DailyNutritionResponse {
        date: summary.date,
        total_calories: dec_to_f64(summary.total_calories),
        total_protein_g: dec_to_f64(summary.total_protein_g),
        total_carbs_g: dec_to_f64(summary.total_carbs_g),
        total_fat_g: dec_to_f64(summary.total_fat_g),
        total_fiber_g: dec_to_f64(summary.total_fiber_g),
        meal_count: summary.meal_count,
        logs: log_responses,
    }))
}

/// POST /api/v1/nutrition/recipes - Create a new recipe
async fn create_recipe(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRecipeRequest>,
) -> Result<Json<RecipeResponse>, ApiError> {
    let recipe = NutritionService::create_recipe(
        state.db(),
        auth.user_id,
        req.name,
        req.description,
        f64_to_dec(req.servings),
        req.is_public,
    )
    .await?;

    // Add initial ingredients if provided
    for (idx, ing) in req.ingredients.into_iter().enumerate() {
        let food_item_id = Uuid::parse_str(&ing.food_item_id)
            .map_err(|_| ApiError::Validation("Invalid food_item_id".to_string()))?;

        NutritionService::add_recipe_ingredient(
            state.db(),
            auth.user_id,
            recipe.id,
            food_item_id,
            f64_to_dec(ing.servings),
            if ing.sort_order == 0 { idx as i32 } else { ing.sort_order },
        )
        .await?;
    }

    // Re-fetch recipe to get updated nutrition values
    let updated_recipe = NutritionService::get_recipe(state.db(), auth.user_id, recipe.id).await?;

    Ok(Json(RecipeResponse {
        id: updated_recipe.id.to_string(),
        name: updated_recipe.name,
        description: updated_recipe.description,
        servings: dec_to_f64(updated_recipe.servings),
        calories_per_serving: dec_to_f64(updated_recipe.calories_per_serving),
        protein_per_serving: dec_to_f64(updated_recipe.protein_per_serving),
        carbs_per_serving: dec_to_f64(updated_recipe.carbs_per_serving),
        fat_per_serving: dec_to_f64(updated_recipe.fat_per_serving),
        fiber_per_serving: dec_to_f64(updated_recipe.fiber_per_serving),
        is_public: updated_recipe.is_public,
        created_at: updated_recipe.created_at,
    }))
}

/// GET /api/v1/nutrition/recipes - List user's recipes
async fn list_recipes(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<RecipeResponse>>, ApiError> {
    let recipes = NutritionService::get_user_recipes(state.db(), auth.user_id).await?;

    let response: Vec<RecipeResponse> = recipes
        .into_iter()
        .map(|r| RecipeResponse {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            servings: dec_to_f64(r.servings),
            calories_per_serving: dec_to_f64(r.calories_per_serving),
            protein_per_serving: dec_to_f64(r.protein_per_serving),
            carbs_per_serving: dec_to_f64(r.carbs_per_serving),
            fat_per_serving: dec_to_f64(r.fat_per_serving),
            fiber_per_serving: dec_to_f64(r.fiber_per_serving),
            is_public: r.is_public,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(response))
}

/// GET /api/v1/nutrition/recipes/:id - Get recipe with ingredients
async fn get_recipe(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<RecipeDetailResponse>, ApiError> {
    let recipe_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid recipe ID".to_string()))?;

    let recipe = NutritionService::get_recipe(state.db(), auth.user_id, recipe_id).await?;
    let ingredients = NutritionService::get_recipe_ingredients(state.db(), auth.user_id, recipe_id).await?;

    // Get food names for ingredients
    let mut ingredient_responses = Vec::new();
    for ing in ingredients {
        let food_name = FoodItemRepository::find_by_id(state.db(), ing.food_item_id)
            .await
            .ok()
            .flatten()
            .map(|item| item.name)
            .unwrap_or_else(|| "Unknown".to_string());

        ingredient_responses.push(RecipeIngredientResponse {
            id: ing.id.to_string(),
            food_item_id: ing.food_item_id.to_string(),
            food_name,
            servings: dec_to_f64(ing.servings),
            sort_order: ing.sort_order,
        });
    }

    Ok(Json(RecipeDetailResponse {
        recipe: RecipeResponse {
            id: recipe.id.to_string(),
            name: recipe.name,
            description: recipe.description,
            servings: dec_to_f64(recipe.servings),
            calories_per_serving: dec_to_f64(recipe.calories_per_serving),
            protein_per_serving: dec_to_f64(recipe.protein_per_serving),
            carbs_per_serving: dec_to_f64(recipe.carbs_per_serving),
            fat_per_serving: dec_to_f64(recipe.fat_per_serving),
            fiber_per_serving: dec_to_f64(recipe.fiber_per_serving),
            is_public: recipe.is_public,
            created_at: recipe.created_at,
        },
        ingredients: ingredient_responses,
    }))
}

/// DELETE /api/v1/nutrition/recipes/:id - Delete a recipe
async fn delete_recipe(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<()>, ApiError> {
    let recipe_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid recipe ID".to_string()))?;

    NutritionService::delete_recipe(state.db(), auth.user_id, recipe_id).await?;

    Ok(Json(()))
}

/// POST /api/v1/nutrition/recipes/:id/ingredients - Add ingredient to recipe
async fn add_ingredient(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AddIngredientRequest>,
) -> Result<Json<RecipeIngredientResponse>, ApiError> {
    let recipe_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid recipe ID".to_string()))?;
    let food_item_id = Uuid::parse_str(&req.food_item_id)
        .map_err(|_| ApiError::Validation("Invalid food_item_id".to_string()))?;

    let ingredient = NutritionService::add_recipe_ingredient(
        state.db(),
        auth.user_id,
        recipe_id,
        food_item_id,
        f64_to_dec(req.servings),
        req.sort_order,
    )
    .await?;

    let food_name = FoodItemRepository::find_by_id(state.db(), food_item_id)
        .await
        .ok()
        .flatten()
        .map(|item| item.name)
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(Json(RecipeIngredientResponse {
        id: ingredient.id.to_string(),
        food_item_id: ingredient.food_item_id.to_string(),
        food_name,
        servings: dec_to_f64(ingredient.servings),
        sort_order: ingredient.sort_order,
    }))
}

/// DELETE /api/v1/nutrition/recipes/:id/ingredients/:food_id - Remove ingredient
async fn remove_ingredient(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, food_id)): Path<(String, String)>,
) -> Result<Json<()>, ApiError> {
    let recipe_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid recipe ID".to_string()))?;
    let food_item_id = Uuid::parse_str(&food_id)
        .map_err(|_| ApiError::Validation("Invalid food_item_id".to_string()))?;

    NutritionService::remove_recipe_ingredient(state.db(), auth.user_id, recipe_id, food_item_id)
        .await?;

    Ok(Json(()))
}
