-- Add migration script here
ALTER TABLE "spin_game_player" DROP CONSTRAINT "spinner_player_user_fk";
ALTER TABLE "spinner_player" DROP CONSTRAINT "spinner_player_fk";
ALTER TABLE "round" DROP CONSTRAINT "spinner_round";
ALTER TABLE "question" DROP CONSTRAINT "quiz_question";

DROP INDEX IF EXISTS "idx_spinner_id";
DROP INDEX IF EXISTS "idx_round_id";
DROP INDEX IF EXISTS "idx_spinner_category";
DROP INDEX IF EXISTS "idx_quiz_category";
DROP INDEX IF EXISTS "idx_guest_user_id";

DROP TABLE IF EXISTS "spinner_player";
DROP TABLE IF EXISTS "round";
DROP TABLE IF EXISTS "spinner";
DROP TABLE IF EXISTS "question";
DROP TABLE IF EXISTS "quiz";
DROP TABLE IF EXISTS "user";

DROP TYPE IF EXISTS "game_category";
DROP TYPE IF EXISTS "user_type";