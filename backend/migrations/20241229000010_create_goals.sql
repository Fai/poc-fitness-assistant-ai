-- Goals and milestones tables
-- Migration: 20241229000010_create_goals.sql

-- Goals table
CREATE TABLE IF NOT EXISTS goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Goal definition
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Goal type and metric
    goal_type VARCHAR(50) NOT NULL, -- weight, exercise, nutrition, hydration, sleep, custom
    metric VARCHAR(100) NOT NULL, -- weight_kg, workout_count, calories, water_ml, sleep_hours, etc.
    
    -- Target values
    target_value DECIMAL(12, 4) NOT NULL,
    start_value DECIMAL(12, 4),
    current_value DECIMAL(12, 4),
    
    -- Direction: increasing (gain muscle) or decreasing (lose weight)
    direction VARCHAR(20) NOT NULL DEFAULT 'decreasing', -- increasing, decreasing
    
    -- Timeline
    start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    target_date DATE,
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, completed, abandoned, paused
    completed_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_goal_type CHECK (goal_type IN ('weight', 'exercise', 'nutrition', 'hydration', 'sleep', 'custom')),
    CONSTRAINT valid_direction CHECK (direction IN ('increasing', 'decreasing')),
    CONSTRAINT valid_status CHECK (status IN ('active', 'completed', 'abandoned', 'paused'))
);

-- Index for user's active goals
CREATE INDEX idx_goals_user_active ON goals(user_id, status) WHERE status = 'active';

-- Index for user's goals by type
CREATE INDEX idx_goals_user_type ON goals(user_id, goal_type);

-- Goal milestones table
CREATE TABLE IF NOT EXISTS goal_milestones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    goal_id UUID NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    
    -- Milestone definition
    name VARCHAR(255) NOT NULL,
    target_value DECIMAL(12, 4) NOT NULL,
    percentage INT NOT NULL, -- 25, 50, 75, 100
    
    -- Achievement
    achieved_at TIMESTAMPTZ,
    actual_value DECIMAL(12, 4),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_percentage CHECK (percentage >= 0 AND percentage <= 100)
);

-- Index for goal milestones
CREATE INDEX idx_milestones_goal ON goal_milestones(goal_id);

-- Trigger to update goals updated_at
CREATE OR REPLACE FUNCTION update_goals_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER goals_updated_at
    BEFORE UPDATE ON goals
    FOR EACH ROW
    EXECUTE FUNCTION update_goals_updated_at();
