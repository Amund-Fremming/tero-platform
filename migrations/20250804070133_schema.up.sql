-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_type AS ENUM (
    'guest',
    'registered'
);

CREATE TYPE game_category AS ENUM (
    'casual',
    'random'
    'ladies',
    'boys'
);

CREATE TABLE "user" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "auth0_id" VARCHAR,
    "user_type" user_type NOT NULL DEFAULT 'guest',
    "last_active" TIMESTAMPTZ NOT NULL DEFAULT now(),
    "name" VARCHAR(100),
    "email" VARCHAR(150),
    "birth_date" DATE
);

CREATE TABLE "quiz_game" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "name" VARCHAR(100) NOT NULL,
    "description" VARCHAR(150),
    "category" game_category NOT NULL DEFAULT 'casual',
    "iterations" INTEGER NOT NULL DEFAULT 0,
    "times_played" INTEGER NOT NULL DEFAULT 0,
    "questions" TEXT[] NOT NULL
);

CREATE TABLE "spin_game" (
    "id" UUID PRIMARY KEY,
    "host_id" UUID NOT NULL,
    "name" VARCHAR(100) NOT NULL,
    "description" VARCHAR(150),
    "category" game_category NOT NULL DEFAULT 'casual',
    "iterations" INTEGER NOT NULL DEFAULT 0,
    "times_played" INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE "spin_game_round" (
    "id" UUID PRIMARY KEY,
    "spin_game_id" UUID NOT NULL,
    "participants" INTEGER NOT NULL DEFAULT 0,
    "content" VARCHAR(200)
);

ALTER TABLE "spin_game_round" ADD CONSTRAINT "spin_game_round_fk" FOREIGN KEY ("spin_game_id") REFERENCES "spin_game" ("id");
ALTER TABLE "spin_game_player" ADD CONSTRAINT "spin_game_player_fk" FOREIGN KEY ("spin_game_id") REFERENCES "spin_game" ("id");
ALTER TABLE "spin_game_player" ADD CONSTRAINT "spin_player_user_fk" FOREIGN KEY ("user_id") REFERENCES "user" ("id");

CREATE INDEX "idx_guest_id" ON "user" ("guest_id");
CREATE INDEX "idx_auth0_id" ON "user" ("guest_id");
CREATE INDEX "idx_quiz_category" ON "quiz_game" ("category");
CREATE INDEX "idx_spin_category" ON "spin_game" ("category");
CREATE INDEX "idx_round_id" ON "spin_game_round" ("id");
CREATE INDEX "idx_spin_id" ON "spin_game" ("id");