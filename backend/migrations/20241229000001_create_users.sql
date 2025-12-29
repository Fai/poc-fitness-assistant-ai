-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on email for faster lookups
CREATE INDEX idx_users_email ON users(email);

-- Create user_settings table
CREATE TABLE user_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    weight_unit VARCHAR(10) NOT NULL DEFAULT 'kg',
    distance_unit VARCHAR(10) NOT NULL DEFAULT 'km',
    energy_unit VARCHAR(10) NOT NULL DEFAULT 'kcal',
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    daily_calorie_goal INTEGER,
    daily_water_goal_ml INTEGER,
    daily_step_goal INTEGER,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_settings_updated_at
    BEFORE UPDATE ON user_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
