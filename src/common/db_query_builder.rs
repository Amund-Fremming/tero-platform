use core::fmt;
use tracing::debug;

#[allow(dead_code)]
pub struct DBQueryBuilder {
    pub base: String,
    pub filters: Vec<String>,
    pub updates: Vec<String>,
    pub limit: Option<String>,
    pub offset: Option<String>,
    pub order_by: Option<String>,
}

#[allow(dead_code, unused_variables)]
impl DBQueryBuilder {
    pub fn new() -> Self {
        Self {
            base: String::new(),
            filters: Vec::new(),
            updates: Vec::new(),
            limit: None,
            offset: None,
            order_by: None,
        }
    }

    pub fn select(mut self, base: &str) -> Self {
        self.base.push_str(&base);
        self
    }

    pub fn update<T>(mut self, table: T) -> Self
    where
        T: fmt::Display,
    {
        self.base.push_str(&format!("UPDATE {}", table.to_string()));
        self
    }

    pub fn from(mut self, table: &str) -> Self {
        self.base.push_str(&format!("\nFROM \"{}\"", table));
        self
    }

    pub fn where_some(mut self, condition: &str) -> Self {
        self.filters.push(condition.to_string());
        self
    }

    pub fn where_opt<T>(mut self, condition: Option<T>) -> Self
    where
        T: fmt::Display,
    {
        if let Some(value) = condition {
            self.filters.push(value.to_string());
        }

        self
    }

    pub fn order_asc(self, by: &str) -> Self {
        self
    }

    pub fn order_desc(self, by: &str) -> Self {
        self
    }

    pub fn limit(mut self, limit: u16) -> Self {
        self.limit = Some(format!("LIMIT {} ", limit));
        self
    }

    pub fn offset(mut self, offset: u16) -> Self {
        self.offset = Some(format!("OFFSET {} ", offset));
        self
    }

    pub fn build(self) -> String {
        let select = self.base;
        let filters = self.filters.join(" AND ");
        let limit = self.limit.unwrap_or("".into());
        let offset = self.offset.unwrap_or("".into());
        let order_by = self.order_by.unwrap_or("".into());

        let query = format!(
            r#"
            {select}
            WHERE {filters}
            {limit}
            {offset}
            "#,
        );

        debug!("Executing query: \n {}", query);
        query
    }
}
