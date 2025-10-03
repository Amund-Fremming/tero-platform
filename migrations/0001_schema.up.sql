-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE "integration_name" AS ENUM (
    'auth0',
    'session'
);

CREATE TYPE "log_ceverity" AS ENUM (
    'critical',
    'warning',
    'info'
);

CREATE TYPE "log_action" AS ENUM (
    'create',
    'read',
    'update',
    'delete'
);

CREATE TYPE "subject_type" AS ENUM (
    'registered_user',
    'guest_user',
    'integration',
    'system'
);

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

CREATE TABLE "system_log" (
    "id" BIGSERIAL PRIMARY KEY,
    "subject_id" VARCHAR(100) NOT NULL,
    "subject_type" subject_type NOT NULL,
    "action" log_action NOT NULL,
    "ceverity" log_ceverity NOT NULL,
    "function" VARCHAR(50) NOT NULL,
    "description" VARCHAR(512) NOT NULL,
    "metadata" JSONB,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE "join_key" (
    "id" PRIMARY KEY VARCHAR(7),
    "name" VARCHAR(4) NOT NULL
)

CREATE TABLE "integration" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "subject" VARCHAR(40) NOT NULL,
    "name" VARCHAR(30) NOT NULL
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

CREATE INDEX "idx_join_key_id" ON "join_key" ("id");

CREATE INDEX "idx_system_log_ceverity" ON "system_log" ("ceverity");;

CREATE INDEX "idx_user_id" ON "user" ("id");
CREATE INDEX "idx_user_auth0_id" ON "user" ("auth0_id");
CREATE INDEX "idx_user_last_active" ON "user" ("last_active");
CREATE INDEX "idx_user_keys" ON "user" ("id", "auth0_id", "guest_id");
CREATE INDEX "idx_auth0_id" ON "user" ("guest_id");

CREATE INDEX "idx_quiz_game_id" ON "quiz_game" ("id");
CREATE INDEX "idx_quiz_game_category" ON "quiz_game" ("category");

CREATE INDEX "idx_spin_game_id" ON "spin_game" ("id");
CREATE INDEX "idx_spin_game_category" ON "spin_game" ("category");
CREATE INDEX "idx_spin_game_round_id" ON "spin_game_round" ("id");