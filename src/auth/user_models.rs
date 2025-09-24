use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub birth_date: Option<NaiveDate>,
}

impl User {
    pub fn strip(&self) -> Self {
        Self {
            id: self.id,
            auth0_id: None,
            user_type: UserType::Guest,
            last_active: Utc::now(),
            last_updated: Utc::now(),
            created_at: Utc::now(),
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
