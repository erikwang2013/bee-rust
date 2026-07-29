use std::marker::PhantomData;

pub use bee_orm_macro::Model;

#[derive(Debug, thiserror::Error)]
pub enum OrmError {
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("query error: {0}")]
    QueryError(String),
    #[error("not found")]
    NotFound,
}

/// A fluent SQL query builder for a model type `T`.
pub struct QuerySet<T> {
    table: String,
    filters: Vec<String>,
    order_clauses: Vec<String>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
    _marker: PhantomData<T>,
}

impl<T> QuerySet<T> {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            filters: Vec::new(),
            order_clauses: Vec::new(),
            limit_val: None,
            offset_val: None,
            _marker: PhantomData,
        }
    }

    /// Add a WHERE condition (e.g. `"age > 18"`).
    pub fn filter(mut self, condition: impl Into<String>) -> Self {
        self.filters.push(condition.into());
        self
    }

    /// Add an ORDER BY clause (e.g. `"id DESC"`).
    pub fn order_by(mut self, clause: impl Into<String>) -> Self {
        self.order_clauses.push(clause.into());
        self
    }

    /// Set the LIMIT value.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit_val = Some(n);
        self
    }

    /// Set the OFFSET value.
    pub fn offset(mut self, n: usize) -> Self {
        self.offset_val = Some(n);
        self
    }

    /// Build the SQL string for this query.
    pub fn to_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);

        if !self.filters.is_empty() {
            let conditions = self.filters.join(" AND ");
            sql.push_str(&format!(" WHERE {}", conditions));
        }

        if !self.order_clauses.is_empty() {
            let order = self.order_clauses.join(", ");
            sql.push_str(&format!(" ORDER BY {}", order));
        }

        if let Some(limit) = self.limit_val {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset_val {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }
}
