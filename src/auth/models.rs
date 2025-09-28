use std::{clone, collections::HashSet};

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

impl Claims {
    pub fn missing_permission<I>(&self, required: I) -> Option<HashSet<Permission>>
    where
        I: IntoIterator<Item = Permission>,
    {
        let required_iter = required.into_iter();
        let permissions = match self.permissions.is_empty() {
            true => return Some(required_iter.collect()),
            false => self.permissions.clone(),
        };

        let missing: HashSet<Permission> =
            required_iter.filter(|p| !permissions.contains(p)).collect();

        (!missing.is_empty()).then_some(missing)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubjectId {
    Guest(Uuid),
    Registered(String),
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
pub struct PutUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub birth_date: Option<NaiveDate>,
}
