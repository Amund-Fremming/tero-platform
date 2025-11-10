use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{game::models::Gender, integration::models::IntegrationName};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListUsersQuery {
    pub page_num: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestrictedConfig {
    pub auth0_domain: String,
    pub gs_domain: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwks {
    pub keys: [Jwk; 2],
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Jwk {
    pub kid: String,
    pub n: String,
    pub e: String,
    pub kty: String,
    pub alg: String,
    #[serde(rename(deserialize = "use"))]
    pub use_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnsureUserQuery {
    pub pseudo_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub enum Permission {
    #[serde(rename(deserialize = "read:admin"))]
    ReadAdmin,
    #[serde(rename(deserialize = "write:admin"))]
    WriteAdmin,
    #[serde(rename(deserialize = "write:game"))]
    WriteGame,
    #[serde(rename(deserialize = "write:system_log"))]
    WriteSystemLog,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    gty: Option<String>,
    aud: Vec<String>,
    azp: String,
    exp: i32,
    iat: i32,
    iss: String,
    pub scope: String,
    pub sub: String,
    pub permissions: Option<HashSet<Permission>>,
}

impl Claims {
    pub fn empty() -> Self {
        Self {
            gty: None,
            aud: Vec::new(),
            azp: String::new(),
            exp: 0,
            iat: 0,
            iss: String::new(),
            scope: String::new(),
            sub: String::from("guest"),
            permissions: None,
        }
    }

    pub fn is_machine(&self) -> bool {
        self.gty == Some("client-credentials".to_string())
    }

    pub fn auth0_id(&self) -> &str {
        &self.sub
    }

    pub fn missing_permission<I>(&self, required: I) -> Option<HashSet<Permission>>
    where
        I: IntoIterator<Item = Permission>,
    {
        let required_iter = required.into_iter();
        let permissions = match &self.permissions {
            None => return Some(required_iter.collect()),
            Some(perm) => perm,
        };

        let missing: HashSet<Permission> =
            required_iter.filter(|p| !permissions.contains(p)).collect();

        (!missing.is_empty()).then_some(missing)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubjectId {
    PseudoUser(Uuid),
    BaseUser(Uuid),
    Integration(IntegrationName),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0User {
    #[serde(rename = "user_id")]
    pub auth0_id: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub username: Option<String>,
    pub phone_number: Option<String>,
    pub phone_verified: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: Option<String>,
    pub nickname: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_type", rename_all = "lowercase")]
pub enum UserType {
    #[serde(rename(deserialize = "guest"))]
    Guest,
    #[serde(rename(deserialize = "registered"))]
    Registered,
    Admin,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PseudoUser {
    pub id: Uuid,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BaseUser {
    pub id: Uuid,
    pub username: String,
    pub auth0_id: Option<String>,
    pub gender: Gender,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub updated_at: DateTime<Utc>,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub birth_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "role", content = "user")]
pub enum UserRole {
    Admin(BaseUser),
    BaseUser(BaseUser),
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct PatchUserRequest {
    pub username: Option<String>,
    pub gender: Option<Gender>,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub birth_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityStats {
    pub total_game_count: i64,
    pub total_user_count: i64,
    pub recent: RecentUserStats,
    pub average: AverageUserStats,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecentUserStats {
    pub this_month_users: i64,
    pub this_week_users: i64,
    pub todays_users: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AverageUserStats {
    pub avg_month_users: f64,
    pub avg_week_users: f64,
    pub avg_daily_users: f64,
}
