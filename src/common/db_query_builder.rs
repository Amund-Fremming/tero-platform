use crate::config::config::CONFIG;

#[allow(dead_code)]
pub struct DBQueryBuilder {
    pub query: String,
    pub page_size: u8,
}

/*
    ISSUES

    Can become quite complex with joins and so on
    abort for later
*/

#[allow(dead_code, unused_variables)]
impl DBQueryBuilder {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            page_size: CONFIG.server.page_size,
        }
    }

    pub fn select_many(self, fields: &str) -> Self {
        self
    }

    pub fn select(self, field: &str) -> Self {
        self
    }

    pub fn from(self, table: &str) -> Self {
        self
    }

    pub fn whhere(self, table: &str) -> Self {
        self
    }

    pub fn order_asc(self, by: &str) -> Self {
        self
    }

    pub fn order_desc(self, by: &str) -> Self {
        self
    }

    pub fn limit(self, limit: u32) -> Self {
        self
    }

    pub fn offset(self, offset: u32) -> Self {
        self
    }

    pub fn build(self) -> String {
        "".to_string()
    }
}
