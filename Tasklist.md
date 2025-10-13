# Tasklist

## For FE

- Possibility to view redirect config (for debugging)
- Read before spin needs to be random or not visible to the user, to much to click on

## Tasklist

**Setup**
- [x] Init github
- [x] Update the old repos readme: - rewrite in rust
- [x] Setup dev env with docker compose
- [x] Setup basic api for health and health detailed
- [x] Setup tracing
- [x] Basic middleware request logger (may not need after some time, but good for debugging)
- [x] Map out all models
- [x] Centrallized config management
- [x] Setup runtimes with .env files GITIGNORE
- [x] Better config management
- [x] Run migrations on startup

**State**
- [x] pg pool
- [x] page cache

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
- [x] Create guest user
- [x] Is valid token endpoint (also serves as user sync)
- [x] User sync

**M2M GameSession**
- [x] Create M2M support for gamesession
- [x] Support new Subject Integration
- [x] Create and give out permissions

**Generic feature**
- [x] Typed search in a handler
- [x] GenericGameService with GetGame, Typed Search

**Rust connection to microservice**
- [x] Create api in rust for consuming created games in db
- [x] Create client for talking to C#
- [x] Api for storing games to database from c#
- [x] Api for creation of game, send to c# and client
- [x] Api for game session creation, send to c# and client
- [x] Join game fn that needs to validate that a user can join a game before getting the url to connect

**Universal Service**
- [x] Pagination support
- [x] Typed search by game, category
- [x] Universal join game
- [x] Cron job for deleting games that is not longer played

**KeyVault**
- [x] Setup index and tables
- [x] Setup db handlers
- [x] Implement core
- [x] Strategy for removing no longer used slugs
- [x] Strategy for cron job, could be errors that make keys stay forever

**Store games**
- [x] Model relations table for a registered user to persist games they have played
- [x] Endpoint for persisting a game
- [x] Endpoint for listing games
- [x] Endpoint for removing relation

---

**System log**
- [x] Enums for action and ceverity
- [x] Implement and SQL migration
- [x] Import integrations on startup to INTEGRAITON_IDS and INTEGRATION_NAMES
- [x] api for gettings logs by filters/pagination
- [ ] Add logs where neccesarry

**Admin**
- [x] Delete games
- [x] Endponints for user history, how many active last week, last month and today
- [ ] Read config endpoint for debugging 

**Cache**
- [x] Implement a generic cache wrapper and implementation for DRY principle for future games and caches
- [x] Implement a generic cache for game search pages
- [x] Expand search cache to support passing in functions to handle when its a cache miss
- [x] Move cache out in its own reusable crate for future use
- [ ] Change datetime to use secs from UNIC EPOCH
- [ ] Use dashmap not hashset and locks
- [ ] Verify that cache works

**SignalR microservice**
- [x] Create C# project with signal installed
- [ ] Create a http client for talking to rust
- [ ] Create api in c# for consuming games from rust
- [ ] Create or add a cache solution for storing game sessions
- [ ] Create auth0 cached client for getting token from c#
- [ ] Add core game logic in c# project

**Notifications**
- [ ] Model a solution for storing alerts
- [ ] Remove notifications after some time to store data storage
- [ ] endpoint for admins to create alerts

**Cleanup/refactor**
- [ ] Change admin routes to own router and files (maybe also for auth and user?)
- [ ] Change webhook to use event streams from auth0, and handle events
- [ ] Expand refresh token / jwt to be longer than an hour
- [ ] Error handling for client, game full/game does not exist ..
- [ ] Optimize db queries by doing with tokio joins
- [ ] Dynamic query builders, make a service that does this with builder pattern, now its super ugly everywhere, and hard to read
- [ ] Some generic code for paginated queries
- [ ] Better handling for ServerErrors (Rows not affected, cache error)
- [ ] Split migrations for better overview
- [ ] Go over indexes and optimize
- [ ] Create relations where possible
- [ ] Cascades
    - if a game is deleted, games and base and saved relations need to be deleted
