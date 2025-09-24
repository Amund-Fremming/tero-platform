use std::collections::HashSet;

use serde::{Deserialize, Serialize};

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
