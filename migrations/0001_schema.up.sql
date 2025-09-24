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

CREATE TYPE gender AS ENUM (
    'm',
    'f',
    'u'   
)

CREATE TABLE "game_name" (
    "id" PRIMARY KEY SERIAL,
    "name" VARCHAR(20) NOT NULL,
    "in_use" BOOLEAN NOT NULL
)

CREATE TABLE "user" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "auth0_id" VARCHAR,
    "user_type" user_type NOT NULL DEFAULT 'guest',
    "last_active" TIMESTAMPTZ NOT NULL DEFAULT now(),
    "birth_date" DATE,
    "gender" gender,
    "email" VARCHAR(150),
    "email_verified" BOOLEAN,
    "family_name" VARCHAR(100),
    "updated_at" TIMESTAMPTZ,
    "given_name" VARCHAR(100),
    "created_at" TIMESTAMPTZ
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

CREATE INDEX "idx_game_name_in_use" ON "game_name"("name", "in_use")

CREATE INDEX "idx_user_id" ON "user" ("id");
CREATE INDEX "idx_auth0_id" ON "user" ("guest_id");

CREATE INDEX "idx_quiz_game_id" ON "quiz_game" ("id");
CREATE INDEX "idx_quiz_game_category" ON "quiz_game" ("category");

CREATE INDEX "idx_spin_game_id" ON "spin_game" ("id");
CREATE INDEX "idx_spin_game_category" ON "spin_game" ("category");
CREATE INDEX "idx_spin_game_round_id" ON "spin_game_round" ("id");