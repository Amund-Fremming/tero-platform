# API Endpoint Migration Guide

This document maps the old API endpoints to the new standardized RESTful endpoints. Update your frontend code to use the new URLs.

## General Changes

### Standards Applied
1. **Plural nouns** for resource collections (e.g., `/users`, `/games`, `/logs`)
2. **Consistent HTTP methods**: GET (read), POST (create), PUT/PATCH (update), DELETE (delete)
3. **Kebab-case** for multi-word path segments (e.g., `activity-stats`, `pseudo-users`)
4. **Resource ID before actions** (e.g., `/{game_id}/saved` instead of `/save/{game_id}`)
5. **Removed verbs** from URLs where HTTP methods suffice
6. **Nested resources** logically grouped

---

## Health Endpoints

### ✅ No Changes Required
Health endpoints already follow best practices.

| Method | Old URL | New URL | Notes |
|--------|---------|---------|-------|
| GET | `/health` | `/health` | No change |
| GET | `/health/detailed` | `/health/detailed` | No change |

---

## Pseudo User Endpoints (Public)

These endpoints are for unauthenticated pseudo/guest users.

| Method | Old URL | New URL | Change Description |
|--------|---------|---------|-------------------|
| POST | `/pseudo/ensure` | `/pseudo-users` | Changed path from `/pseudo` to `/pseudo-users`, removed `/ensure` (POST implies creation/ensure) |
| GET | `/pseudo/popup` | `/pseudo-users/popups` | Changed to plural `pseudo-users` and `popups` |

---

## User Endpoints (Protected)

These endpoints require authentication.

| Method | Old URL | New URL | Change Description |
|--------|---------|---------|-------------------|
| GET | `/user` | `/users/me` | Changed to plural `/users`, added `/me` to get current user (more RESTful) |
| GET | `/user/list` | `/users` | Changed to plural `/users`, removed `/list` (GET on collection implies list) |
| GET | `/user/stats` | `/users/activity-stats` | Changed to plural `/users`, renamed to kebab-case `activity-stats` |
| GET | `/user/popup` | `/users/popups` | Changed to plural `/users` and `popups` |
| PUT | `/user/popup` | `/users/popups` | Changed to plural `/users` and `popups` |
| DELETE | `/user/{user_id}` | `/users/{user_id}` | Changed to plural `/users` |
| PATCH | `/user/{user_id}` | `/users/{user_id}` | Changed to plural `/users` |

---

## Game Endpoints (Protected)

All game endpoints require authentication.

### General Game Endpoints

| Method | Old URL | New URL | Change Description |
|--------|---------|---------|-------------------|
| POST | `/game/general/page` | `/games` | Simplified path, removed `/general/page` |
| POST | `/game/general/{game_type}/create` | `/games/{game_type}` | Removed `/general` and `/create` (POST implies creation) |
| DELETE | `/game/general/{game_type}/{game_id}` | `/games/{game_type}/{game_id}` | Removed `/general` |
| PATCH | `/game/general/{game_type}/free-key/{key_word}` | `/games/{game_type}/{game_id}/keys/{key_word}` | Removed `/general`, changed `/free-key` to `/keys` |
| POST | `/game/general/save/{base_id}` | `/games/{game_id}/saved` | Removed `/general`, changed path structure |
| DELETE | `/game/general/unsave/{base_id}` | `/games/{game_id}/saved` | Removed `/general` and `/unsave`, use DELETE on `/saved` |
| POST | `/game/general/saved` | `/games/saved` | Removed `/general`, changed POST to GET |
| GET | `/game/general/saved` | `/games/saved` | Removed `/general`, now uses GET (proper REST method) |

### Standalone Game Endpoints

| Method | Old URL | New URL | Change Description |
|--------|---------|---------|-------------------|
| GET | `/game/static/{game_type}/initiate/{game_id}` | `/games/standalone/{game_type}/{game_id}` | Changed `/static` to `/standalone`, removed `/initiate` (GET implies retrieval) |
| POST | `/game/static/persist` | `/games/standalone/sessions` | Changed `/static` to `/standalone`, `/persist` to `/sessions` (POST implies persistence) |

### Interactive Game Endpoints

| Method | Old URL | New URL | Change Description |
|--------|---------|---------|-------------------|
| POST | `/game/session/persist` | `/games/interactive/sessions` | Changed `/session` to `/interactive`, `/persist` to `/sessions` |
| POST | `/game/session/{game_type}/initiate/{game_id}` | `/games/interactive/{game_type}/{game_id}` | Changed `/session` to `/interactive`, removed `/initiate` |
| POST | `/game/session/{game_type}/join/{game_id}` | `/games/interactive/{game_type}/{game_id}/join` | Changed `/session` to `/interactive` |

---

## System Log Endpoints (Protected)

