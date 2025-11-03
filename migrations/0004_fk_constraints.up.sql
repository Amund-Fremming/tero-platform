-- Add foreign key constraints to saved_game table
ALTER TABLE "saved_game"
ADD CONSTRAINT fk_saved_game_user_id
FOREIGN KEY (user_id) REFERENCES "base_user"(id) ON DELETE CASCADE;

ALTER TABLE "saved_game"
ADD CONSTRAINT fk_saved_game_base_id
FOREIGN KEY (base_id) REFERENCES "game_base"(id) ON DELETE CASCADE;