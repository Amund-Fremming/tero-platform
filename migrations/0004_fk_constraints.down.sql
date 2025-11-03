-- Remove foreign key constraints from saved_game table
ALTER TABLE "saved_game"
DROP CONSTRAINT fk_saved_game_user_id;

ALTER TABLE "saved_game"
DROP CONSTRAINT fk_saved_game_base_id;