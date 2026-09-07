use crate::storage::db::Database;
use rhai::{Array, Dynamic, Map};

#[derive(Clone)]
pub struct ModelDef {
    pub table: String,
    pub primary_key: String,
    pub db: Database,
}

impl ModelDef {
    pub fn new(table: &str, db: Database) -> Self {
        Self {
            table: table.to_string(),
            primary_key: "id".to_string(),
            db,
        }
    }

    pub fn with_pk(table: &str, primary_key: &str, db: Database) -> Self {
        Self {
            table: table.to_string(),
            primary_key: primary_key.to_string(),
            db,
        }
    }

    pub fn all(&self) -> Array {
        let sql = format!("SELECT * FROM {} ORDER BY {} DESC", self.table, self.primary_key);
        self.db.query(&sql, Array::new()).unwrap_or_default()
    }

    pub fn find(&self, id: Dynamic) -> Dynamic {
        let sql = format!("SELECT * FROM {} WHERE {} = ? LIMIT 1", self.table, self.primary_key);
        let mut params = Array::new();
        params.push(id);
        self.db.first(&sql, params).unwrap_or(Dynamic::UNIT)
    }

    pub fn create(&self, data: Map) -> Dynamic {
        if let Ok(res) = self.db.insert(&self.table, data) {
            if let Some(id) = res.get("last_insert_id") {
                return self.find(id.clone());
            }
        }
        Dynamic::UNIT
    }

    pub fn count(&self) -> i64 {
        let sql = format!("SELECT COUNT(*) as count FROM {}", self.table);
        if let Ok(res) = self.db.first(&sql, Array::new()) {
            if let Some(map) = res.try_cast::<Map>() {
                if let Some(c) = map.get("count") {
                    return c.as_int().unwrap_or(0);
                }
            }
        }
        0
    }

    pub fn query_builder(&self) -> QueryBuilder {
        QueryBuilder::new(&self.table, self.db.clone())
    }
}

#[derive(Clone)]
pub struct QueryBuilder {
    pub table: String,
    pub db: Database,
    pub wheres: Vec<(String, String, Dynamic)>,
    pub order_by: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl QueryBuilder {
    pub fn new(table: &str, db: Database) -> Self {
        Self {
            table: table.to_string(),
            db,
            wheres: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    pub fn where_eq(&mut self, col: &str, val: Dynamic) -> Self {
        self.wheres.push((col.to_string(), "=".to_string(), val));
        self.clone()
    }

    pub fn where_op(&mut self, col: &str, op: &str, val: Dynamic) -> Self {
        self.wheres.push((col.to_string(), op.to_string(), val));
        self.clone()
    }

    pub fn order_by_clause(&mut self, col: &str, dir: &str) -> Self {
        self.order_by = Some(format!("{} {}", col, dir));
        self.clone()
    }

    pub fn set_limit(&mut self, n: i64) -> Self {
        self.limit = Some(n);
        self.clone()
    }

    pub fn set_offset(&mut self, n: i64) -> Self {
        self.offset = Some(n);
        self.clone()
    }

    pub fn get(&self) -> Array {
        let (sql, params) = self.build_select_sql(false);
        self.db.query(&sql, params).unwrap_or_default()
    }

    pub fn first(&self) -> Dynamic {
        let (sql, params) = self.build_select_sql(true);
        self.db.first(&sql, params).unwrap_or(Dynamic::UNIT)
    }

    pub fn count(&self) -> i64 {
        let mut sql = format!("SELECT COUNT(*) as count FROM {}", self.table);
        let mut params = Array::new();

        if !self.wheres.is_empty() {
            let mut parts = Vec::new();
            for (col, op, val) in &self.wheres {
                parts.push(format!("{} {} ?", col, op));
                params.push(val.clone());
            }
            sql.push_str(" WHERE ");
            sql.push_str(&parts.join(" AND "));
        }

        if let Ok(res) = self.db.first(&sql, params) {
            if let Some(map) = res.try_cast::<Map>() {
                if let Some(c) = map.get("count") {
                    return c.as_int().unwrap_or(0);
                }
            }
        }
        0
    }

    pub fn delete(&self) -> i64 {
        let mut sql = format!("DELETE FROM {}", self.table);
        let mut params = Array::new();

        if !self.wheres.is_empty() {
            let mut parts = Vec::new();
            for (col, op, val) in &self.wheres {
                parts.push(format!("{} {} ?", col, op));
                params.push(val.clone());
            }
            sql.push_str(" WHERE ");
            sql.push_str(&parts.join(" AND "));
        }

        if let Ok(res) = self.db.run(&sql, params) {
            if let Some(ch) = res.get("changes") {
                return ch.as_int().unwrap_or(0);
            }
        }
        0
    }

    pub fn update(&self, data: Map) -> i64 {
        if data.is_empty() {
            return 0;
        }

        let mut sets = Vec::new();
        let mut params = Array::new();

        for (k, v) in data {
            sets.push(format!("{} = ?", k));
            params.push(v);
        }

        let mut sql = format!("UPDATE {} SET {}", self.table, sets.join(", "));

        if !self.wheres.is_empty() {
            let mut parts = Vec::new();
            for (col, op, val) in &self.wheres {
                parts.push(format!("{} {} ?", col, op));
                params.push(val.clone());
            }
            sql.push_str(" WHERE ");
            sql.push_str(&parts.join(" AND "));
        }

        if let Ok(res) = self.db.run(&sql, params) {
            if let Some(ch) = res.get("changes") {
                return ch.as_int().unwrap_or(0);
            }
        }
        0
    }

    fn build_select_sql(&self, is_first: bool) -> (String, Array) {
        let mut sql = format!("SELECT * FROM {}", self.table);
        let mut params = Array::new();

        if !self.wheres.is_empty() {
            let mut parts = Vec::new();
            for (col, op, val) in &self.wheres {
                parts.push(format!("{} {} ?", col, op));
                params.push(val.clone());
            }
            sql.push_str(" WHERE ");
            sql.push_str(&parts.join(" AND "));
        }

        if let Some(ref ord) = self.order_by {
            sql.push_str(&format!(" ORDER BY {}", ord));
        }

        if is_first {
            sql.push_str(" LIMIT 1");
        } else {
            if let Some(lim) = self.limit {
                sql.push_str(&format!(" LIMIT {}", lim));
            }
            if let Some(off) = self.offset {
                sql.push_str(&format!(" OFFSET {}", off));
            }
        }

        (sql, params)
    }
}
