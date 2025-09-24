use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::games::models::Gender;

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub enum Permission {
    #[serde(rename(deserialize = "read:admin"))]
    ReadAdmin,
    #[serde(rename(deserialize = "write:admin"))]
    WriteAdmin,
    #[serde(rename(deserialize = "save:games"))]
    SaveGames,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionCtx {
    permissions: HashSet<Permission>,
}

impl PermissionCtx {
    pub fn none() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    pub fn new(permissions: HashSet<Permission>) -> Self {
        Self { permissions }
    }

    pub fn has(&self, required_perm: Permission) -> bool {
        self.permissions.contains(&required_perm)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    aud: Vec<String>,
    azp: String,
    exp: i32,
    iat: i32,
    iss: String,
    pub scope: String,
    pub sub: String,
    pub permissions: HashSet<Permission>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubjectId {
    Guest(Uuid),
    Registered(String),
    Auth0,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0User {
    pub auth0_id: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub phone: Option<String>,
    pub phone_verified: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_type", rename_all = "lowercase")]
pub enum UserType {
    #[serde(rename(deserialize = "guest"))]
    Guest,
    #[serde(rename(deserialize = "registered"))]
    Registered,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub auth0_id: Option<String>,
    pub user_type: UserType,
    pub last_active: DateTime<Utc>,
    pub gender: Gender,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub updated_at: DateTime<Utc>,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub birth_date: Option<NaiveDate>,
}

impl User {
    pub fn strip(&self) -> Self {
        Self {
            id: self.id,
            auth0_id: None,
            user_type: UserType::Guest,
            last_active: Utc::now(),
            gender: Gender::Unknown,
            updated_at: Utc::now(),
            created_at: self.created_at,
            given_name: self.given_name.clone(),
            family_name: None,
            email: None,
            email_verified: None,
            birth_date: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrippedUser {
    pub id: Uuid,
    pub given_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub birth_date: Option<NaiveDate>,
}
