use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemLog {
    pub id: i64,
    pub subject_id: String,
    pub subject_type: SubjectType,
    pub action: LogAction,
    pub ceverity: LogCeverity,
    pub file_name: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "ceverity", rename_all = "lowercase")]
pub enum LogCeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "action", rename_all = "lowercase")]
pub enum LogAction {
    Create,
    Read,
    Update,
    Delete,
    Other,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemLogRequest {
    pub action: Option<LogAction>,
    pub ceverity: Option<LogCeverity>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
