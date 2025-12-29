-- Weight and Body Composition Tracking Tables
-- Migration: 20241229000002_create_weight_logs

-- Weight logs table
CREATE TABLE weight_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    weight_kg DECIMAL(5,2) NOT NULL CHECK (weight_kg >= 20 AND weight_kg <= 500),
    recorded_at TIMESTAMPTZ NOT NULL,
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    notes TEXT,
    is_anomaly BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient user+date queries (most common access pattern)
CREATE INDEX idx_weight_logs_user_date ON weight_logs(user_id, recorded_at DESC);

-- Body composition logs table
CREATE TABLE body_composition_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recorded_at TIMESTAMPTZ NOT NULL,
    body_fat_percent DECIMAL(4,1) CHECK (body_fat_percent >= 0 AND body_fat_percent <= 100),
    muscle_mass_kg DECIMAL(5,2) CHECK (muscle_mass_kg >= 0),
    water_percent DECIMAL(4,1) CHECK (water_percent >= 0 AND water_percent <= 100),
    bone_mass_kg DECIMAL(4,2) CHECK (bone_mass_kg >= 0),
    visceral_fat INTEGER CHECK (visceral_fat >= 0),
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient user+date queries
CREATE INDEX idx_body_composition_logs_user_date ON body_composition_logs(user_id, recorded_at DESC);
