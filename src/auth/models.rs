use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{game::models::Gender, integration::models::IntegrationName};

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
    gty: String,
    aud: String,
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
            gty: String::new(),
            aud: String::new(),
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
        self.gty == "client-credentials"
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
    Guest(Uuid),
    Registered(Uuid),
    Integration(IntegrationName),
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
pub struct UserKeys {
    pub user_id: Uuid,
    pub auth0_id: Option<String>,
    pub guest_id: Option<Uuid>,
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
            id: Uuid::nil(),
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
pub struct PutUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub birth_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityStats {
    pub total_game_count: i32,
    pub total_user_count: i32,
    pub recent: RecentUserStats,
    pub average: AverageUserStats,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecentUserStats {
    pub this_month: i32,
    pub this_week: i32,
    pub today: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AverageUserStats {
    pub avg_month_users: i32,
    pub avg_week_users: i32,
    pub avg_daily_users: i32,
}
