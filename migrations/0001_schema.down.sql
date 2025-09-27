-- Add migration script here

-- Drop constraints
ALTER TABLE "spin_game_round" DROP CONSTRAINT "spin_game_round_fk";

-- Drop indexes
DROP INDEX IF EXISTS "idx_join_key_id";

DROP INDEX IF EXISTS "idx_user_id";
DROP INDEX IF EXISTS "idx_auth0_id";

DROP INDEX IF EXISTS "idx_quiz_game_id";
DROP INDEX IF EXISTS "idx_quiz_game_category";

DROP INDEX IF EXISTS "idx_spin_game_id";
DROP INDEX IF EXISTS "idx_spin_game_category";
DROP INDEX IF EXISTS "idx_spin_game_round_id";

-- Drop tables
DROP TABLE IF EXISTS "integration";
DROP TABLE IF EXISTS "join_key";
DROP TABLE IF EXISTS "user";
DROP TABLE IF EXISTS "quiz_game";
DROP TABLE IF EXISTS "spin_game";
DROP TABLE IF EXISTS "spin_game_round";

-- Drop types
DROP TYPE IF EXISTS "user_type";
DROP TYPE IF EXISTS "game_category";
DROP TYPE IF EXISTS "gender";