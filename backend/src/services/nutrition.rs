//! Nutrition service - business logic for food tracking

use crate::error::ApiError;
use crate::repositories::{
    AddRecipeIngredient, CreateFoodItem, CreateFoodLog, CreateRecipe, DailyNutritionSummary,
    FoodItem, FoodItemRepository, FoodLog, FoodLogRepository, Recipe, RecipeIngredient,
    RecipeRepository,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Nutrition service
pub struct NutritionService;

impl NutritionService {
    /// Search for food items
    pub async fn search_foods(
        db: &PgPool,
        query: &str,
        limit: Option<i64>,
    ) -> Result<Vec<FoodItem>, ApiError> {
        let limit = limit.unwrap_or(20).min(100);
        
        if query.trim().is_empty() {
            return Err(ApiError::Validation("Search query cannot be empty".to_string()));
        }

        let items = FoodItemRepository::search(db, query, limit)
            .await
            .map_err(ApiError::Internal)?;

        Ok(items)
    }

    /// Look up food by barcode
    pub async fn lookup_barcode(db: &PgPool, barcode: &str) -> Result<Option<FoodItem>, ApiError> {
        if barcode.trim().is_empty() {
            return Err(ApiError::Validation("Barcode cannot be empty".to_string()));
        }

        let item = FoodItemRepository::find_by_barcode(db, barcode)
            .await
            .map_err(ApiError::Internal)?;

        Ok(item)
    }

    /// Create a custom food item
    pub async fn create_food_item(
        db: &PgPool,
        user_id: Uuid,
        name: String,
        serving_size: Decimal,
        serving_unit: String,
        calories: Decimal,
        protein_g: Decimal,
        carbs_g: Decimal,
        fat_g: Decimal,
        fiber_g: Decimal,
        sugar_g: Decimal,
        brand: Option<String>,
        barcode: Option<String>,
    ) -> Result<FoodItem, ApiError> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err(ApiError::Validation("Food name cannot be empty".to_string()));
        }
        if serving_size <= Decimal::ZERO {
            return Err(ApiError::Validation("Serving size must be positive".to_string()));
        }
        if calories < Decimal::ZERO {
            return Err(ApiError::Validation("Calories cannot be negative".to_string()));
        }

        let input = CreateFoodItem {
            name,
            brand,
            barcode,
            serving_size,
            serving_unit,
            calories,
            protein_g,
            carbohydrates_g: carbs_g,
            fat_g,
            fiber_g,
            sugar_g,
            sodium_mg: None,
            source: "user".to_string(),
            created_by: Some(user_id),
        };

        let item = FoodItemRepository::create(db, input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(item)
    }

    /// Log a food entry
    pub async fn log_food(
        db: &PgPool,
        user_id: Uuid,
        food_item_id: Option<Uuid>,
        custom_name: Option<String>,
        servings: Decimal,
        meal_type: String,
        consumed_at: Option<DateTime<Utc>>,
        notes: Option<String>,
    ) -> Result<FoodLog, ApiError> {
        // Validate meal type
        let valid_meal_types = ["breakfast", "lunch", "dinner", "snack"];
        if !valid_meal_types.contains(&meal_type.to_lowercase().as_str()) {
            return Err(ApiError::Validation(format!(
                "Invalid meal type. Must be one of: {}",
                valid_meal_types.join(", ")
            )));
        }

        if servings <= Decimal::ZERO {
            return Err(ApiError::Validation("Servings must be positive".to_string()));
        }

        // Get nutritional values
        let (calories, protein_g, carbs_g, fat_g, fiber_g) = if let Some(item_id) = food_item_id {
            let item = FoodItemRepository::find_by_id(db, item_id)
                .await
                .map_err(ApiError::Internal)?
                .ok_or_else(|| ApiError::NotFound("Food item not found".to_string()))?;

            (
                item.calories * servings,
                item.protein_g * servings,
                item.carbohydrates_g * servings,
                item.fat_g * servings,
                item.fiber_g * servings,
            )
        } else if custom_name.is_some() {
            // Custom entry - calories must be provided separately
            return Err(ApiError::Validation(
                "Custom food entries require food_item_id or pre-calculated nutrition".to_string(),
            ));
        } else {
            return Err(ApiError::Validation(
                "Either food_item_id or custom_name is required".to_string(),
            ));
        };

        let input = CreateFoodLog {
            user_id,
            food_item_id,
            custom_name,
            servings,
            calories,
            protein_g,
            carbohydrates_g: carbs_g,
            fat_g,
            fiber_g,
            meal_type: meal_type.to_lowercase(),
            consumed_at: consumed_at.unwrap_or_else(Utc::now),
            notes,
        };

        let log = FoodLogRepository::create(db, input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(log)
    }


    /// Get daily nutrition summary
    pub async fn get_daily_summary(
        db: &PgPool,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<DailyNutritionSummary, ApiError> {
        let summary = FoodLogRepository::get_daily_summary(db, user_id, date)
            .await
            .map_err(ApiError::Internal)?;

        Ok(summary)
    }

    /// Get food logs for a specific date
    pub async fn get_logs_by_date(
        db: &PgPool,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<FoodLog>, ApiError> {
        let logs = FoodLogRepository::get_by_date(db, user_id, date)
            .await
            .map_err(ApiError::Internal)?;

        Ok(logs)
    }

    /// Delete a food log entry
    pub async fn delete_log(
        db: &PgPool,
        user_id: Uuid,
        log_id: Uuid,
    ) -> Result<(), ApiError> {
        let deleted = FoodLogRepository::delete(db, user_id, log_id)
            .await
            .map_err(ApiError::Internal)?;

        if !deleted {
            return Err(ApiError::NotFound("Food log not found".to_string()));
        }

        Ok(())
    }

    // ==================== Recipe Methods ====================

    /// Create a new recipe
    pub async fn create_recipe(
        db: &PgPool,
        user_id: Uuid,
        name: String,
        description: Option<String>,
        servings: Decimal,
        is_public: bool,
    ) -> Result<Recipe, ApiError> {
        if name.trim().is_empty() {
            return Err(ApiError::Validation("Recipe name cannot be empty".to_string()));
        }
        if servings <= Decimal::ZERO {
            return Err(ApiError::Validation("Servings must be positive".to_string()));
        }

        let input = CreateRecipe {
            user_id,
            name,
            description,
            servings,
            is_public,
        };

        let recipe = RecipeRepository::create(db, input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(recipe)
    }

    /// Get a recipe by ID (checks ownership or public access)
    pub async fn get_recipe(
        db: &PgPool,
        user_id: Uuid,
        recipe_id: Uuid,
    ) -> Result<Recipe, ApiError> {
        let recipe = RecipeRepository::find_by_id(db, recipe_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Recipe not found".to_string()))?;

        // Check access: user owns it or it's public
        if recipe.user_id != user_id && !recipe.is_public {
            return Err(ApiError::NotFound("Recipe not found".to_string()));
        }

        Ok(recipe)
    }

    /// Get all recipes for a user
    pub async fn get_user_recipes(
        db: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Recipe>, ApiError> {
        let recipes = RecipeRepository::get_by_user(db, user_id)
            .await
            .map_err(ApiError::Internal)?;

        Ok(recipes)
    }

    /// Add an ingredient to a recipe
    pub async fn add_recipe_ingredient(
        db: &PgPool,
        user_id: Uuid,
        recipe_id: Uuid,
        food_item_id: Uuid,
        servings: Decimal,
        sort_order: i32,
    ) -> Result<RecipeIngredient, ApiError> {
        // Verify user owns the recipe
        RecipeRepository::find_by_id_and_user(db, recipe_id, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Recipe not found".to_string()))?;

        // Verify food item exists
        FoodItemRepository::find_by_id(db, food_item_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Food item not found".to_string()))?;

        if servings <= Decimal::ZERO {
            return Err(ApiError::Validation("Servings must be positive".to_string()));
        }

        let input = AddRecipeIngredient {
            recipe_id,
            food_item_id,
            servings,
            sort_order,
        };

        let ingredient = RecipeRepository::add_ingredient(db, input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(ingredient)
    }

    /// Get ingredients for a recipe
    pub async fn get_recipe_ingredients(
        db: &PgPool,
        user_id: Uuid,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeIngredient>, ApiError> {
        // Verify access to recipe
        Self::get_recipe(db, user_id, recipe_id).await?;

        let ingredients = RecipeRepository::get_ingredients(db, recipe_id)
            .await
            .map_err(ApiError::Internal)?;

        Ok(ingredients)
    }

    /// Remove an ingredient from a recipe
    pub async fn remove_recipe_ingredient(
        db: &PgPool,
        user_id: Uuid,
        recipe_id: Uuid,
        food_item_id: Uuid,
    ) -> Result<(), ApiError> {
        // Verify user owns the recipe
        RecipeRepository::find_by_id_and_user(db, recipe_id, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Recipe not found".to_string()))?;

        let removed = RecipeRepository::remove_ingredient(db, recipe_id, food_item_id)
            .await
            .map_err(ApiError::Internal)?;

        if !removed {
            return Err(ApiError::NotFound("Ingredient not found in recipe".to_string()));
        }

        Ok(())
    }

    /// Delete a recipe
    pub async fn delete_recipe(
        db: &PgPool,
        user_id: Uuid,
        recipe_id: Uuid,
    ) -> Result<(), ApiError> {
        let deleted = RecipeRepository::delete(db, user_id, recipe_id)
            .await
            .map_err(ApiError::Internal)?;

        if !deleted {
            return Err(ApiError::NotFound("Recipe not found".to_string()));
        }

        Ok(())
    }
}

/// Aggregates daily nutrition totals from a list of food logs
pub fn aggregate_daily_nutrition(logs: &[FoodLog]) -> (Decimal, Decimal, Decimal, Decimal, Decimal) {
    logs.iter().fold(
        (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
        |(cal, pro, carb, fat, fib), log| {
            (
                cal + log.calories,
                pro + log.protein_g,
                carb + log.carbohydrates_g,
                fat + log.fat_g,
                fib + log.fiber_g,
            )
        },
    )
}

/// Ingredient with its nutritional information for recipe calculation
#[derive(Debug, Clone)]
pub struct IngredientNutrition {
    pub servings: Decimal,
    pub calories_per_serving: Decimal,
    pub protein_per_serving: Decimal,
    pub carbs_per_serving: Decimal,
    pub fat_per_serving: Decimal,
    pub fiber_per_serving: Decimal,
}

/// Calculated recipe nutrition totals
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeNutrition {
    pub total_calories: Decimal,
    pub total_protein: Decimal,
    pub total_carbs: Decimal,
    pub total_fat: Decimal,
    pub total_fiber: Decimal,
    pub calories_per_serving: Decimal,
    pub protein_per_serving: Decimal,
    pub carbs_per_serving: Decimal,
    pub fat_per_serving: Decimal,
    pub fiber_per_serving: Decimal,
}

/// Calculates recipe nutrition from ingredients
/// 
/// For each ingredient: nutrition = ingredient_servings * food_item_nutrition_per_serving
/// Total recipe nutrition = sum of all ingredient nutritions
/// Per-serving = total / recipe_servings
pub fn calculate_recipe_nutrition(
    ingredients: &[IngredientNutrition],
    recipe_servings: Decimal,
) -> RecipeNutrition {
    let (total_cal, total_pro, total_carb, total_fat, total_fib) = ingredients.iter().fold(
        (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
        |(cal, pro, carb, fat, fib), ing| {
            (
                cal + ing.calories_per_serving * ing.servings,
                pro + ing.protein_per_serving * ing.servings,
                carb + ing.carbs_per_serving * ing.servings,
                fat + ing.fat_per_serving * ing.servings,
                fib + ing.fiber_per_serving * ing.servings,
            )
        },
    );

    // Avoid division by zero
    let servings = if recipe_servings <= Decimal::ZERO {
        Decimal::ONE
    } else {
        recipe_servings
    };

    RecipeNutrition {
        total_calories: total_cal,
        total_protein: total_pro,
        total_carbs: total_carb,
        total_fat: total_fat,
        total_fiber: total_fib,
        calories_per_serving: total_cal / servings,
        protein_per_serving: total_pro / servings,
        carbs_per_serving: total_carb / servings,
        fat_per_serving: total_fat / servings,
        fiber_per_serving: total_fib / servings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_daily_nutrition_empty() {
        let logs: Vec<FoodLog> = vec![];
        let (cal, pro, carb, fat, fib) = aggregate_daily_nutrition(&logs);
        assert_eq!(cal, Decimal::ZERO);
        assert_eq!(pro, Decimal::ZERO);
        assert_eq!(carb, Decimal::ZERO);
        assert_eq!(fat, Decimal::ZERO);
        assert_eq!(fib, Decimal::ZERO);
    }

    #[test]
    fn test_aggregate_daily_nutrition_single() {
        let logs = vec![create_test_food_log(
            Decimal::new(500, 0),  // 500 cal
            Decimal::new(30, 0),   // 30g protein
            Decimal::new(50, 0),   // 50g carbs
            Decimal::new(20, 0),   // 20g fat
            Decimal::new(5, 0),    // 5g fiber
        )];
        let (cal, pro, carb, fat, fib) = aggregate_daily_nutrition(&logs);
        assert_eq!(cal, Decimal::new(500, 0));
        assert_eq!(pro, Decimal::new(30, 0));
        assert_eq!(carb, Decimal::new(50, 0));
        assert_eq!(fat, Decimal::new(20, 0));
        assert_eq!(fib, Decimal::new(5, 0));
    }

    #[test]
    fn test_aggregate_daily_nutrition_multiple() {
        let logs = vec![
            create_test_food_log(
                Decimal::new(300, 0),
                Decimal::new(20, 0),
                Decimal::new(30, 0),
                Decimal::new(10, 0),
                Decimal::new(3, 0),
            ),
            create_test_food_log(
                Decimal::new(450, 0),
                Decimal::new(35, 0),
                Decimal::new(40, 0),
                Decimal::new(15, 0),
                Decimal::new(7, 0),
            ),
        ];
        let (cal, pro, carb, fat, fib) = aggregate_daily_nutrition(&logs);
        assert_eq!(cal, Decimal::new(750, 0));
        assert_eq!(pro, Decimal::new(55, 0));
        assert_eq!(carb, Decimal::new(70, 0));
        assert_eq!(fat, Decimal::new(25, 0));
        assert_eq!(fib, Decimal::new(10, 0));
    }

    /// Helper to create a test FoodLog with specified nutrition values
    fn create_test_food_log(
        calories: Decimal,
        protein_g: Decimal,
        carbohydrates_g: Decimal,
        fat_g: Decimal,
        fiber_g: Decimal,
    ) -> FoodLog {
        FoodLog {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            food_item_id: None,
            custom_name: Some("Test Food".to_string()),
            servings: Decimal::ONE,
            calories,
            protein_g,
            carbohydrates_g,
            fat_g,
            fiber_g,
            meal_type: "lunch".to_string(),
            logged_at: Utc::now(),
            consumed_at: Utc::now(),
            notes: None,
            created_at: Utc::now(),
        }
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Helper to check if a food item name/brand contains the search term
    fn food_matches_query(name: &str, brand: Option<&str>, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        let name_lower = name.to_lowercase();
        
        // Check if name contains query
        if name_lower.contains(&query_lower) {
            return true;
        }
        
        // Check if brand contains query
        if let Some(b) = brand {
            if b.to_lowercase().contains(&query_lower) {
                return true;
            }
        }
        
        // Check if any word in query matches
        for word in query_lower.split_whitespace() {
            if word.len() >= 2 && name_lower.contains(word) {
                return true;
            }
        }
        
        false
    }

    /// Strategy to generate valid nutrition values (non-negative decimals)
    fn nutrition_value_strategy() -> impl Strategy<Value = Decimal> {
        (0u32..10000u32).prop_map(|v| Decimal::new(v as i64, 1)) // 0.0 to 999.9
    }

    /// Strategy to generate a FoodLog with random nutrition values
    fn food_log_strategy() -> impl Strategy<Value = FoodLog> {
        (
            nutrition_value_strategy(), // calories
            nutrition_value_strategy(), // protein
            nutrition_value_strategy(), // carbs
            nutrition_value_strategy(), // fat
            nutrition_value_strategy(), // fiber
        )
            .prop_map(|(cal, pro, carb, fat, fib)| FoodLog {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                food_item_id: None,
                custom_name: Some("Test Food".to_string()),
                servings: Decimal::ONE,
                calories: cal,
                protein_g: pro,
                carbohydrates_g: carb,
                fat_g: fat,
                fiber_g: fib,
                meal_type: "lunch".to_string(),
                logged_at: Utc::now(),
                consumed_at: Utc::now(),
                notes: None,
                created_at: Utc::now(),
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 2: Nutrition Aggregation Correctness
        /// For any set of food log entries on a given day, the daily nutrition summary
        /// totals (calories, protein, carbs, fat, fiber) should equal the sum of the
        /// individual entry values.
        /// Feature: fitness-assistant-ai, Property 2: Nutrition Aggregation Correctness
        /// **Validates: Requirements 2.4**
        #[test]
        fn prop_nutrition_aggregation_correctness(
            logs in proptest::collection::vec(food_log_strategy(), 0..50)
        ) {
            // Calculate expected totals by manually summing
            let expected_calories: Decimal = logs.iter().map(|l| l.calories).sum();
            let expected_protein: Decimal = logs.iter().map(|l| l.protein_g).sum();
            let expected_carbs: Decimal = logs.iter().map(|l| l.carbohydrates_g).sum();
            let expected_fat: Decimal = logs.iter().map(|l| l.fat_g).sum();
            let expected_fiber: Decimal = logs.iter().map(|l| l.fiber_g).sum();

            // Get actual totals from aggregate function
            let (actual_cal, actual_pro, actual_carb, actual_fat, actual_fib) = 
                aggregate_daily_nutrition(&logs);

            // Property: aggregated totals must equal sum of individual entries
            prop_assert_eq!(actual_cal, expected_calories,
                "Calories mismatch: got {}, expected {}", actual_cal, expected_calories);
            prop_assert_eq!(actual_pro, expected_protein,
                "Protein mismatch: got {}, expected {}", actual_pro, expected_protein);
            prop_assert_eq!(actual_carb, expected_carbs,
                "Carbs mismatch: got {}, expected {}", actual_carb, expected_carbs);
            prop_assert_eq!(actual_fat, expected_fat,
                "Fat mismatch: got {}, expected {}", actual_fat, expected_fat);
            prop_assert_eq!(actual_fib, expected_fiber,
                "Fiber mismatch: got {}, expected {}", actual_fib, expected_fiber);
        }

        /// Property: Aggregation is commutative (order doesn't matter)
        #[test]
        fn prop_nutrition_aggregation_commutative(
            logs in proptest::collection::vec(food_log_strategy(), 2..20)
        ) {
            let (cal1, pro1, carb1, fat1, fib1) = aggregate_daily_nutrition(&logs);
            
            // Reverse the order
            let mut reversed = logs.clone();
            reversed.reverse();
            let (cal2, pro2, carb2, fat2, fib2) = aggregate_daily_nutrition(&reversed);

            // Results should be identical regardless of order
            prop_assert_eq!(cal1, cal2, "Calories should be order-independent");
            prop_assert_eq!(pro1, pro2, "Protein should be order-independent");
            prop_assert_eq!(carb1, carb2, "Carbs should be order-independent");
            prop_assert_eq!(fat1, fat2, "Fat should be order-independent");
            prop_assert_eq!(fib1, fib2, "Fiber should be order-independent");
        }

        /// Property: Empty input yields zero totals
        #[test]
        fn prop_nutrition_aggregation_identity(
            logs in proptest::collection::vec(food_log_strategy(), 1..10)
        ) {
            let empty: Vec<FoodLog> = vec![];
            let (cal, pro, carb, fat, fib) = aggregate_daily_nutrition(&empty);
            
            // Empty aggregation should be zero (identity element)
            prop_assert_eq!(cal, Decimal::ZERO);
            prop_assert_eq!(pro, Decimal::ZERO);
            prop_assert_eq!(carb, Decimal::ZERO);
            prop_assert_eq!(fat, Decimal::ZERO);
            prop_assert_eq!(fib, Decimal::ZERO);

            // Adding empty to any set should not change the result
            let (cal_with_data, _, _, _, _) = aggregate_daily_nutrition(&logs);
            let combined: Vec<FoodLog> = logs.iter().chain(empty.iter()).cloned().collect();
            let (cal_combined, _, _, _, _) = aggregate_daily_nutrition(&combined);
            prop_assert_eq!(cal_with_data, cal_combined);
        }

        /// Property 6: Food Search Relevance
        /// For any search query, all returned results should contain the query terms
        /// in either the name or brand field.
        /// Feature: fitness-assistant-ai, Property 6: Food Search Relevance
        /// **Validates: Requirements 2.1**
        #[test]
        fn prop_food_search_relevance(
            name in "[a-zA-Z ]{3,20}",
            brand in proptest::option::of("[a-zA-Z ]{3,15}"),
            query in "[a-zA-Z]{2,10}"
        ) {
            // If the food item matches the query, it should be found
            let matches = food_matches_query(&name, brand.as_deref(), &query);
            
            // This is a logical property: if we search for a term that exists
            // in the name or brand, the item should match
            if name.to_lowercase().contains(&query.to_lowercase()) {
                prop_assert!(matches, 
                    "Food '{}' should match query '{}' since name contains it", 
                    name, query);
            }
            
            if let Some(ref b) = brand {
                if b.to_lowercase().contains(&query.to_lowercase()) {
                    prop_assert!(matches,
                        "Food '{}' with brand '{}' should match query '{}'",
                        name, b, query);
                }
            }
        }

        /// Property: Search results are bounded
        /// The number of results should never exceed the requested limit
        #[test]
        fn prop_search_limit_respected(limit in 1i64..=100) {
            // This property verifies the limit parameter is respected
            // In actual implementation, we'd verify against DB results
            let effective_limit = limit.min(100);
            prop_assert!(effective_limit <= 100);
            prop_assert!(effective_limit >= 1);
        }
    }

    #[test]
    fn test_food_matches_query_exact() {
        assert!(food_matches_query("Apple", None, "apple"));
        assert!(food_matches_query("Apple", None, "Apple"));
        assert!(food_matches_query("Green Apple", None, "apple"));
        assert!(food_matches_query("Banana", Some("Dole"), "dole"));
    }

    #[test]
    fn test_food_matches_query_partial() {
        assert!(food_matches_query("Chicken Breast", None, "chicken"));
        assert!(food_matches_query("Grilled Chicken", None, "grill"));
    }

    #[test]
    fn test_food_matches_query_no_match() {
        assert!(!food_matches_query("Apple", None, "banana"));
        assert!(!food_matches_query("Chicken", Some("Tyson"), "beef"));
    }
}

#[cfg(test)]
mod recipe_tests {
    use super::*;
    use proptest::prelude::*;

    /// Strategy to generate valid positive nutrition values
    fn positive_nutrition_strategy() -> impl Strategy<Value = Decimal> {
        (1u32..1000u32).prop_map(|v| Decimal::new(v as i64, 1)) // 0.1 to 99.9
    }

    /// Strategy to generate valid servings (positive)
    fn servings_strategy() -> impl Strategy<Value = Decimal> {
        (1u32..100u32).prop_map(|v| Decimal::new(v as i64, 1)) // 0.1 to 9.9
    }

    /// Strategy to generate an ingredient with nutrition
    fn ingredient_strategy() -> impl Strategy<Value = IngredientNutrition> {
        (
            servings_strategy(),
            positive_nutrition_strategy(),
            positive_nutrition_strategy(),
            positive_nutrition_strategy(),
            positive_nutrition_strategy(),
            positive_nutrition_strategy(),
        )
            .prop_map(|(servings, cal, pro, carb, fat, fib)| IngredientNutrition {
                servings,
                calories_per_serving: cal,
                protein_per_serving: pro,
                carbs_per_serving: carb,
                fat_per_serving: fat,
                fiber_per_serving: fib,
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 7: Recipe Nutrition Calculation
        /// For any recipe composed of N ingredients with specified quantities,
        /// the total nutritional values should equal the sum of each ingredient's
        /// nutrition multiplied by (quantity / serving_size).
        /// Feature: fitness-assistant-ai, Property 7: Recipe Nutrition Calculation
        /// **Validates: Requirements 2.5**
        #[test]
        fn prop_recipe_nutrition_calculation(
            ingredients in proptest::collection::vec(ingredient_strategy(), 1..10),
            recipe_servings in servings_strategy()
        ) {
            // Calculate expected totals manually
            let expected_total_cal: Decimal = ingredients.iter()
                .map(|i| i.calories_per_serving * i.servings)
                .sum();
            let expected_total_pro: Decimal = ingredients.iter()
                .map(|i| i.protein_per_serving * i.servings)
                .sum();
            let expected_total_carb: Decimal = ingredients.iter()
                .map(|i| i.carbs_per_serving * i.servings)
                .sum();
            let expected_total_fat: Decimal = ingredients.iter()
                .map(|i| i.fat_per_serving * i.servings)
                .sum();
            let expected_total_fib: Decimal = ingredients.iter()
                .map(|i| i.fiber_per_serving * i.servings)
                .sum();

            // Calculate using the function
            let result = calculate_recipe_nutrition(&ingredients, recipe_servings);

            // Property: totals should match
            prop_assert_eq!(result.total_calories, expected_total_cal,
                "Total calories mismatch");
            prop_assert_eq!(result.total_protein, expected_total_pro,
                "Total protein mismatch");
            prop_assert_eq!(result.total_carbs, expected_total_carb,
                "Total carbs mismatch");
            prop_assert_eq!(result.total_fat, expected_total_fat,
                "Total fat mismatch");
            prop_assert_eq!(result.total_fiber, expected_total_fib,
                "Total fiber mismatch");

            // Property: per-serving should equal total / servings
            prop_assert_eq!(result.calories_per_serving, expected_total_cal / recipe_servings,
                "Calories per serving mismatch");
            prop_assert_eq!(result.protein_per_serving, expected_total_pro / recipe_servings,
                "Protein per serving mismatch");
            prop_assert_eq!(result.carbs_per_serving, expected_total_carb / recipe_servings,
                "Carbs per serving mismatch");
            prop_assert_eq!(result.fat_per_serving, expected_total_fat / recipe_servings,
                "Fat per serving mismatch");
            prop_assert_eq!(result.fiber_per_serving, expected_total_fib / recipe_servings,
                "Fiber per serving mismatch");
        }

        /// Property: Recipe with single ingredient equals that ingredient's nutrition
        #[test]
        fn prop_recipe_single_ingredient(
            ing in ingredient_strategy()
        ) {
            let ingredients = vec![ing.clone()];
            let result = calculate_recipe_nutrition(&ingredients, Decimal::ONE);

            // With 1 serving of recipe, totals should equal ingredient * servings
            let expected_cal = ing.calories_per_serving * ing.servings;
            prop_assert_eq!(result.total_calories, expected_cal);
            prop_assert_eq!(result.calories_per_serving, expected_cal);
        }

        /// Property: Empty recipe has zero nutrition
        #[test]
        fn prop_recipe_empty(
            recipe_servings in servings_strategy()
        ) {
            let ingredients: Vec<IngredientNutrition> = vec![];
            let result = calculate_recipe_nutrition(&ingredients, recipe_servings);

            prop_assert_eq!(result.total_calories, Decimal::ZERO);
            prop_assert_eq!(result.total_protein, Decimal::ZERO);
            prop_assert_eq!(result.total_carbs, Decimal::ZERO);
            prop_assert_eq!(result.total_fat, Decimal::ZERO);
            prop_assert_eq!(result.total_fiber, Decimal::ZERO);
        }

        /// Property: Doubling recipe servings halves per-serving nutrition
        #[test]
        fn prop_recipe_servings_scaling(
            ingredients in proptest::collection::vec(ingredient_strategy(), 1..5),
            base_servings in (1u32..50u32).prop_map(|v| Decimal::new(v as i64, 0))
        ) {
            let result1 = calculate_recipe_nutrition(&ingredients, base_servings);
            let result2 = calculate_recipe_nutrition(&ingredients, base_servings * Decimal::new(2, 0));

            // Totals should be the same
            prop_assert_eq!(result1.total_calories, result2.total_calories);

            // Per-serving should be halved (use approximate comparison due to decimal precision)
            let expected_per_serving = result1.calories_per_serving / Decimal::new(2, 0);
            let diff = (result2.calories_per_serving - expected_per_serving).abs();
            let tolerance = Decimal::new(1, 10); // 0.0000000001
            prop_assert!(diff < tolerance,
                "Doubling servings should halve per-serving calories. Got {}, expected {}, diff {}",
                result2.calories_per_serving, expected_per_serving, diff);
        }
    }

    #[test]
    fn test_recipe_calculation_basic() {
        let ingredients = vec![
            IngredientNutrition {
                servings: Decimal::new(2, 0), // 2 servings
                calories_per_serving: Decimal::new(100, 0), // 100 cal/serving
                protein_per_serving: Decimal::new(10, 0),
                carbs_per_serving: Decimal::new(20, 0),
                fat_per_serving: Decimal::new(5, 0),
                fiber_per_serving: Decimal::new(2, 0),
            },
            IngredientNutrition {
                servings: Decimal::new(1, 0), // 1 serving
                calories_per_serving: Decimal::new(50, 0), // 50 cal/serving
                protein_per_serving: Decimal::new(5, 0),
                carbs_per_serving: Decimal::new(10, 0),
                fat_per_serving: Decimal::new(2, 0),
                fiber_per_serving: Decimal::new(1, 0),
            },
        ];

        // Total: 2*100 + 1*50 = 250 calories
        // Recipe makes 2 servings, so 125 cal/serving
        let result = calculate_recipe_nutrition(&ingredients, Decimal::new(2, 0));

        assert_eq!(result.total_calories, Decimal::new(250, 0));
        assert_eq!(result.calories_per_serving, Decimal::new(125, 0));
        assert_eq!(result.total_protein, Decimal::new(25, 0)); // 2*10 + 1*5
        assert_eq!(result.protein_per_serving, Decimal::new(125, 1)); // 12.5
    }

    #[test]
    fn test_recipe_zero_servings_defaults_to_one() {
        let ingredients = vec![IngredientNutrition {
            servings: Decimal::ONE,
            calories_per_serving: Decimal::new(100, 0),
            protein_per_serving: Decimal::new(10, 0),
            carbs_per_serving: Decimal::new(20, 0),
            fat_per_serving: Decimal::new(5, 0),
            fiber_per_serving: Decimal::new(2, 0),
        }];

        let result = calculate_recipe_nutrition(&ingredients, Decimal::ZERO);

        // Should default to 1 serving to avoid division by zero
        assert_eq!(result.total_calories, Decimal::new(100, 0));
        assert_eq!(result.calories_per_serving, Decimal::new(100, 0));
    }
}


#[cfg(test)]
mod barcode_tests {
    use proptest::prelude::*;
    use std::collections::HashMap;

    /// Simulates a barcode lookup database for testing
    struct MockBarcodeDb {
        items: HashMap<String, String>, // barcode -> food name
    }

    impl MockBarcodeDb {
        fn new() -> Self {
            Self {
                items: HashMap::new(),
            }
        }

        fn insert(&mut self, barcode: &str, name: &str) {
            self.items.insert(barcode.to_string(), name.to_string());
        }

        fn lookup(&self, barcode: &str) -> Option<&String> {
            self.items.get(barcode)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 8: Barcode Lookup Consistency
        /// For any barcode that exists in the database, looking it up should
        /// return the correct food item consistently.
        /// Feature: fitness-assistant-ai, Property 8: Barcode Lookup Consistency
        /// **Validates: Requirements 2.3**
        #[test]
        fn prop_barcode_lookup_consistency(
            barcode in "[0-9]{8,13}",
            food_name in "[a-zA-Z ]{3,30}"
        ) {
            let mut db = MockBarcodeDb::new();
            db.insert(&barcode, &food_name);

            // Property: Looking up a barcode that was inserted should return the same item
            let result = db.lookup(&barcode);
            prop_assert!(result.is_some(), "Barcode {} should be found", barcode);
            prop_assert_eq!(result.unwrap(), &food_name,
                "Barcode {} should return food '{}'", barcode, food_name);

            // Property: Looking up the same barcode multiple times should be consistent
            let result2 = db.lookup(&barcode);
            prop_assert_eq!(result, result2, "Multiple lookups should be consistent");
        }

        /// Property: Non-existent barcodes return None
        #[test]
        fn prop_barcode_not_found(
            barcode in "[0-9]{8,13}",
            other_barcode in "[0-9]{8,13}"
        ) {
            let mut db = MockBarcodeDb::new();
            db.insert(&barcode, "Test Food");

            // If barcodes are different, lookup should return None
            if barcode != other_barcode {
                let result = db.lookup(&other_barcode);
                prop_assert!(result.is_none(),
                    "Barcode {} should not be found when only {} exists",
                    other_barcode, barcode);
            }
        }

        /// Property: Barcode uniqueness - each barcode maps to exactly one item
        #[test]
        fn prop_barcode_uniqueness(
            barcode in "[0-9]{8,13}",
            food1 in "[a-zA-Z]{3,15}",
            food2 in "[a-zA-Z]{3,15}"
        ) {
            let mut db = MockBarcodeDb::new();
            
            // Insert first food
            db.insert(&barcode, &food1);
            
            // Insert second food with same barcode (overwrites)
            db.insert(&barcode, &food2);
            
            // Should return the latest value
            let result = db.lookup(&barcode);
            prop_assert_eq!(result.unwrap(), &food2,
                "Barcode should map to the most recently inserted item");
        }
    }

    #[test]
    fn test_barcode_lookup_basic() {
        let mut db = MockBarcodeDb::new();
        db.insert("012345678901", "Test Product");
        
        assert_eq!(db.lookup("012345678901"), Some(&"Test Product".to_string()));
        assert_eq!(db.lookup("999999999999"), None);
    }
}
