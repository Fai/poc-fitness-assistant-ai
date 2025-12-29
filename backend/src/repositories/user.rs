//! User repository for database operations

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// User record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User settings record from database (extended)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserSettingsRecord {
    pub user_id: Uuid,
    pub weight_unit: String,
    pub distance_unit: String,
    pub energy_unit: String,
    pub timezone: String,
    pub daily_calorie_goal: Option<i32>,
    pub daily_water_goal_ml: Option<i32>,
    pub daily_step_goal: Option<i32>,
    pub height_cm: Option<Decimal>,
    pub date_of_birth: Option<NaiveDate>,
    pub biological_sex: Option<String>,
    pub activity_level: String,
    pub height_unit: String,
    pub temperature_unit: String,
    pub updated_at: DateTime<Utc>,
}

/// Input for updating user settings
#[derive(Debug, Clone, Default)]
pub struct UpdateUserSettings {
    pub weight_unit: Option<String>,
    pub distance_unit: Option<String>,
    pub energy_unit: Option<String>,
    pub timezone: Option<String>,
    pub daily_calorie_goal: Option<i32>,
    pub daily_water_goal_ml: Option<i32>,
    pub daily_step_goal: Option<i32>,
    pub height_cm: Option<f64>,
    pub date_of_birth: Option<NaiveDate>,
    pub biological_sex: Option<String>,
    pub activity_level: Option<String>,
    pub height_unit: Option<String>,
    pub temperature_unit: Option<String>,
}

/// User repository for database operations
pub struct UserRepository;

impl UserRepository {
    /// Create a new user with default settings
    pub async fn create(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
    ) -> Result<UserRecord> {
        let mut tx = pool.begin().await?;

        // Insert user
        let user = sqlx::query_as::<_, UserRecord>(
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, $2)
            RETURNING id, email, password_hash, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&mut *tx)
        .await?;

        // Create default settings
        sqlx::query(
            r#"
            INSERT INTO user_settings (user_id)
            VALUES ($1)
            "#,
        )
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRecord>> {
        let user = sqlx::query_as::<_, UserRecord>(
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRecord>> {
        let user = sqlx::query_as::<_, UserRecord>(
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Get user settings (extended)
    pub async fn get_settings(pool: &PgPool, user_id: Uuid) -> Result<Option<UserSettingsRecord>> {
        let settings = sqlx::query_as::<_, UserSettingsRecord>(
            r#"
            SELECT user_id, weight_unit, distance_unit, energy_unit, timezone,
                   daily_calorie_goal, daily_water_goal_ml, daily_step_goal,
                   height_cm, date_of_birth, biological_sex, activity_level,
                   height_unit, temperature_unit, updated_at
            FROM user_settings
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(settings)
    }

    /// Update user settings
    pub async fn update_settings(
        pool: &PgPool,
        user_id: Uuid,
        updates: UpdateUserSettings,
    ) -> Result<UserSettingsRecord> {
        let settings = sqlx::query_as::<_, UserSettingsRecord>(
            r#"
            UPDATE user_settings SET
                weight_unit = COALESCE($2, weight_unit),
                distance_unit = COALESCE($3, distance_unit),
                energy_unit = COALESCE($4, energy_unit),
                timezone = COALESCE($5, timezone),
                daily_calorie_goal = COALESCE($6, daily_calorie_goal),
                daily_water_goal_ml = COALESCE($7, daily_water_goal_ml),
                daily_step_goal = COALESCE($8, daily_step_goal),
                height_cm = COALESCE($9, height_cm),
                date_of_birth = COALESCE($10, date_of_birth),
                biological_sex = COALESCE($11, biological_sex),
                activity_level = COALESCE($12, activity_level),
                height_unit = COALESCE($13, height_unit),
                temperature_unit = COALESCE($14, temperature_unit),
                updated_at = NOW()
            WHERE user_id = $1
            RETURNING user_id, weight_unit, distance_unit, energy_unit, timezone,
                      daily_calorie_goal, daily_water_goal_ml, daily_step_goal,
                      height_cm, date_of_birth, biological_sex, activity_level,
                      height_unit, temperature_unit, updated_at
            "#,
        )
        .bind(user_id)
        .bind(updates.weight_unit)
        .bind(updates.distance_unit)
        .bind(updates.energy_unit)
        .bind(updates.timezone)
        .bind(updates.daily_calorie_goal)
        .bind(updates.daily_water_goal_ml)
        .bind(updates.daily_step_goal)
        .bind(updates.height_cm)
        .bind(updates.date_of_birth)
        .bind(updates.biological_sex)
        .bind(updates.activity_level)
        .bind(updates.height_unit)
        .bind(updates.temperature_unit)
        .fetch_one(pool)
        .await?;

        Ok(settings)
    }

    /// Check if email exists
    pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)
            "#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require database - marked with #[ignore]
    // Run with: cargo test --features integration -- --ignored
}
