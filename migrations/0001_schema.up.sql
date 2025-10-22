-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE "game_type" AS ENUM (
    'spin',
    'quiz'
);

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
    'ladies',
    'boys',
    'default',
    'random'
);

CREATE TYPE gender AS ENUM (
    'm',
    'f',
    'u'   
);

CREATE TABLE "saved_game" (
    "id" UUID PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "base_id" UUID NOT NULL,
    "game_id" UUID NOT NULL,
    "game_type" game_type NOT NULL,
    UNIQUE ("base_id", "game_id")
);

CREATE TABLE "system_log" (
    "id" BIGSERIAL PRIMARY KEY,
    "subject_id" VARCHAR(100) NOT NULL,
    "subject_type" subject_type NOT NULL,
    "action" log_action NOT NULL,
    "ceverity" log_ceverity NOT NULL,
    "file_name" VARCHAR(50) NOT NULL,
    "description" VARCHAR(512) NOT NULL,
    "metadata" JSONB,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE "prefix_word" (
    "word" VARCHAR(5) PRIMARY KEY
);

CREATE TABLE "suffix_word" (
    "word" VARCHAR(5) PRIMARY KEY
);

CREATE TABLE "integration" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "subject" VARCHAR(40) NOT NULL,
    "name" integration_name NOT NULL
);

CREATE TABLE "user" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "username" VARCHAR (100) NOT NULL,
    "auth0_id" VARCHAR,
    "guest_id" UUID,
    "user_type" user_type NOT NULL DEFAULT 'guest',
    "last_active" TIMESTAMPTZ NOT NULL DEFAULT now(),
    "birth_date" DATE,
    "gender" gender NOT NULL DEFAULT 'u',
    "email" VARCHAR(150),
    "email_verified" BOOLEAN,
    "family_name" VARCHAR(100),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "given_name" VARCHAR(100),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE "game_base" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "name" VARCHAR(100) NOT NULL,
    "description" VARCHAR(150),
    "game_type" game_type NOT NULL,
    "category" game_category NOT NULL DEFAULT 'casual',
    "iterations" INTEGER NOT NULL DEFAULT 0,
    "times_played" INTEGER NOT NULL DEFAULT 0,
    "last_played" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE "quiz_game" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "base_id" UUID NOT NULL,
    "questions" TEXT[] NOT NULL
);

CREATE TABLE "spin_game" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "base_id" UUID NOT NULL,
    "rounds" TEXT[] NOT NULL
);

CREATE INDEX "idx_saved_game_id" ON "saved_game" ("id");
CREATE INDEX "idx_saved_game_delete_keys" ON "saved_game" ("id", "user_id");

CREATE INDEX "idx_system_log_ceverity" ON "system_log" ("ceverity", "created_at" DESC);

CREATE INDEX "idx_integration_subject" ON "integration" ("subject");

CREATE INDEX "idx_quiz_game_id" ON "quiz_game" ("id");

CREATE INDEX "idx_spin_game_id" ON "spin_game" ("id");

CREATE INDEX "idx_game_base_id" ON "game_base" ("id");
CREATE INDEX "idx_game_base_game_type" ON "game_base" ("game_type", "times_played" DESC);
CREATE INDEX "idx_game_base_type_and_category" ON "game_base" ("game_type", "category", "times_played" DESC);

CREATE INDEX "idx_user_id" ON "user" ("id");
CREATE INDEX "idx_user_auth0_id" ON "user" ("auth0_id");
CREATE INDEX "idx_user_guest_id" ON "user" ("guest_id");
CREATE INDEX "idx_user_last_active" ON "user" ("last_active" DESC);
CREATE INDEX "idx_user_keys" ON "user" ("id", "auth0_id", "guest_id");


ALTER TABLE "saved_game" ADD CONSTRAINT "fk_saved_game_user" 
    FOREIGN KEY ("user_id") REFERENCES "user"("id") ON DELETE CASCADE;

ALTER TABLE "saved_game" ADD CONSTRAINT "fk_saved_game_base" 
    FOREIGN KEY ("base_id") REFERENCES "game_base"("id") ON DELETE CASCADE;

ALTER TABLE "quiz_game" ADD CONSTRAINT "fk_quiz_game_base" 
    FOREIGN KEY ("base_id") REFERENCES "game_base"("id") ON DELETE CASCADE;

ALTER TABLE "spin_game" ADD CONSTRAINT "fk_spin_game_base" 
    FOREIGN KEY ("base_id") REFERENCES "game_base"("id") ON DELETE CASCADE;