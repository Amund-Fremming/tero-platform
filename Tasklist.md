# Tasklist

## Quick notes

- Persist gamesession needs to be protected. Make singalR hub a integration, validate M2M Token
- Index on auth0 id on user

## Thoughts

- Cleanup ids: - some are uuid some are i32, and fix indexes to these if they are uuid
- Cleanup user id: - user has many ids now, decide on one sort.
- Split the fn get_subject_and_permissins into two fn, SRP

## Tasklist

**Setup**
- [x] Init github
- [x] Update the old repos readme: - rewrite in rust
- [x] Setup dev env with docker compose
- [x] Setup basic api for health and health detailed
- [x] Setup tracing
- [x] Basic middleware request logger (may not need after some time, but good for debugging)
- [x] Map out all models
- [ ] Setup runtimes with .env files GITIGNORE, maybe also runtimmes for auth0, so we have separate triggers for different environments. Less pain to work with

**State**
- [x] Implement state with pg pool

**Error**
- [x] Implement descriptive error handling with internal logging not visible to the outside
- [x] Implement IntoResponse for all errors for the ServerError

**Auth0**
- [x] App (fe) application setup
- [x] API (be) setup
- [x] Add permissions

**User/Auth**
- [x] Add support for guest user and persistet user
- [x] Create middleware for injecting an extention for user
- [x] Post, put, delete
- [x] Put endpoint for updating last active
- [x] Auth0 webhook for users
- [x] Implement peristent storage for webhook api
- [x] Permissions extention
- [x] List all users (admin access)
- [x] Decode and validate tokens
- [x] Permission checks for endpoints
- [x] Maybe update endpoints to require user id for fetching users, targeting query on id, not auth0_id or guest_id. this also makes it possible for admins to query users 
- [ ] Update user sync from backend to auth0 (daily job/trigger)
- [ ] Sync on registered user creation, needs to deactivate/delete the guest user
- [ ] Sync for when a user gets admin permissions, needs to update user type

**Cache**
- [x] Implement a generic cache wrapper and implementation for DRY principle for future games and caches
- [x] Implement a generic cache for game search pages
- [x] Expand search cache to support passing in functions to handle when its a cache miss
- [x] Move cache out in its own reusable crate for future use
- [ ] Tests to verify that the cache works

**Generic feature**
- [x] Typed search in a handler
- [x] GenericGameService with GetGame, Typed Search

**SignalR microservice**
- [ ] Create C# project with signal installed
- [ ] Create a http client for talking to rust
- [ ] Create api in c# for consuming games from rust
- [ ] Create or add a cache solution for storing game sessions
- [ ] Add core game logic in c# project

**Rust connection to microservice**
- [ ] Create api in rust for consuming created games in db
- [ ] Create client for talking to C#
- [ ] Api for storing games to database from c#
- [ ] Api for creation of game, send to c# and client
- [ ] Api for game session creation, send to c# and client

**UniversalService**
- [ ] Pagination support
- [ ] Typed search for all games
- [ ] Universal join game

**Admin**
- [ ] Endponints for user history, how many active last week, last month and today
- [ ] Endpoints for fetching logs based on time or ceverity?

**Audit**
- [ ] Enums for action and ceverity
- [ ] Implement and SQL migration
- [ ] Add audit logs where neccesarry

**Consents**
- [ ] Make it a static table / json file loaded from startup
- [ ] Use a bitmap for storing consents on the user profile rather than a own table for lookups (No need for realations and joins)
- [ ] Push notifications/alterts/mail?/sms?

**Cleanup/refactor**
- [ ] Better handling for ServerErrors (Rows not affected, cache error)

---

## Models not implemented

- audit
    - severity
    - action
    - trace ?

- saved_games 
    - id (PK)
    - user_id (FK)
    - game_id (FK)
    - game_type (enum: spinner, quiz, ...)
    - saved