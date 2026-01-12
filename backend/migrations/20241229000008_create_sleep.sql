-- Sleep tracking tables
-- Migration: 20241229000008_create_sleep.sql

-- Sleep logs table with stage breakdown
CREATE TABLE IF NOT EXISTS sleep_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Sleep timing
    sleep_start TIMESTAMPTZ NOT NULL,
    sleep_end TIMESTAMPTZ NOT NULL,
    
    -- Total duration in minutes (calculated from start/end)
    total_duration_minutes INT NOT NULL,
    
    -- Sleep stage breakdown in minutes
    awake_minutes INT NOT NULL DEFAULT 0,
    light_minutes INT NOT NULL DEFAULT 0,
    deep_minutes INT NOT NULL DEFAULT 0,
    rem_minutes INT NOT NULL DEFAULT 0,
    
    -- Sleep quality metrics
    sleep_efficiency DECIMAL(5, 2), -- Percentage (0-100)
    sleep_score INT, -- Overall score (0-100)
    
    -- Additional metrics
    times_awoken INT DEFAULT 0,
    avg_heart_rate INT,
    min_heart_rate INT,
    hrv_average DECIMAL(6, 2),
    respiratory_rate DECIMAL(4, 1),
    
    -- Metadata
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    notes TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_duration CHECK (total_duration_minutes > 0),
    CONSTRAINT valid_stages CHECK (
        awake_minutes >= 0 AND 
        light_minutes >= 0 AND 
        deep_minutes >= 0 AND 
        rem_minutes >= 0
    ),
    CONSTRAINT valid_efficiency CHECK (
        sleep_efficiency IS NULL OR 
        (sleep_efficiency >= 0 AND sleep_efficiency <= 100)
    ),
    CONSTRAINT valid_score CHECK (
        sleep_score IS NULL OR 
        (sleep_score >= 0 AND sleep_score <= 100)
    ),
    CONSTRAINT valid_sleep_times CHECK (sleep_end > sleep_start)
);

-- Index for user queries by date (most common query pattern)
CREATE INDEX idx_sleep_logs_user_date ON sleep_logs(user_id, DATE(sleep_end) DESC);

-- Index for date range queries
CREATE INDEX idx_sleep_logs_user_sleep_end ON sleep_logs(user_id, sleep_end DESC);

-- Sleep goals table
CREATE TABLE IF NOT EXISTS sleep_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Target sleep duration in minutes
    target_duration_minutes INT NOT NULL DEFAULT 480, -- 8 hours
    
    -- Target bedtime and wake time (time of day)
    target_bedtime TIME,
    target_wake_time TIME,
    
    -- Reminder settings
    bedtime_reminder_enabled BOOLEAN NOT NULL DEFAULT false,
    bedtime_reminder_minutes_before INT DEFAULT 30,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- One goal per user
    CONSTRAINT unique_user_sleep_goal UNIQUE (user_id),
    CONSTRAINT valid_target_duration CHECK (
        target_duration_minutes >= 60 AND target_duration_minutes <= 1440
    )
);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_sleep_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER sleep_logs_updated_at
    BEFORE UPDATE ON sleep_logs
    FOR EACH ROW
    EXECUTE FUNCTION update_sleep_updated_at();

CREATE TRIGGER sleep_goals_updated_at
    BEFORE UPDATE ON sleep_goals
    FOR EACH ROW
    EXECUTE FUNCTION update_sleep_updated_at();
