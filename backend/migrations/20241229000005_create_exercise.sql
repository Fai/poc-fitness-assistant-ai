-- Exercise and Workout Tracking Tables
-- Migration: 20241229000005_create_exercise.sql

-- Exercise library - predefined exercises with metadata
CREATE TABLE exercises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    category VARCHAR(50) NOT NULL, -- 'strength', 'cardio', 'flexibility', 'sports', 'other'
    muscle_groups TEXT[] NOT NULL DEFAULT '{}',
    equipment VARCHAR(100), -- 'barbell', 'dumbbell', 'machine', 'bodyweight', 'cable', etc.
    calories_per_minute DECIMAL(5, 2), -- estimated calories burned per minute
    description TEXT,
    instructions TEXT,
    is_custom BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for searching exercises by name
CREATE INDEX idx_exercises_name ON exercises(LOWER(name));
CREATE INDEX idx_exercises_category ON exercises(category);
CREATE INDEX idx_exercises_muscle_groups ON exercises USING GIN(muscle_groups);

-- Workouts - individual workout sessions
CREATE TABLE workouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255), -- optional workout name
    workout_type VARCHAR(50) NOT NULL, -- 'strength', 'cardio', 'flexibility', 'sports', 'mixed', 'other'
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_minutes INTEGER, -- can be calculated or manually entered
    calories_burned INTEGER,
    avg_heart_rate INTEGER CHECK (avg_heart_rate IS NULL OR (avg_heart_rate >= 30 AND avg_heart_rate <= 250)),
    max_heart_rate INTEGER CHECK (max_heart_rate IS NULL OR (max_heart_rate >= 30 AND max_heart_rate <= 250)),
    distance_meters DECIMAL(10, 2) CHECK (distance_meters IS NULL OR distance_meters >= 0),
    pace_seconds_per_km INTEGER, -- for cardio workouts
    elevation_gain_meters DECIMAL(8, 2),
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for user workout history queries
CREATE INDEX idx_workouts_user_date ON workouts(user_id, started_at DESC);
CREATE INDEX idx_workouts_user_type ON workouts(user_id, workout_type);

-- Workout exercises - exercises performed in a workout (for strength training)
CREATE TABLE workout_exercises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workout_id UUID NOT NULL REFERENCES workouts(id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL REFERENCES exercises(id) ON DELETE RESTRICT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workout_exercises_workout ON workout_exercises(workout_id);

-- Exercise sets - individual sets within a workout exercise
CREATE TABLE exercise_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workout_exercise_id UUID NOT NULL REFERENCES workout_exercises(id) ON DELETE CASCADE,
    set_number INTEGER NOT NULL,
    reps INTEGER CHECK (reps IS NULL OR reps >= 0),
    weight_kg DECIMAL(6, 2) CHECK (weight_kg IS NULL OR weight_kg >= 0),
    duration_seconds INTEGER CHECK (duration_seconds IS NULL OR duration_seconds >= 0),
    distance_meters DECIMAL(10, 2) CHECK (distance_meters IS NULL OR distance_meters >= 0),
    rest_seconds INTEGER CHECK (rest_seconds IS NULL OR rest_seconds >= 0),
    rpe DECIMAL(3, 1) CHECK (rpe IS NULL OR (rpe >= 1 AND rpe <= 10)), -- Rate of Perceived Exertion
    is_warmup BOOLEAN NOT NULL DEFAULT FALSE,
    is_dropset BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_exercise_sets_workout_exercise ON exercise_sets(workout_exercise_id);

-- Workout templates - saved workout routines
CREATE TABLE workout_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    workout_type VARCHAR(50) NOT NULL,
    estimated_duration_minutes INTEGER,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workout_templates_user ON workout_templates(user_id);

-- Template exercises - exercises in a workout template
CREATE TABLE template_exercises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES workout_templates(id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL REFERENCES exercises(id) ON DELETE RESTRICT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    target_sets INTEGER,
    target_reps INTEGER,
    target_weight_kg DECIMAL(6, 2),
    target_duration_seconds INTEGER,
    rest_seconds INTEGER,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_template_exercises_template ON template_exercises(template_id);

-- Trigger to update updated_at
CREATE TRIGGER update_exercises_updated_at
    BEFORE UPDATE ON exercises
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_workouts_updated_at
    BEFORE UPDATE ON workouts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_workout_templates_updated_at
    BEFORE UPDATE ON workout_templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
