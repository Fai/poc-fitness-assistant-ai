-- Heart rate and HRV tracking tables
-- Migration: 20241229000009_create_biometrics.sql

-- Heart rate logs table
CREATE TABLE IF NOT EXISTS heart_rate_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Heart rate value
    bpm INT NOT NULL,
    
    -- Context of measurement
    context VARCHAR(50) NOT NULL DEFAULT 'resting', -- resting, active, workout, sleep, recovery
    
    -- When measured
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Optional workout reference
    workout_id UUID REFERENCES workouts(id) ON DELETE SET NULL,
    
    -- Metadata
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    notes TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_bpm CHECK (bpm > 0 AND bpm < 300)
);

-- Index for user queries by date
CREATE INDEX idx_heart_rate_logs_user_date ON heart_rate_logs(user_id, recorded_at DESC);

-- Index for resting heart rate queries
CREATE INDEX idx_heart_rate_logs_resting ON heart_rate_logs(user_id, recorded_at DESC) 
    WHERE context = 'resting';

-- HRV (Heart Rate Variability) logs table
CREATE TABLE IF NOT EXISTS hrv_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- HRV metrics (RMSSD is most common)
    rmssd DECIMAL(8, 2) NOT NULL, -- Root Mean Square of Successive Differences (ms)
    sdnn DECIMAL(8, 2), -- Standard Deviation of NN intervals (ms)
    
    -- Context
    context VARCHAR(50) NOT NULL DEFAULT 'morning', -- morning, sleep, recovery, workout
    
    -- When measured
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Metadata
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    notes TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_rmssd CHECK (rmssd > 0 AND rmssd < 500),
    CONSTRAINT valid_sdnn CHECK (sdnn IS NULL OR (sdnn > 0 AND sdnn < 500))
);

-- Index for user queries by date
CREATE INDEX idx_hrv_logs_user_date ON hrv_logs(user_id, recorded_at DESC);

-- Index for morning HRV (baseline) queries
CREATE INDEX idx_hrv_logs_morning ON hrv_logs(user_id, recorded_at DESC) 
    WHERE context = 'morning';

-- Heart rate zones table (user-specific)
CREATE TABLE IF NOT EXISTS heart_rate_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Max heart rate (can be measured or calculated)
    max_heart_rate INT NOT NULL,
    resting_heart_rate INT,
    
    -- Zone boundaries (as percentage of max HR or HRR)
    zone1_min INT NOT NULL, -- Recovery/Easy
    zone1_max INT NOT NULL,
    zone2_min INT NOT NULL, -- Aerobic/Fat burn
    zone2_max INT NOT NULL,
    zone3_min INT NOT NULL, -- Tempo/Cardio
    zone3_max INT NOT NULL,
    zone4_min INT NOT NULL, -- Threshold/Hard
    zone4_max INT NOT NULL,
    zone5_min INT NOT NULL, -- VO2 Max/Peak
    zone5_max INT NOT NULL,
    
    -- Calculation method
    calculation_method VARCHAR(50) NOT NULL DEFAULT 'percentage', -- percentage, karvonen
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- One zone config per user
    CONSTRAINT unique_user_hr_zones UNIQUE (user_id),
    CONSTRAINT valid_max_hr CHECK (max_heart_rate > 0 AND max_heart_rate < 250),
    CONSTRAINT valid_resting_hr CHECK (resting_heart_rate IS NULL OR (resting_heart_rate > 0 AND resting_heart_rate < 150))
);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_hr_zones_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER heart_rate_zones_updated_at
    BEFORE UPDATE ON heart_rate_zones
    FOR EACH ROW
    EXECUTE FUNCTION update_hr_zones_updated_at();
