use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum QueryOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
}

#[derive(Debug, Clone)]
pub struct QueryCondition {
    pub field_path: String,
    pub operator: QueryOperator,
    pub value: serde_json::Value,
}

pub struct Engine {
    pub log_storage: crate::logs::LogStorageSetup,
    pub database_name: String,
    pub database_path: String,
    index: Arc<RwLock<HashMap<String, HashMap<String, serde_json::Value>>>>,
}

impl Engine {
    pub fn new(database_name: String, database_path: String) -> Self {
        let log_storage = crate::logs::LogStorageSetup::new(
            database_name.clone(),
            std::path::PathBuf::from(database_path.clone()).join(format!("{}.log", database_name)),
        );
        Engine {
            log_storage,
            database_name,
            database_path,
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_row(&self, table: String, key: String, values: serde_json::Value) {
        let entry = crate::logs::LogEntry::new(
            "add".to_string(),
            table.clone(),
            key.clone(),
            Some(values.to_string()),
            0,
        );
        self.log_storage.log_entry(entry).await;

        let mut index = self.index.write().unwrap();
        let table_map = index.entry(table).or_insert_with(HashMap::new);
        table_map.insert(key, values);
    }

    pub fn get_row(&self, table: String, key: String) -> Option<serde_json::Value> {
        let index = self.index.read().unwrap();
        index.get(&table)?.get(&key).cloned()
    }

    pub fn query_rows(
        &self,
        table: String,
        conditions: Vec<QueryCondition>,
    ) -> Vec<(String, serde_json::Value)> {
        let index = self.index.read().unwrap();
        let table_data = match index.get(&table) {
            Some(data) => data,
            None => return Vec::new(),
        };

        table_data
            .iter()
            .filter(|(_, value)| self.matches_conditions(value, &conditions))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    pub fn query_rows_with_limit(
        &self,
        table: String,
        conditions: Vec<QueryCondition>,
        limit: Option<usize>,
    ) -> Vec<(String, serde_json::Value)> {
        let index = self.index.read().unwrap();
        let table_data = match index.get(&table) {
            Some(data) => data,
            None => return Vec::new(),
        };

        let mut results: Vec<(String, serde_json::Value)> = table_data
            .iter()
            .filter(|(_, value)| self.matches_conditions(value, &conditions))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();

        if let Some(limit_count) = limit {
            results.truncate(limit_count);
        }

        results
    }

    fn matches_conditions(&self, value: &serde_json::Value, conditions: &[QueryCondition]) -> bool {
        conditions
            .iter()
            .all(|condition| self.matches_condition(value, condition))
    }

    fn matches_condition(&self, value: &serde_json::Value, condition: &QueryCondition) -> bool {
        let field_value = self.get_nested_value(value, &condition.field_path);

        match (&field_value, &condition.operator, &condition.value) {
            (Some(field_val), QueryOperator::Equals, expected) => *field_val == expected,
            (Some(field_val), QueryOperator::NotEquals, expected) => *field_val != expected,
            (
                Some(serde_json::Value::String(field_str)),
                QueryOperator::Contains,
                serde_json::Value::String(expected_str),
            ) => field_str.contains(expected_str.as_str()),
            (
                Some(serde_json::Value::Number(field_num)),
                QueryOperator::GreaterThan,
                serde_json::Value::Number(expected_num),
            ) => field_num.as_f64().unwrap_or(0.0) > expected_num.as_f64().unwrap_or(0.0),
            (
                Some(serde_json::Value::Number(field_num)),
                QueryOperator::LessThan,
                serde_json::Value::Number(expected_num),
            ) => field_num.as_f64().unwrap_or(0.0) < expected_num.as_f64().unwrap_or(0.0),
            (
                Some(serde_json::Value::Number(field_num)),
                QueryOperator::GreaterThanOrEqual,
                serde_json::Value::Number(expected_num),
            ) => field_num.as_f64().unwrap_or(0.0) >= expected_num.as_f64().unwrap_or(0.0),
            (
                Some(serde_json::Value::Number(field_num)),
                QueryOperator::LessThanOrEqual,
                serde_json::Value::Number(expected_num),
            ) => field_num.as_f64().unwrap_or(0.0) <= expected_num.as_f64().unwrap_or(0.0),
            (None, _, _) => false,
            _ => false,
        }
    }

    fn get_nested_value<'a>(
        &self,
        value: &'a serde_json::Value,
        field_path: &str,
    ) -> Option<&'a serde_json::Value> {
        let path_parts: Vec<&str> = field_path.split('.').collect();
        let mut current = value;

        for part in path_parts {
            match current {
                serde_json::Value::Object(obj) => {
                    current = obj.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    pub async fn remove_row(&self, table: String, key: String) {
        let entry =
            crate::logs::LogEntry::new("remove".to_string(), table.clone(), key.clone(), None, 0);
        self.log_storage.log_entry(entry).await;

        let mut index = self.index.write().unwrap();
        if let Some(table_map) = index.get_mut(&table) {
            table_map.remove(&key);
        }
    }

    pub async fn update_row(&self, table: String, key: String, values: serde_json::Value) {
        let entry = crate::logs::LogEntry::new(
            "update".to_string(),
            table.clone(),
            key.clone(),
            Some(values.to_string()),
            0,
        );
        self.log_storage.log_entry(entry).await;

        let mut index = self.index.write().unwrap();
        let table_map = index.entry(table).or_insert_with(HashMap::new);
        table_map.insert(key, values);
    }

    pub fn replay_log(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        if !self.log_storage.log_file_path.exists() {
            return Ok(());
        }

        let file = File::open(&self.log_storage.log_file_path)?;
        let reader = BufReader::new(file);

        let mut index = self.index.write().unwrap();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split('|').collect();

            if parts.len() >= 3 {
                let operation = parts[0];
                let table = parts[1].to_string();
                let key = parts[2].to_string();
                let value = if parts.len() > 3 && !parts[3].is_empty() {
                    serde_json::from_str(parts[3]).ok()
                } else {
                    None
                };

                let table_map = index.entry(table.clone()).or_insert_with(HashMap::new);

                match operation {
                    "add" | "update" => {
                        if let Some(val) = value {
                            table_map.insert(key, val);
                        }
                    }
                    "remove" => {
                        table_map.remove(&key);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
