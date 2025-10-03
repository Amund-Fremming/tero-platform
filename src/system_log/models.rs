use core::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemLog {
    pub id: i64,
    pub subject_id: String,
    pub subject_type: SubjectType,
    pub action: Action,
    pub ceverity: LogCeverity,
    pub function: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub create_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "ceverity", rename_all = "lowercase")]
pub enum LogCeverity {
    Critical,
    Warning,
    Info,
}

impl fmt::Display for LogCeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogCeverity::Critical => write!(f, "critical"),
            LogCeverity::Warning => write!(f, "warning"),
            LogCeverity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "action", rename_all = "lowercase")]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Sync,
    Other,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Create => write!(f, "write"),
            Action::Read => write!(f, "read"),
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
            Action::Sync => write!(f, "sync"),
            Action::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "subject_type", rename_all = "lowercase")]
pub enum SubjectType {
    #[sqlx(rename = "registered_user")]
    RegisteredUser,
    #[sqlx(rename = "guest_user")]
    GuestUser,
    Integration,
    System,
}

impl fmt::Display for SubjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubjectType::RegisteredUser => write!(f, "registered"),
            SubjectType::GuestUser => write!(f, "guest"),
            SubjectType::Integration => write!(f, "integration"),
            SubjectType::System => write!(f, "system"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyslogPageQuery {
    pub page_num: u16,
    pub subject_type: Option<SubjectType>,
    pub action: Option<Action>,
    pub ceverity: Option<LogCeverity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSyslogRequest {
    pub action: Option<Action>,
    pub ceverity: Option<LogCeverity>,
    pub description: Option<String>,
    pub function: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
