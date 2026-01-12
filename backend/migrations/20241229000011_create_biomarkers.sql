-- Biomarker and supplement tracking tables
-- Migration: 20241229000011_create_biomarkers.sql

-- Biomarker reference ranges table
CREATE TABLE IF NOT EXISTS biomarker_ranges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    category VARCHAR(50) NOT NULL, -- blood, hormone, vitamin, mineral, lipid, metabolic
    unit VARCHAR(50) NOT NULL,
    
    -- Reference ranges (can vary by sex/age, these are general)
    low_threshold DECIMAL(12, 4),
    optimal_min DECIMAL(12, 4),
    optimal_max DECIMAL(12, 4),
    high_threshold DECIMAL(12, 4),
    
    -- Metadata
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed common biomarkers
INSERT INTO biomarker_ranges (name, display_name, category, unit, low_threshold, optimal_min, optimal_max, high_threshold, description) VALUES
-- Blood markers
('hemoglobin', 'Hemoglobin', 'blood', 'g/dL', 12.0, 13.5, 17.5, 18.5, 'Oxygen-carrying protein in red blood cells'),
('hematocrit', 'Hematocrit', 'blood', '%', 36, 38, 50, 54, 'Percentage of blood volume occupied by red blood cells'),
('wbc', 'White Blood Cells', 'blood', 'K/uL', 3.5, 4.5, 11.0, 12.0, 'Immune system cells'),
('platelets', 'Platelets', 'blood', 'K/uL', 140, 150, 400, 450, 'Blood clotting cells'),

-- Lipid panel
('total_cholesterol', 'Total Cholesterol', 'lipid', 'mg/dL', NULL, NULL, 200, 240, 'Total blood cholesterol'),
('ldl', 'LDL Cholesterol', 'lipid', 'mg/dL', NULL, NULL, 100, 130, 'Low-density lipoprotein (bad cholesterol)'),
('hdl', 'HDL Cholesterol', 'lipid', 'mg/dL', 40, 60, NULL, NULL, 'High-density lipoprotein (good cholesterol)'),
('triglycerides', 'Triglycerides', 'lipid', 'mg/dL', NULL, NULL, 150, 200, 'Blood fat'),

-- Metabolic markers
('glucose_fasting', 'Fasting Glucose', 'metabolic', 'mg/dL', 65, 70, 100, 126, 'Blood sugar after fasting'),
('hba1c', 'HbA1c', 'metabolic', '%', NULL, NULL, 5.7, 6.5, 'Average blood sugar over 3 months'),
('insulin_fasting', 'Fasting Insulin', 'metabolic', 'uIU/mL', NULL, 2, 10, 25, 'Fasting insulin level'),

-- Vitamins
('vitamin_d', 'Vitamin D (25-OH)', 'vitamin', 'ng/mL', 20, 40, 80, 100, 'Vitamin D status'),
('vitamin_b12', 'Vitamin B12', 'vitamin', 'pg/mL', 200, 400, 900, 1100, 'B12 status'),
('folate', 'Folate', 'vitamin', 'ng/mL', 3, 5, 20, 25, 'Folate/B9 status'),

-- Minerals
('iron', 'Iron', 'mineral', 'ug/dL', 50, 60, 170, 200, 'Serum iron'),
('ferritin', 'Ferritin', 'mineral', 'ng/mL', 20, 40, 200, 300, 'Iron storage'),
('magnesium', 'Magnesium', 'mineral', 'mg/dL', 1.5, 1.8, 2.3, 2.6, 'Serum magnesium'),

-- Hormones
('testosterone_total', 'Total Testosterone', 'hormone', 'ng/dL', 250, 400, 900, 1100, 'Total testosterone (male reference)'),
('cortisol_am', 'Cortisol (AM)', 'hormone', 'ug/dL', 5, 10, 20, 25, 'Morning cortisol'),
('tsh', 'TSH', 'hormone', 'mIU/L', 0.4, 0.5, 4.0, 5.0, 'Thyroid stimulating hormone'),
('free_t4', 'Free T4', 'hormone', 'ng/dL', 0.7, 0.9, 1.7, 2.0, 'Free thyroxine'),
('free_t3', 'Free T3', 'hormone', 'pg/mL', 2.0, 2.3, 4.2, 4.8, 'Free triiodothyronine')
ON CONFLICT (name) DO NOTHING;

-- Biomarker logs table
CREATE TABLE IF NOT EXISTS biomarker_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    biomarker_id UUID NOT NULL REFERENCES biomarker_ranges(id),
    
    -- Value and classification
    value DECIMAL(12, 4) NOT NULL,
    classification VARCHAR(20), -- low, optimal, high, critical_low, critical_high
    
    -- Test metadata
    test_date DATE NOT NULL,
    lab_name VARCHAR(255),
    notes TEXT,
    
    -- Source
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for user biomarker queries
CREATE INDEX idx_biomarker_logs_user_date ON biomarker_logs(user_id, test_date DESC);
CREATE INDEX idx_biomarker_logs_user_biomarker ON biomarker_logs(user_id, biomarker_id, test_date DESC);

-- Supplements table
CREATE TABLE IF NOT EXISTS supplements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Supplement info
    name VARCHAR(255) NOT NULL,
    brand VARCHAR(255),
    dosage VARCHAR(100) NOT NULL,
    frequency VARCHAR(50) NOT NULL, -- daily, twice_daily, weekly, as_needed
    
    -- Schedule
    time_of_day VARCHAR(50), -- morning, afternoon, evening, with_meals
    start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    end_date DATE,
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    notes TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for user supplements
CREATE INDEX idx_supplements_user_active ON supplements(user_id) WHERE is_active = true;

-- Supplement logs table (for tracking adherence)
CREATE TABLE IF NOT EXISTS supplement_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplement_id UUID NOT NULL REFERENCES supplements(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Log entry
    taken_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    skipped BOOLEAN NOT NULL DEFAULT false,
    notes TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for supplement adherence queries
CREATE INDEX idx_supplement_logs_supplement_date ON supplement_logs(supplement_id, DATE(taken_at) DESC);
CREATE INDEX idx_supplement_logs_user_date ON supplement_logs(user_id, DATE(taken_at) DESC);

-- Trigger for supplements updated_at
CREATE OR REPLACE FUNCTION update_supplements_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER supplements_updated_at
    BEFORE UPDATE ON supplements
    FOR EACH ROW
    EXECUTE FUNCTION update_supplements_updated_at();