| Method | Old URL | New URL | Change Description |
|--------|---------|---------|-------------------|
| POST | `/log` | `/logs` | Changed to plural `/logs` |
| GET | `/log` | `/logs` | Changed to plural `/logs` |

---

## Webhook Endpoints (Integration Only)

| Method | Old URL | New URL | Change Description |
|--------|---------|---------|-------------------|
| POST | `/events/create/{pseudo_id}` | `/webhooks/auth0/{pseudo_id}` | Changed `/events` to `/webhooks/auth0`, removed `/create` (POST implies creation) |

---

## Quick Reference: Complete URL Mapping

### Old → New URL Map

```
# Health
/health                                           → /health (no change)
/health/detailed                                  → /health/detailed (no change)

# Pseudo Users
/pseudo/ensure                                    → /pseudo-users
/pseudo/popup                                     → /pseudo-users/popups

# Users
/user                                             → /users/me
/user/list                                        → /users
/user/stats                                       → /users/activity-stats
/user/popup                                       → /users/popups
/user/{user_id}                                   → /users/{user_id}

# Games
/game/general/page                                → /games
/game/general/{game_type}/create                  → /games/{game_type}
/game/general/{game_type}/{game_id}               → /games/{game_type}/{game_id}
/game/general/{game_type}/free-key/{key_word}     → /games/{game_type}/{game_id}/keys/{key_word}
/game/general/save/{base_id}                      → /games/{game_id}/saved (POST)
/game/general/unsave/{base_id}                    → /games/{game_id}/saved (DELETE)
/game/general/saved                               → /games/saved (now GET instead of POST)

# Standalone Games
/game/static/{game_type}/initiate/{game_id}       → /games/standalone/{game_type}/{game_id}
/game/static/persist                              → /games/standalone/sessions

# Interactive Games  
/game/session/persist                             → /games/interactive/sessions
/game/session/{game_type}/initiate/{game_id}      → /games/interactive/{game_type}/{game_id}
/game/session/{game_type}/join/{game_id}          → /games/interactive/{game_type}/{game_id}/join

# Logs
/log                                              → /logs

# Webhooks
/events/create/{pseudo_id}                        → /webhooks/auth0/{pseudo_id}
```

---

## Migration Checklist for Frontend Developers

- [ ] Update all `/pseudo/*` calls to `/pseudo-users/*`
- [ ] Update all `/user/*` calls to `/users/*`
- [ ] Update "get current user" from `GET /user` to `GET /users/me`
- [ ] Update "list users" from `GET /user/list` to `GET /users`
- [ ] Update all `/game/*` calls to `/games/*`
- [ ] Update `/game/general/page` to `/games` (same POST with JSON body)
- [ ] Update save game from `POST /game/general/save/{id}` to `POST /games/{id}/saved`
- [ ] Update unsave game from `DELETE /game/general/unsave/{id}` to `DELETE /games/{id}/saved`
- [ ] Update get saved games from `POST /game/general/saved` to `GET /games/saved` (change to GET with query params)
- [ ] Update `/game/static/*` to `/games/standalone/*`
- [ ] Update `/game/session/*` to `/games/interactive/*`
- [ ] Update all `/log/*` calls to `/logs/*`
- [ ] Update webhook URLs from `/events/*` to `/webhooks/auth0/*`
- [ ] Update `/popup` to `/popups` (plural)
- [ ] Update `/user/stats` to `/users/activity-stats`
- [ ] Update key management from `/free-key/` to `/keys/`

---

## HTTP Method Changes

Pay special attention to these endpoints where the HTTP method changed:

| Endpoint | Old Method | New Method | Reason |
|----------|-----------|------------|---------|
| Get saved games | POST | GET | Retrieving data should use GET, not POST |

---

## Breaking Changes Summary

1. **Path structure**: Most endpoints changed from singular to plural (e.g., `/user` → `/users`)
2. **Naming**: Some multi-word paths now use kebab-case (e.g., `activity-stats`)
3. **Nesting**: Game routes reorganized under `/games` with clearer subpaths
4. **Method changes**: Saved games list now uses GET instead of POST
5. **Action removal**: Removed redundant action verbs like `/create`, `/ensure`, `/initiate` where HTTP method is sufficient

---

## Testing Recommendations

1. Start with health endpoints (no changes) to verify connectivity
2. Update authentication/user endpoints next
3. Update game endpoints last (most complex changes)
4. Use the Quick Reference mapping table for systematic updates
5. Test all HTTP methods on changed endpoints (GET, POST, PATCH, DELETE)

---

## Support

If you encounter any issues with the new endpoints or need clarification on any changes, please refer to the backend API documentation or contact the backend team.

**Migration Date**: 2025-11-18  
**Backend Version**: 0.1.0  
**Breaking Changes**: Yes - all frontend API calls must be updated
