bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq)]
    pub struct LogLevel: u8 {
        const INFO    = 0b0001;
        const DEBUG   = 0b0010;
        const WARNING = 0b0100;
        const ERROR   = 0b1000;
        const ALL     = Self::INFO.bits() | Self::DEBUG.bits() | Self::WARNING.bits() | Self::ERROR.bits();
        const NONE    = 0b0000;
   }
}

#[derive(Clone)]
pub enum LogPath {
    Console,
    File(String),
}

#[derive(Clone)]
pub struct Logger {
    pub name: String,
    pub level: LogLevel,
    pub path: LogPath,
    pub color: bool,
}

impl Logger {
    pub fn new(name: String, level: LogLevel, path: LogPath, color: bool) -> Self {
        Logger {
            name,
            level,
            path,
            color,
        }
    }
    pub async fn log(&self, level: LogLevel, message: &str) {
        if self.level.contains(level) {
            if level == LogLevel::INFO {
                self.info(message).await;
            } else if level == LogLevel::DEBUG {
                self.debug(message).await;
            } else if level == LogLevel::WARNING {
                self.warning(message).await;
            } else if level == LogLevel::ERROR {
                self.error(message).await;
            }
        }
    }

    async fn log_in_file(&self, formatted_message: &str) {
        if let LogPath::File(ref path) = self.path {
            use tokio::fs::OpenOptions;
            use tokio::io::AsyncWriteExt;

            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .await
                .unwrap();
            file.write_all(formatted_message.as_bytes()).await.unwrap();
            file.write_all(b"\n").await.unwrap();
            file.flush().await.unwrap();
        }
    }

    pub async fn info(&self, message: &str) {
        if !self.level.contains(LogLevel::INFO) {
            return;
        }
        let timestamp = Self::get_timestamp();
        let formatted_message = format!("[{}] [INFO] [{}] {}", timestamp, self.name, message);
        if self.color {
            println!("\x1b[32m{}\x1b[0m", formatted_message);
        } else {
            println!("{}", formatted_message);
        }
        self.log_in_file(&formatted_message).await;
    }

    pub async fn debug(&self, message: &str) {
        if !self.level.contains(LogLevel::DEBUG) {
            return;
        }
        let timestamp = Self::get_timestamp();
        let formatted_message = format!("[{}] [DEBUG] [{}] {}", timestamp, self.name, message);
        if self.color {
            println!("\x1b[34m{}\x1b[0m", formatted_message);
        } else {
            println!("{}", formatted_message);
        }
        self.log_in_file(&formatted_message).await;
    }

    pub async fn warning(&self, message: &str) {
        if !self.level.contains(LogLevel::WARNING) {
            return;
        }
        let timestamp = Self::get_timestamp();
        let formatted_message = format!("[{}] [WARNING] [{}] {}", timestamp, self.name, message);
        if self.color {
            println!("\x1b[33m{}\x1b[0m", formatted_message);
        } else {
            println!("{}", formatted_message);
        }
        self.log_in_file(&formatted_message).await;
    }

    pub async fn error(&self, message: &str) {
        if !self.level.contains(LogLevel::ERROR) {
            return;
        }
        let timestamp = Self::get_timestamp();
        let formatted_message = format!("[{}] [ERROR] [{}] {}", timestamp, self.name, message);
        if self.color {
            println!("\x1b[31m{}\x1b[0m", formatted_message);
        } else {
            println!("{}", formatted_message);
        }
        self.log_in_file(&formatted_message).await;
    }

    fn get_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        const SECONDS_PER_DAY: u64 = 86400;
        const SECONDS_PER_HOUR: u64 = 3600;
        const SECONDS_PER_MINUTE: u64 = 60;

        let days_since_epoch = now / SECONDS_PER_DAY;

        let mut year = 1970;
        let mut remaining_days = days_since_epoch;
        while remaining_days >= 365 {
            let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                366
            } else {
                365
            };
            if remaining_days >= days_in_year {
                remaining_days -= days_in_year;
                year += 1;
            } else {
                break;
            }
        }
        let month = (remaining_days / 30) + 1;
        let day = (remaining_days % 30) + 1;
        let seconds_today = now % SECONDS_PER_DAY;
        let hours = seconds_today / SECONDS_PER_HOUR;
        let minutes = (seconds_today % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
        let seconds = seconds_today % SECONDS_PER_MINUTE;

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hours, minutes, seconds
        )
    }
}
