-- Nutrition tracking tables
-- Migration: 20241229000004_create_nutrition.sql

-- Food items database with nutritional information
CREATE TABLE food_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    brand VARCHAR(255),
    barcode VARCHAR(50),
    
    -- Nutritional values per serving
    serving_size DECIMAL(10, 2) NOT NULL,
    serving_unit VARCHAR(50) NOT NULL DEFAULT 'g',
    
    -- Macronutrients (per serving)
    calories DECIMAL(10, 2) NOT NULL DEFAULT 0,
    protein_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    carbohydrates_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    fat_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    fiber_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    sugar_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    
    -- Micronutrients (optional, per serving)
    sodium_mg DECIMAL(10, 2),
    potassium_mg DECIMAL(10, 2),
    cholesterol_mg DECIMAL(10, 2),
    vitamin_a_iu DECIMAL(10, 2),
    vitamin_c_mg DECIMAL(10, 2),
    calcium_mg DECIMAL(10, 2),
    iron_mg DECIMAL(10, 2),
    
    -- Metadata
    source VARCHAR(50) NOT NULL DEFAULT 'user', -- 'user', 'usda', 'openfoodfacts'
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Full-text search index for food items
CREATE INDEX idx_food_items_search ON food_items 
    USING GIN (to_tsvector('english', name || ' ' || COALESCE(brand, '')));

-- Barcode lookup index
CREATE UNIQUE INDEX idx_food_items_barcode ON food_items(barcode) WHERE barcode IS NOT NULL;

-- Name index for exact matches
CREATE INDEX idx_food_items_name ON food_items(LOWER(name));

-- Food logs - user's daily food intake
CREATE TABLE food_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    food_item_id UUID REFERENCES food_items(id) ON DELETE SET NULL,
    
    -- If food_item_id is null, store custom entry
    custom_name VARCHAR(255),
    
    -- Serving information
    servings DECIMAL(10, 2) NOT NULL DEFAULT 1,
    
    -- Calculated nutritional values (at time of logging)
    calories DECIMAL(10, 2) NOT NULL,
    protein_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    carbohydrates_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    fat_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    fiber_g DECIMAL(10, 2) NOT NULL DEFAULT 0,
    
    -- Meal categorization
    meal_type VARCHAR(20) NOT NULL DEFAULT 'snack', -- 'breakfast', 'lunch', 'dinner', 'snack'
    
    -- Timing
    logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Notes
    notes TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for user's daily food logs
CREATE INDEX idx_food_logs_user_date ON food_logs(user_id, DATE(consumed_at));
CREATE INDEX idx_food_logs_user_consumed ON food_logs(user_id, consumed_at);


-- Recipes - user-created combinations of food items
CREATE TABLE recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Total servings the recipe makes
    servings DECIMAL(10, 2) NOT NULL DEFAULT 1,
    
    -- Calculated totals (per serving)
    calories_per_serving DECIMAL(10, 2) NOT NULL DEFAULT 0,
    protein_per_serving DECIMAL(10, 2) NOT NULL DEFAULT 0,
    carbs_per_serving DECIMAL(10, 2) NOT NULL DEFAULT 0,
    fat_per_serving DECIMAL(10, 2) NOT NULL DEFAULT 0,
    fiber_per_serving DECIMAL(10, 2) NOT NULL DEFAULT 0,
    
    -- Metadata
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_recipes_user ON recipes(user_id);
CREATE INDEX idx_recipes_public ON recipes(is_public) WHERE is_public = TRUE;

-- Recipe ingredients - junction table
CREATE TABLE recipe_ingredients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    food_item_id UUID NOT NULL REFERENCES food_items(id) ON DELETE CASCADE,
    
    -- Amount of this ingredient
    servings DECIMAL(10, 2) NOT NULL DEFAULT 1,
    
    -- Order in recipe
    sort_order INT NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_recipe_ingredients_recipe ON recipe_ingredients(recipe_id);
CREATE UNIQUE INDEX idx_recipe_ingredients_unique ON recipe_ingredients(recipe_id, food_item_id);

-- Trigger to update recipe nutritional totals when ingredients change
CREATE OR REPLACE FUNCTION update_recipe_nutrition()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE recipes r
    SET 
        calories_per_serving = COALESCE(totals.calories, 0) / NULLIF(r.servings, 0),
        protein_per_serving = COALESCE(totals.protein, 0) / NULLIF(r.servings, 0),
        carbs_per_serving = COALESCE(totals.carbs, 0) / NULLIF(r.servings, 0),
        fat_per_serving = COALESCE(totals.fat, 0) / NULLIF(r.servings, 0),
        fiber_per_serving = COALESCE(totals.fiber, 0) / NULLIF(r.servings, 0),
        updated_at = NOW()
    FROM (
        SELECT 
            ri.recipe_id,
            SUM(fi.calories * ri.servings) as calories,
            SUM(fi.protein_g * ri.servings) as protein,
            SUM(fi.carbohydrates_g * ri.servings) as carbs,
            SUM(fi.fat_g * ri.servings) as fat,
            SUM(fi.fiber_g * ri.servings) as fiber
        FROM recipe_ingredients ri
        JOIN food_items fi ON fi.id = ri.food_item_id
        WHERE ri.recipe_id = COALESCE(NEW.recipe_id, OLD.recipe_id)
        GROUP BY ri.recipe_id
    ) totals
    WHERE r.id = totals.recipe_id;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_recipe_nutrition
AFTER INSERT OR UPDATE OR DELETE ON recipe_ingredients
FOR EACH ROW EXECUTE FUNCTION update_recipe_nutrition();

-- Daily nutrition goals (optional per-user override)
CREATE TABLE nutrition_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Daily targets
    calories_target DECIMAL(10, 2),
    protein_target_g DECIMAL(10, 2),
    carbs_target_g DECIMAL(10, 2),
    fat_target_g DECIMAL(10, 2),
    fiber_target_g DECIMAL(10, 2),
    
    -- Macro ratios (percentages, should sum to 100)
    protein_ratio DECIMAL(5, 2),
    carbs_ratio DECIMAL(5, 2),
    fat_ratio DECIMAL(5, 2),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id)
);
