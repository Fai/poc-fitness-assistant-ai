-- Hydration tracking tables
-- Tracks water intake and daily hydration goals

-- Hydration logs table
CREATE TABLE IF NOT EXISTS hydration_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Amount consumed
    amount_ml INTEGER NOT NULL CHECK (amount_ml > 0 AND amount_ml <= 10000),
    
    -- Beverage type (water, tea, coffee, etc.)
    beverage_type VARCHAR(50) NOT NULL DEFAULT 'water',
    
    -- When consumed
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Source of entry (manual, quick, device)
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    
    -- Optional notes
    notes TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for user's hydration logs by date (most common query pattern)
CREATE INDEX idx_hydration_logs_user_date ON hydration_logs(user_id, DATE(consumed_at));

-- Index for daily aggregation queries
CREATE INDEX idx_hydration_logs_consumed_at ON hydration_logs(consumed_at DESC);

-- Hydration goals table (user-specific daily goals)
CREATE TABLE IF NOT EXISTS hydration_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Daily goal in ml
    daily_goal_ml INTEGER NOT NULL CHECK (daily_goal_ml > 0 AND daily_goal_ml <= 20000),
    
    -- Whether goal is auto-calculated based on weight
    is_auto_calculated BOOLEAN NOT NULL DEFAULT true,
    
    -- Reminder settings
    reminders_enabled BOOLEAN NOT NULL DEFAULT false,
    reminder_interval_minutes INTEGER CHECK (reminder_interval_minutes IS NULL OR (reminder_interval_minutes >= 15 AND reminder_interval_minutes <= 480)),
    reminder_start_time TIME,
    reminder_end_time TIME,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- One goal per user
    CONSTRAINT unique_user_hydration_goal UNIQUE (user_id)
);

-- Trigger to update updated_at on hydration_goals
CREATE OR REPLACE FUNCTION update_hydration_goals_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_hydration_goals_updated_at
    BEFORE UPDATE ON hydration_goals
    FOR EACH ROW
    EXECUTE FUNCTION update_hydration_goals_updated_at();

-- Daily hydration summary view (materialized for performance)
-- This can be used for quick daily lookups
CREATE OR REPLACE VIEW hydration_daily_summary AS
SELECT 
    user_id,
    DATE(consumed_at) as date,
    SUM(amount_ml) as total_ml,
    COUNT(*) as entry_count,
    MIN(consumed_at) as first_entry,
    MAX(consumed_at) as last_entry
FROM hydration_logs
GROUP BY user_id, DATE(consumed_at);

-- Comments for documentation
COMMENT ON TABLE hydration_logs IS 'Individual water/beverage intake entries';
COMMENT ON TABLE hydration_goals IS 'User-specific daily hydration goals and reminder settings';
COMMENT ON COLUMN hydration_logs.amount_ml IS 'Amount consumed in milliliters';
COMMENT ON COLUMN hydration_logs.beverage_type IS 'Type of beverage: water, tea, coffee, juice, etc.';
COMMENT ON COLUMN hydration_goals.daily_goal_ml IS 'Daily hydration target in milliliters';
COMMENT ON COLUMN hydration_goals.is_auto_calculated IS 'True if goal is calculated from user weight';
