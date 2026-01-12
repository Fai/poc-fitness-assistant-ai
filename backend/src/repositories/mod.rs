//! Database repositories
//!
//! Provides data access layer for database operations.

pub mod biometrics;
pub mod biomarkers;
pub mod exercise;
pub mod goals;
pub mod hydration;
pub mod nutrition;
pub mod sleep;
pub mod user;
pub mod weight;

pub use biometrics::{
    CreateHeartRateLog, CreateHrvLog, HeartRateLogRecord, HeartRateLogRepository,
    HeartRateZonesRecord, HeartRateZonesRepository, HrvLogRecord, HrvLogRepository,
    UpsertHeartRateZones,
};
pub use biomarkers::{
    BiomarkerLogRepository, BiomarkerLogWithRange, BiomarkerRangeRecord, BiomarkerRangeRepository,
    CreateBiomarkerLog, CreateSupplement, CreateSupplementLog, SupplementLogRepository,
    SupplementRecord, SupplementRepository,
};
pub use exercise::{
    AddWorkoutExercise, CreateExercise, CreateExerciseSet, CreateWorkout, ExerciseRecord,
    ExerciseRepository, ExerciseSetRecord, ExerciseSetRepository, WorkoutExerciseRecord,
    WorkoutExerciseRepository, WorkoutRecord, WorkoutRepository,
};
pub use goals::{
    CreateGoal, CreateMilestone, GoalRecord, GoalRepository, MilestoneRecord,
    MilestoneRepository, UpdateGoal,
};
pub use hydration::{
    CreateHydrationLog, DailyHydrationSummary, HydrationGoalRecord, HydrationGoalRepository,
    HydrationLogRecord, HydrationLogRepository, UpsertHydrationGoal,
};
pub use nutrition::{
    AddRecipeIngredient, CreateFoodItem, CreateFoodLog, CreateRecipe, DailyNutritionSummary,
    FoodItem, FoodItemRepository, FoodLog, FoodLogRepository, Recipe, RecipeIngredient,
    RecipeRepository,
};
pub use sleep::{
    CreateSleepLog, SleepGoalRecord, SleepGoalRepository, SleepLogRecord, SleepLogRepository,
    SleepSummary, UpsertSleepGoal,
};
pub use user::{UpdateUserSettings, UserRepository};
pub use weight::{
    BodyCompositionRepository, CreateBodyCompositionLog, CreateWeightLog, WeightRepository,
};
