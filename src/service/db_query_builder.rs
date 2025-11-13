use core::fmt;
use sqlx::Postgres;
use tracing::debug;

pub struct DBQueryBuilder<'a> {
    builder: sqlx::QueryBuilder<'a, Postgres>,
    where_used: bool,
}

#[allow(dead_code, unused_variables)]
impl<'a> DBQueryBuilder<'a> {
    pub fn select(base: &str) -> Self {
        Self {
            builder: sqlx::QueryBuilder::new(base),
            where_used: false,
        }
    }

    pub fn from(mut self, table: &'a str) -> Self {
        self.builder.push(" FROM ");
        self.builder.push(table);
        self
    }

    pub fn r#where<T>(mut self, field: &str, value: &T) -> Self
    where
        T: fmt::Display,
    {
        match self.where_used {
            true => {
                self.builder.push(format!(" AND {field} = "));
                self.builder.push_bind(value.to_string());
            }
            false => {
                self.builder.push(format!(" WHERE {field} = "));
                self.builder.push_bind(value.to_string());
                self.where_used = true;
            }
        }

        self
    }

    pub fn where_opt<T>(mut self, field: &str, value: &Option<T>) -> Self
    where
        T: fmt::Display,
    {
        if let Some(value) = value {
            match self.where_used {
                true => {
                    self.builder.push(format!(" AND {field} = "));
                    self.builder.push_bind(value.to_string());
                }
                false => {
                    self.builder.push(format!(" WHERE {field} = "));
                    self.builder.push_bind(value.to_string());
                    self.where_used = true;
                }
            }
        }

        self
    }

    pub fn order_asc(mut self, field: &'a str) -> Self {
        self.builder.push(" ORDER BY ");
        self.builder.push_bind(field);
        self.builder.push(" ASC ");
        self
    }

    pub fn order_desc(mut self, field: &'a str) -> Self {
        self.builder.push(" ORDER BY ");
        self.builder.push_bind(field);
        self.builder.push(" DESC ");
        self
    }

    pub fn limit(mut self, limit: impl Into<usize>) -> Self {
        let limit = limit.into();
        self.builder.push(" LIMIT ");
        self.builder.push_bind(limit.to_string());
        self
    }

    pub fn offset(mut self, offset: impl Into<usize>) -> Self {
        let offset = offset.into();
        self.builder.push(" OFFSET ");
        self.builder.push_bind(offset.to_string());
        self
    }

    pub fn build(self) -> sqlx::QueryBuilder<'a, Postgres> {
        let query = self.builder.sql();
        debug!("Built query: {}", query);

        self.builder
    }
}
