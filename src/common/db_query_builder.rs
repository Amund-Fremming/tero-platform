use core::fmt;
use tracing::debug;

pub struct DBQueryBuilder {
    pub query: String,
    pub filters: Vec<String>,
    pub filters_added: bool,
}

#[allow(dead_code, unused_variables)]
impl DBQueryBuilder {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            filters: Vec::new(),
            filters_added: false,
        }
    }

    pub fn select(mut self, base: &str) -> Self {
        self.query.push_str(&base);
        self
    }

    pub fn from(mut self, table: &str) -> Self {
        self.query.push_str(&format!("\nFROM \"{}\"", table));
        self
    }

    pub fn r#where(mut self, field: &str, value: &str) -> Self {
        self.filters.push(format!("{} = '{}'", field, value));
        self
    }

    pub fn where_opt<T>(mut self, field: &str, value: Option<T>) -> Self
    where
        T: fmt::Display,
    {
        if let Some(value) = value {
            self.filters.push(format!("{} = {}", field, value));
        }

        self
    }

    pub fn order_asc(mut self, field: &str) -> Self {
        self.ensure_filters();
        self.query.push_str(&format!("\nORDER BY {} ASC", field));
        self
    }

    pub fn order_desc(mut self, field: &str) -> Self {
        self.ensure_filters();
        self.query.push_str(&format!("\nORDER BY {} DESC", field));
        self
    }

    pub fn limit(mut self, limit: impl Into<usize>) -> Self {
        self.ensure_filters();
        let limit = limit.into();
        self.query.push_str(&format!("\nLIMIT {}", limit));
        self
    }

    pub fn offset(mut self, offset: u16) -> Self {
        self.ensure_filters();
        self.query.push_str(&format!("\nOFFSET {} ", offset));
        self
    }

    fn ensure_filters(&mut self) {
        if self.filters.is_empty() || self.filters_added {
            return;
        };

        self.query
            .push_str(&format!("\nWHERE {}", self.filters.join(" AND ")));

        self.filters_added = true;
    }

    pub fn build(self) -> String {
        debug!("Executing query: \n {}", self.query);
        self.query
    }
}
