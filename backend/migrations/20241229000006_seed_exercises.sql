-- Seed common exercises for the exercise library
-- These are standard exercises that all users can access

-- Strength - Chest
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Bench Press', 'strength', ARRAY['chest', 'triceps', 'shoulders'], 'barbell', 8.0, 'Compound chest exercise using a barbell', false),
    ('Incline Bench Press', 'strength', ARRAY['chest', 'shoulders', 'triceps'], 'barbell', 8.0, 'Upper chest focused press on incline bench', false),
    ('Dumbbell Fly', 'strength', ARRAY['chest'], 'dumbbells', 6.0, 'Isolation exercise for chest', false),
    ('Push-Up', 'strength', ARRAY['chest', 'triceps', 'shoulders'], NULL, 7.0, 'Bodyweight chest exercise', false),
    ('Cable Crossover', 'strength', ARRAY['chest'], 'cable', 5.0, 'Cable isolation for chest', false)
ON CONFLICT DO NOTHING;

-- Strength - Back
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Deadlift', 'strength', ARRAY['back', 'hamstrings', 'glutes'], 'barbell', 10.0, 'Compound posterior chain exercise', false),
    ('Barbell Row', 'strength', ARRAY['back', 'biceps'], 'barbell', 7.0, 'Compound back exercise', false),
    ('Pull-Up', 'strength', ARRAY['back', 'biceps'], NULL, 8.0, 'Bodyweight back exercise', false),
    ('Lat Pulldown', 'strength', ARRAY['back', 'biceps'], 'cable', 6.0, 'Machine back exercise', false),
    ('Seated Cable Row', 'strength', ARRAY['back', 'biceps'], 'cable', 6.0, 'Cable back exercise', false)
ON CONFLICT DO NOTHING;

-- Strength - Legs
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Squat', 'strength', ARRAY['quadriceps', 'glutes', 'hamstrings'], 'barbell', 9.0, 'Compound leg exercise', false),
    ('Leg Press', 'strength', ARRAY['quadriceps', 'glutes'], 'machine', 7.0, 'Machine leg exercise', false),
    ('Romanian Deadlift', 'strength', ARRAY['hamstrings', 'glutes', 'back'], 'barbell', 8.0, 'Hamstring focused deadlift variation', false),
    ('Leg Curl', 'strength', ARRAY['hamstrings'], 'machine', 5.0, 'Isolation hamstring exercise', false),
    ('Leg Extension', 'strength', ARRAY['quadriceps'], 'machine', 5.0, 'Isolation quadriceps exercise', false),
    ('Calf Raise', 'strength', ARRAY['calves'], 'machine', 4.0, 'Calf isolation exercise', false),
    ('Lunge', 'strength', ARRAY['quadriceps', 'glutes', 'hamstrings'], 'dumbbells', 7.0, 'Unilateral leg exercise', false)
ON CONFLICT DO NOTHING;

-- Strength - Shoulders
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Overhead Press', 'strength', ARRAY['shoulders', 'triceps'], 'barbell', 7.0, 'Compound shoulder exercise', false),
    ('Lateral Raise', 'strength', ARRAY['shoulders'], 'dumbbells', 5.0, 'Isolation for lateral deltoids', false),
    ('Front Raise', 'strength', ARRAY['shoulders'], 'dumbbells', 5.0, 'Isolation for front deltoids', false),
    ('Face Pull', 'strength', ARRAY['shoulders', 'back'], 'cable', 5.0, 'Rear deltoid and upper back exercise', false)
ON CONFLICT DO NOTHING;

-- Strength - Arms
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Bicep Curl', 'strength', ARRAY['biceps'], 'dumbbells', 5.0, 'Isolation bicep exercise', false),
    ('Tricep Pushdown', 'strength', ARRAY['triceps'], 'cable', 5.0, 'Isolation tricep exercise', false),
    ('Hammer Curl', 'strength', ARRAY['biceps', 'forearms'], 'dumbbells', 5.0, 'Bicep and forearm exercise', false),
    ('Skull Crusher', 'strength', ARRAY['triceps'], 'barbell', 5.0, 'Tricep isolation exercise', false),
    ('Dip', 'strength', ARRAY['triceps', 'chest', 'shoulders'], NULL, 7.0, 'Compound arm exercise', false)
ON CONFLICT DO NOTHING;

-- Strength - Core
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Plank', 'strength', ARRAY['core', 'shoulders'], NULL, 4.0, 'Isometric core exercise', false),
    ('Crunch', 'strength', ARRAY['core'], NULL, 5.0, 'Basic abdominal exercise', false),
    ('Russian Twist', 'strength', ARRAY['core', 'obliques'], NULL, 6.0, 'Rotational core exercise', false),
    ('Hanging Leg Raise', 'strength', ARRAY['core', 'hip_flexors'], NULL, 6.0, 'Advanced core exercise', false),
    ('Ab Wheel Rollout', 'strength', ARRAY['core'], 'ab_wheel', 7.0, 'Advanced core exercise', false)
ON CONFLICT DO NOTHING;

-- Cardio
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Running', 'cardio', ARRAY['legs', 'cardiovascular'], NULL, 11.0, 'Outdoor or treadmill running', false),
    ('Cycling', 'cardio', ARRAY['legs', 'cardiovascular'], 'bike', 8.0, 'Outdoor or stationary cycling', false),
    ('Swimming', 'cardio', ARRAY['full_body', 'cardiovascular'], NULL, 10.0, 'Full body cardio exercise', false),
    ('Rowing', 'cardio', ARRAY['back', 'legs', 'cardiovascular'], 'rowing_machine', 9.0, 'Full body cardio on rowing machine', false),
    ('Jump Rope', 'cardio', ARRAY['legs', 'cardiovascular'], 'jump_rope', 12.0, 'High intensity cardio', false),
    ('Elliptical', 'cardio', ARRAY['legs', 'cardiovascular'], 'elliptical', 7.0, 'Low impact cardio machine', false),
    ('Stair Climber', 'cardio', ARRAY['legs', 'glutes', 'cardiovascular'], 'stair_climber', 9.0, 'Cardio machine simulating stairs', false),
    ('Walking', 'cardio', ARRAY['legs', 'cardiovascular'], NULL, 4.0, 'Low intensity cardio', false)
ON CONFLICT DO NOTHING;

-- Flexibility/Mobility
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Yoga', 'flexibility', ARRAY['full_body'], NULL, 3.0, 'Mind-body practice with poses', false),
    ('Stretching', 'flexibility', ARRAY['full_body'], NULL, 2.0, 'General flexibility work', false),
    ('Foam Rolling', 'flexibility', ARRAY['full_body'], 'foam_roller', 2.0, 'Self-myofascial release', false)
ON CONFLICT DO NOTHING;

-- HIIT
INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, description, is_custom)
VALUES 
    ('Burpee', 'hiit', ARRAY['full_body', 'cardiovascular'], NULL, 12.0, 'Full body explosive exercise', false),
    ('Mountain Climber', 'hiit', ARRAY['core', 'cardiovascular'], NULL, 10.0, 'Core and cardio exercise', false),
    ('Box Jump', 'hiit', ARRAY['legs', 'cardiovascular'], 'box', 10.0, 'Plyometric leg exercise', false),
    ('Kettlebell Swing', 'hiit', ARRAY['glutes', 'hamstrings', 'back'], 'kettlebell', 11.0, 'Explosive hip hinge exercise', false),
    ('Battle Ropes', 'hiit', ARRAY['arms', 'shoulders', 'cardiovascular'], 'battle_ropes', 12.0, 'Upper body cardio exercise', false)
ON CONFLICT DO NOTHING;
