-- Extend user profile with health-related data
-- Migration: 20241229000003_extend_user_profile

-- Add new columns to user_settings for health profile
ALTER TABLE user_settings
    ADD COLUMN height_cm DECIMAL(5,1),
    ADD COLUMN date_of_birth DATE,
    ADD COLUMN biological_sex VARCHAR(10),
    ADD COLUMN activity_level VARCHAR(20) DEFAULT 'lightly_active',
    ADD COLUMN height_unit VARCHAR(10) DEFAULT 'cm',
    ADD COLUMN temperature_unit VARCHAR(10) DEFAULT 'celsius';

-- Add constraints
ALTER TABLE user_settings
    ADD CONSTRAINT chk_height_range CHECK (height_cm IS NULL OR (height_cm >= 50 AND height_cm <= 300)),
    ADD CONSTRAINT chk_biological_sex CHECK (biological_sex IS NULL OR biological_sex IN ('male', 'female')),
    ADD CONSTRAINT chk_activity_level CHECK (activity_level IN ('sedentary', 'lightly_active', 'moderately_active', 'very_active', 'extra_active')),
    ADD CONSTRAINT chk_height_unit CHECK (height_unit IN ('cm', 'meters', 'inches', 'ft/in')),
    ADD CONSTRAINT chk_temperature_unit CHECK (temperature_unit IN ('celsius', 'fahrenheit'));

-- Create user_goals table for weight and other goals
CREATE TABLE IF NOT EXISTS user_weight_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_weight_kg DECIMAL(5,2) NOT NULL CHECK (target_weight_kg >= 20 AND target_weight_kg <= 500),
    start_weight_kg DECIMAL(5,2) NOT NULL CHECK (start_weight_kg >= 20 AND start_weight_kg <= 500),
    start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    target_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'abandoned')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, status) -- Only one active goal per user
);

CREATE INDEX idx_user_weight_goals_user ON user_weight_goals(user_id, status);

-- Add comment for documentation
COMMENT ON COLUMN user_settings.height_cm IS 'User height stored in centimeters (SI unit)';
COMMENT ON COLUMN user_settings.biological_sex IS 'Biological sex for physiological calculations (male/female)';
COMMENT ON COLUMN user_settings.activity_level IS 'Activity level for TDEE calculation';
