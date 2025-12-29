//! Nutrition repository - database operations for food items and logs

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Food item from the database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FoodItem {
    pub id: Uuid,
    pub name: String,
    pub brand: Option<String>,
    pub barcode: Option<String>,
    pub serving_size: Decimal,
    pub serving_unit: String,
    pub calories: Decimal,
    pub protein_g: Decimal,
    pub carbohydrates_g: Decimal,
    pub fat_g: Decimal,
    pub fiber_g: Decimal,
    pub sugar_g: Decimal,
    pub sodium_mg: Option<Decimal>,
    pub potassium_mg: Option<Decimal>,
    pub cholesterol_mg: Option<Decimal>,
    pub source: String,
    pub verified: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Food log entry
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FoodLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub food_item_id: Option<Uuid>,
    pub custom_name: Option<String>,
    pub servings: Decimal,
    pub calories: Decimal,
    pub protein_g: Decimal,
    pub carbohydrates_g: Decimal,
    pub fat_g: Decimal,
    pub fiber_g: Decimal,
    pub meal_type: String,
    pub logged_at: DateTime<Utc>,
    pub consumed_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a new food item
#[derive(Debug, Clone)]
pub struct CreateFoodItem {
    pub name: String,
    pub brand: Option<String>,
    pub barcode: Option<String>,
    pub serving_size: Decimal,
    pub serving_unit: String,
    pub calories: Decimal,
    pub protein_g: Decimal,
    pub carbohydrates_g: Decimal,
    pub fat_g: Decimal,
    pub fiber_g: Decimal,
    pub sugar_g: Decimal,
    pub sodium_mg: Option<Decimal>,
    pub source: String,
    pub created_by: Option<Uuid>,
}

/// Input for logging food
#[derive(Debug, Clone)]
pub struct CreateFoodLog {
    pub user_id: Uuid,
    pub food_item_id: Option<Uuid>,
    pub custom_name: Option<String>,
    pub servings: Decimal,
    pub calories: Decimal,
    pub protein_g: Decimal,
    pub carbohydrates_g: Decimal,
    pub fat_g: Decimal,
    pub fiber_g: Decimal,
    pub meal_type: String,
    pub consumed_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Food item repository
pub struct FoodItemRepository;

impl FoodItemRepository {
    /// Search food items using full-text search
    pub async fn search(db: &PgPool, query: &str, limit: i64) -> Result<Vec<FoodItem>> {
        let items = sqlx::query_as::<_, FoodItem>(
            r#"
            SELECT id, name, brand, barcode, serving_size, serving_unit,
                   calories, protein_g, carbohydrates_g, fat_g, fiber_g, sugar_g,
                   sodium_mg, potassium_mg, cholesterol_mg, source, verified,
                   created_by, created_at, updated_at
            FROM food_items
            WHERE to_tsvector('english', name || ' ' || COALESCE(brand, '')) 
                  @@ plainto_tsquery('english', $1)
               OR LOWER(name) LIKE LOWER($2)
            ORDER BY 
                CASE WHEN verified THEN 0 ELSE 1 END,
                ts_rank(to_tsvector('english', name || ' ' || COALESCE(brand, '')), 
                        plainto_tsquery('english', $1)) DESC
            LIMIT $3
            "#,
        )
        .bind(query)
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(db)
        .await?;

        Ok(items)
    }

    /// Find food item by barcode
    pub async fn find_by_barcode(db: &PgPool, barcode: &str) -> Result<Option<FoodItem>> {
        let item = sqlx::query_as::<_, FoodItem>(
            r#"
            SELECT id, name, brand, barcode, serving_size, serving_unit,
                   calories, protein_g, carbohydrates_g, fat_g, fiber_g, sugar_g,
                   sodium_mg, potassium_mg, cholesterol_mg, source, verified,
                   created_by, created_at, updated_at
            FROM food_items
            WHERE barcode = $1
            "#,
        )
        .bind(barcode)
        .fetch_optional(db)
        .await?;

        Ok(item)
    }


    /// Find food item by ID
    pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<FoodItem>> {
        let item = sqlx::query_as::<_, FoodItem>(
            r#"
            SELECT id, name, brand, barcode, serving_size, serving_unit,
                   calories, protein_g, carbohydrates_g, fat_g, fiber_g, sugar_g,
                   sodium_mg, potassium_mg, cholesterol_mg, source, verified,
                   created_by, created_at, updated_at
            FROM food_items
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?;

        Ok(item)
    }

    /// Create a new food item
    pub async fn create(db: &PgPool, input: CreateFoodItem) -> Result<FoodItem> {
        let item = sqlx::query_as::<_, FoodItem>(
            r#"
            INSERT INTO food_items (
                name, brand, barcode, serving_size, serving_unit,
                calories, protein_g, carbohydrates_g, fat_g, fiber_g, sugar_g,
                sodium_mg, source, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, name, brand, barcode, serving_size, serving_unit,
                      calories, protein_g, carbohydrates_g, fat_g, fiber_g, sugar_g,
                      sodium_mg, potassium_mg, cholesterol_mg, source, verified,
                      created_by, created_at, updated_at
            "#,
        )
        .bind(&input.name)
        .bind(&input.brand)
        .bind(&input.barcode)
        .bind(input.serving_size)
        .bind(&input.serving_unit)
        .bind(input.calories)
        .bind(input.protein_g)
        .bind(input.carbohydrates_g)
        .bind(input.fat_g)
        .bind(input.fiber_g)
        .bind(input.sugar_g)
        .bind(input.sodium_mg)
        .bind(&input.source)
        .bind(input.created_by)
        .fetch_one(db)
        .await?;

        Ok(item)
    }
}

/// Food log repository
pub struct FoodLogRepository;

impl FoodLogRepository {
    /// Log a food entry
    pub async fn create(db: &PgPool, input: CreateFoodLog) -> Result<FoodLog> {
        let log = sqlx::query_as::<_, FoodLog>(
            r#"
            INSERT INTO food_logs (
                user_id, food_item_id, custom_name, servings,
                calories, protein_g, carbohydrates_g, fat_g, fiber_g,
                meal_type, consumed_at, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, user_id, food_item_id, custom_name, servings,
                      calories, protein_g, carbohydrates_g, fat_g, fiber_g,
                      meal_type, logged_at, consumed_at, notes, created_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.food_item_id)
        .bind(&input.custom_name)
        .bind(input.servings)
        .bind(input.calories)
        .bind(input.protein_g)
        .bind(input.carbohydrates_g)
        .bind(input.fat_g)
        .bind(input.fiber_g)
        .bind(&input.meal_type)
        .bind(input.consumed_at)
        .bind(&input.notes)
        .fetch_one(db)
        .await?;

        Ok(log)
    }

    /// Get food logs for a user on a specific date
    pub async fn get_by_date(db: &PgPool, user_id: Uuid, date: NaiveDate) -> Result<Vec<FoodLog>> {
        let logs = sqlx::query_as::<_, FoodLog>(
            r#"
            SELECT id, user_id, food_item_id, custom_name, servings,
                   calories, protein_g, carbohydrates_g, fat_g, fiber_g,
                   meal_type, logged_at, consumed_at, notes, created_at
            FROM food_logs
            WHERE user_id = $1 AND DATE(consumed_at) = $2
            ORDER BY consumed_at ASC
            "#,
        )
        .bind(user_id)
        .bind(date)
        .fetch_all(db)
        .await?;

        Ok(logs)
    }

    /// Get food logs for a date range
    pub async fn get_by_date_range(
        db: &PgPool,
        user_id: Uuid,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<FoodLog>> {
        let logs = sqlx::query_as::<_, FoodLog>(
            r#"
            SELECT id, user_id, food_item_id, custom_name, servings,
                   calories, protein_g, carbohydrates_g, fat_g, fiber_g,
                   meal_type, logged_at, consumed_at, notes, created_at
            FROM food_logs
            WHERE user_id = $1 
              AND DATE(consumed_at) >= $2 
              AND DATE(consumed_at) <= $3
            ORDER BY consumed_at ASC
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(db)
        .await?;

        Ok(logs)
    }

    /// Delete a food log entry
    pub async fn delete(db: &PgPool, user_id: Uuid, log_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM food_logs WHERE id = $1 AND user_id = $2"
        )
        .bind(log_id)
        .bind(user_id)
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Daily nutrition summary
#[derive(Debug, Clone)]
pub struct DailyNutritionSummary {
    pub date: NaiveDate,
    pub total_calories: Decimal,
    pub total_protein_g: Decimal,
    pub total_carbs_g: Decimal,
    pub total_fat_g: Decimal,
    pub total_fiber_g: Decimal,
    pub meal_count: i64,
}

impl FoodLogRepository {
    /// Get daily nutrition summary
    pub async fn get_daily_summary(
        db: &PgPool,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<DailyNutritionSummary> {
        let row = sqlx::query_as::<_, (Decimal, Decimal, Decimal, Decimal, Decimal, i64)>(
            r#"
            SELECT 
                COALESCE(SUM(calories), 0) as total_calories,
                COALESCE(SUM(protein_g), 0) as total_protein,
                COALESCE(SUM(carbohydrates_g), 0) as total_carbs,
                COALESCE(SUM(fat_g), 0) as total_fat,
                COALESCE(SUM(fiber_g), 0) as total_fiber,
                COUNT(*) as meal_count
            FROM food_logs
            WHERE user_id = $1 AND DATE(consumed_at) = $2
            "#,
        )
        .bind(user_id)
        .bind(date)
        .fetch_one(db)
        .await?;

        Ok(DailyNutritionSummary {
            date,
            total_calories: row.0,
            total_protein_g: row.1,
            total_carbs_g: row.2,
            total_fat_g: row.3,
            total_fiber_g: row.4,
            meal_count: row.5,
        })
    }
}

/// Recipe from the database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Recipe {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub servings: Decimal,
    pub calories_per_serving: Decimal,
    pub protein_per_serving: Decimal,
    pub carbs_per_serving: Decimal,
    pub fat_per_serving: Decimal,
    pub fiber_per_serving: Decimal,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recipe ingredient from the database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecipeIngredient {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub food_item_id: Uuid,
    pub servings: Decimal,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a new recipe
#[derive(Debug, Clone)]
pub struct CreateRecipe {
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub servings: Decimal,
    pub is_public: bool,
}

/// Input for adding an ingredient to a recipe
#[derive(Debug, Clone)]
pub struct AddRecipeIngredient {
    pub recipe_id: Uuid,
    pub food_item_id: Uuid,
    pub servings: Decimal,
    pub sort_order: i32,
}

/// Recipe repository
pub struct RecipeRepository;

impl RecipeRepository {
    /// Create a new recipe
    pub async fn create(db: &PgPool, input: CreateRecipe) -> Result<Recipe> {
        let recipe = sqlx::query_as::<_, Recipe>(
            r#"
            INSERT INTO recipes (user_id, name, description, servings, is_public)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, name, description, servings,
                      calories_per_serving, protein_per_serving, carbs_per_serving,
                      fat_per_serving, fiber_per_serving, is_public, created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.servings)
        .bind(input.is_public)
        .fetch_one(db)
        .await?;

        Ok(recipe)
    }

    /// Find recipe by ID
    pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<Recipe>> {
        let recipe = sqlx::query_as::<_, Recipe>(
            r#"
            SELECT id, user_id, name, description, servings,
                   calories_per_serving, protein_per_serving, carbs_per_serving,
                   fat_per_serving, fiber_per_serving, is_public, created_at, updated_at
            FROM recipes
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?;

        Ok(recipe)
    }

    /// Find recipe by ID and user (for ownership check)
    pub async fn find_by_id_and_user(db: &PgPool, id: Uuid, user_id: Uuid) -> Result<Option<Recipe>> {
        let recipe = sqlx::query_as::<_, Recipe>(
            r#"
            SELECT id, user_id, name, description, servings,
                   calories_per_serving, protein_per_serving, carbs_per_serving,
                   fat_per_serving, fiber_per_serving, is_public, created_at, updated_at
            FROM recipes
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        Ok(recipe)
    }

    /// Get all recipes for a user
    pub async fn get_by_user(db: &PgPool, user_id: Uuid) -> Result<Vec<Recipe>> {
        let recipes = sqlx::query_as::<_, Recipe>(
            r#"
            SELECT id, user_id, name, description, servings,
                   calories_per_serving, protein_per_serving, carbs_per_serving,
                   fat_per_serving, fiber_per_serving, is_public, created_at, updated_at
            FROM recipes
            WHERE user_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(db)
        .await?;

        Ok(recipes)
    }

    /// Add ingredient to recipe
    pub async fn add_ingredient(db: &PgPool, input: AddRecipeIngredient) -> Result<RecipeIngredient> {
        let ingredient = sqlx::query_as::<_, RecipeIngredient>(
            r#"
            INSERT INTO recipe_ingredients (recipe_id, food_item_id, servings, sort_order)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (recipe_id, food_item_id) 
            DO UPDATE SET servings = EXCLUDED.servings, sort_order = EXCLUDED.sort_order
            RETURNING id, recipe_id, food_item_id, servings, sort_order, created_at
            "#,
        )
        .bind(input.recipe_id)
        .bind(input.food_item_id)
        .bind(input.servings)
        .bind(input.sort_order)
        .fetch_one(db)
        .await?;

        Ok(ingredient)
    }

    /// Get ingredients for a recipe
    pub async fn get_ingredients(db: &PgPool, recipe_id: Uuid) -> Result<Vec<RecipeIngredient>> {
        let ingredients = sqlx::query_as::<_, RecipeIngredient>(
            r#"
            SELECT id, recipe_id, food_item_id, servings, sort_order, created_at
            FROM recipe_ingredients
            WHERE recipe_id = $1
            ORDER BY sort_order ASC
            "#,
        )
        .bind(recipe_id)
        .fetch_all(db)
        .await?;

        Ok(ingredients)
    }

    /// Remove ingredient from recipe
    pub async fn remove_ingredient(db: &PgPool, recipe_id: Uuid, food_item_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM recipe_ingredients WHERE recipe_id = $1 AND food_item_id = $2"
        )
        .bind(recipe_id)
        .bind(food_item_id)
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a recipe
    pub async fn delete(db: &PgPool, user_id: Uuid, recipe_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM recipes WHERE id = $1 AND user_id = $2"
        )
        .bind(recipe_id)
        .bind(user_id)
        .execute(db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
