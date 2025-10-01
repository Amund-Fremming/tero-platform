# Tasklist

## Quick notes

- Split webhook mw to its own middleware
- 

- read before spin blir random, mindre for bruker å velge
- Persist gamesession needs to be protected. Make singalR hub a integration, validate M2M Token
- add write system log permission

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
- [ ] Expand refresh token / jwt to be longer than an hour
- [ ] Update user sync from backend to auth0 (daily job/trigger)
- [ ] Cron job for deleting guest users after a time, then make the frontend users need to create new. This is to prevent having to link users to guest users
- [ ] Sync for when a user gets admin permissions, needs to update user type

**M2M GameSession**
- [ ] Create M2M support for gamesession
- [ ] Support new Subject Integration
- [ ] Create auth0 cached client for getting token from c#
- [ ] Create and give out permissions

**Cache**
- [x] Implement a generic cache wrapper and implementation for DRY principle for future games and caches
- [x] Implement a generic cache for game search pages
- [x] Expand search cache to support passing in functions to handle when its a cache miss
- [x] Move cache out in its own reusable crate for future use
- [ ] Change ttl to use UNIC EPOCH not datetime
- [ ] Tests to verify that the cache works

**Generic feature**
- [x] Typed search in a handler
- [x] GenericGameService with GetGame, Typed Search

**SignalR microservice**
- [x] Create C# project with signal installed
- [ ] Create a http client for talking to rust
- [ ] Create api in c# for consuming games from rust
- [ ] Create or add a cache solution for storing game sessions
- [ ] Add core game logic in c# project

**Rust connection to microservice**
- [x] Create api in rust for consuming created games in db
- [x] Create client for talking to C#
- [x] Api for storing games to database from c#
- [x] Api for creation of game, send to c# and client
- [x] Api for game session creation, send to c# and client

**UniversalService**
- [x] Pagination support
- [ ] typed search by game, category, page and most played
- [ ] Universal join game

**KeyVault**
- [x] Setup index and tables
- [x] Setup db handlers
- [x] Implement core
- [x] strategy for removing no longer used slugs

**Admin**
- [ ] Delete games
- [ ] Endponints for user history, how many active last week, last month and today
- [ ] Endpoints for fetching logs based on time or ceverity
- [ ] Possibility to view config like redirect (for debugging)

**Store games**
- [ ] Model relations table for a registered user to persist games they have played
- [ ] Endpoint for persisting a game
- [ ] Endpoint for listing a game

**Audit**
- [x] Enums for action and ceverity
- [x] Implement and SQL migration
- [x] Import integrations on startup to INTEGRAITON_IDS and INTEGRATION_NAMES
- [x] Add audit logs where neccesarry

**Consents**
- [ ] Make it a static table / json file loaded from startup
- [ ] Use a bitmap for storing consents on the user profile rather than a own table for lookups (No need for realations and joins)
- [ ] Push notifications/alterts/mail?/sms?

**Notifications**
- [ ] Model a solution for storing alerts
- [ ] Remove notifications after some time to store data storage
- [ ] endpoint for admins to create alerts

**Cleanup/refactor**
- [ ] Better handling for ServerErrors (Rows not affected, cache error)
