pub struct LogEntry {
    operation: String,
    table: String,
    key: String,
    value: Option<String>,
    offset: u64,
}

impl LogEntry {
    pub fn new(
        operation: String,
        table: String,
        key: String,
        value: Option<String>,
        offset: u64,
    ) -> Self {
        LogEntry {
            operation,
            table,
            key,
            value,
            offset,
        }
    }
}

pub struct LogStorageSetup {
    pub database_name: String,
    pub log_file_path: std::path::PathBuf,
}

impl LogStorageSetup {
    pub fn new(database_name: String, log_file_path: std::path::PathBuf) -> Self {
        LogStorageSetup {
            database_name,
            log_file_path,
        }
    }

    pub async fn log_entry(&self, entry: LogEntry) {
        use tokio::fs::OpenOptions;
        use tokio::io::AsyncWriteExt;

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.log_file_path)
            .await
            .unwrap();

        let entry_str = format!(
            "{}|{}|{}|{}\n",
            entry.operation,
            entry.table,
            entry.key,
            entry.value.unwrap_or_default()
        );

        file.write_all(entry_str.as_bytes()).await.unwrap();
        file.flush().await.unwrap();
    }
}
