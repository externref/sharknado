use crate::helpers::messages::Messages;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::engine::{QueryCondition, QueryOperator};
pub struct TCPServer {
    pub listener: tokio::net::TcpListener,
    logger: crate::helpers::logging::Logger,
    engine: Arc<crate::engine::Engine>,
    user_manager: Arc<crate::user_manager::UserManager>,
}

impl TCPServer {
    pub async fn new(
        host: String,
        port: u16,
        logger: crate::helpers::logging::Logger,
        database_name: String,
        user_manager: Arc<crate::user_manager::UserManager>,
    ) -> Self {
        let listener = tokio::net::TcpListener::bind((host.as_str(), port))
            .await
            .unwrap();

        let local_data_path = Self::get_local_storage_path();

        logger
            .info(&format!("Database name: {}", database_name))
            .await;
        logger
            .info(&format!("Database storage path: {}", local_data_path))
            .await;

        let engine = Arc::new(crate::engine::Engine::new(database_name, local_data_path));

        if let Err(e) = engine.replay_log() {
            eprintln!("Failed to replay log: {}", e);
        }

        TCPServer {
            listener,
            logger,
            engine,
            user_manager,
        }
    }

    fn get_local_storage_path() -> String {
        use std::env;
        use std::path::PathBuf;

        let app_data_dir = if cfg!(target_os = "windows") {
            env::var("APPDATA")
                .map(|path| PathBuf::from(path).join("sharknado"))
                .unwrap_or_else(|_| PathBuf::from("./data"))
        } else if cfg!(target_os = "macos") {
            env::var("HOME")
                .map(|path| PathBuf::from(path).join("Library/Application Support/sharknado"))
                .unwrap_or_else(|_| PathBuf::from("./data"))
        } else {
            env::var("XDG_DATA_HOME")
                .map(|path| PathBuf::from(path).join("sharknado"))
                .or_else(|_| {
                    env::var("HOME").map(|path| PathBuf::from(path).join(".local/share/sharknado"))
                })
                .unwrap_or_else(|_| PathBuf::from("./data"))
        };

        if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
            eprintln!(
                "Warning: Could not create data directory {:?}: {}",
                app_data_dir, e
            );
            eprintln!("Falling back to ./data directory");

            let fallback_dir = PathBuf::from("./data");
            if let Err(e) = std::fs::create_dir_all(&fallback_dir) {
                eprintln!("Error: Could not create fallback data directory: {}", e);
            }
            return fallback_dir.to_string_lossy().to_string();
        }

        app_data_dir.to_string_lossy().to_string()
    }

    async fn parse_command(&self, command: &str, connection_id: &str) -> String {
        let parts: Vec<&str> = command.trim().splitn(4, ' ').collect();

        if parts.is_empty() {
            return Messages::ERROR_EMPTY_COMMAND.to_string();
        }

        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "login" => {
                if parts.len() != 3 {
                    return Messages::ERROR_LOGIN_ARGS.to_string();
                }
                
                let username = parts[1];
                let password = parts[2];
                
                match self.user_manager.authenticate_connection(connection_id, username, password) {
                    Ok(()) => {
                        self.logger.info(&format!("User {} logged in from {}", username, connection_id)).await;
                        Messages::LOGIN_SUCCESS.to_string()
                    }
                    Err(_) => Messages::ERROR_INVALID_CREDENTIALS.to_string(),
                }
            }
            "logout" => {
                self.user_manager.logout_connection(connection_id);
                self.logger.info(&format!("User logged out from {}", connection_id)).await;
                Messages::LOGOUT_SUCCESS.to_string()
            }
            "whoami" => {
                if let Some(user) = self.user_manager.get_connection_user(connection_id) {
                    Messages::user_whoami_response(&user.username, &user.role.to_string())
                } else {
                    Messages::no_user_logged_in()
                }
            }
            "set" => {
                if !self.user_manager.is_connection_authenticated(connection_id) {
                    return Messages::ERROR_NOT_AUTHENTICATED.to_string();
                }
                
                if parts.len() != 4 {
                    return Messages::ERROR_SET_ARGS.to_string();
                }
                let table = parts[1].to_string();
                let key = parts[2].to_string();
                let json_value = parts[3];

                match serde_json::from_str(json_value) {
                    Ok(value) => {
                        self.engine.add_row(table, key, value).await;
                        self.logger
                            .debug(&format!(
                                "SET operation: {} {} {}",
                                parts[1], parts[2], json_value
                            ))
                            .await;
                        Messages::SUCCESS_OK.to_string()
                    }
                    Err(_) => Messages::ERROR_INVALID_JSON.to_string(),
                }
            }
            "get" => {
                if !self.user_manager.is_connection_authenticated(connection_id) {
                    return Messages::ERROR_NOT_AUTHENTICATED.to_string();
                }
                
                if parts.len() != 3 {
                    return Messages::ERROR_GET_ARGS.to_string();
                }
                let table = parts[1].to_string();
                let key = parts[2].to_string();

                match self.engine.get_row(table.clone(), key.clone()) {
                    Some(value) => {
                        self.logger
                            .debug(&format!("GET operation: {} {} -> found", table, key))
                            .await;
                        format!("{}\n", value.to_string())
                    }
                    None => {
                        self.logger
                            .debug(&format!("GET operation: {} {} -> not found", table, key))
                            .await;
                        Messages::SUCCESS_NULL.to_string()
                    }
                }
            }
            "update" => {
                if !self.user_manager.is_connection_authenticated(connection_id) {
                    return Messages::ERROR_NOT_AUTHENTICATED.to_string();
                }
                
                if parts.len() != 4 {
                    return Messages::ERROR_UPDATE_ARGS.to_string();
                }
                let table = parts[1].to_string();
                let key = parts[2].to_string();
                let json_value = parts[3];

                match serde_json::from_str(json_value) {
                    Ok(value) => {
                        self.engine.update_row(table, key, value).await;
                        self.logger
                            .debug(&format!(
                                "UPDATE operation: {} {} {}",
                                parts[1], parts[2], json_value
                            ))
                            .await;
                        Messages::SUCCESS_OK.to_string()
                    }
                    Err(_) => Messages::ERROR_INVALID_JSON.to_string(),
                }
            }
            "delete" => {
                if !self.user_manager.is_connection_authenticated(connection_id) {
                    return Messages::ERROR_NOT_AUTHENTICATED.to_string();
                }
                
                if parts.len() != 3 {
                    return Messages::ERROR_DELETE_ARGS.to_string();
                }
                let table = parts[1].to_string();
                let key = parts[2].to_string();

                self.engine.remove_row(table.clone(), key.clone()).await;
                self.logger
                    .debug(&format!("DELETE operation: {} {}", table, key))
                    .await;
                Messages::SUCCESS_OK.to_string()
            }
            "query" => {
                if !self.user_manager.is_connection_authenticated(connection_id) {
                    return Messages::ERROR_NOT_AUTHENTICATED.to_string();
                }
                
                if parts.len() < 3 {
                    return Messages::ERROR_QUERY_ARGS.to_string();
                }
                let table = parts[1].to_string();
                let conditions_str = parts[2..].join(" ");

                // Simple condition parsing for single conditions
                let conditions = match self.parse_single_condition(&conditions_str) {
                    Ok(cond) => vec![cond],
                    Err(err) => return Messages::query_error(&err),
                };

                let results = self.engine.query_rows(table.clone(), conditions);

                if results.is_empty() {
                    self.logger
                        .debug(&format!("QUERY operation: {} -> 0 results", table))
                        .await;
                    Messages::QUERY_NO_RESULTS.to_string()
                } else {
                    self.logger
                        .debug(&format!(
                            "QUERY operation: {} -> {} results",
                            table,
                            results.len()
                        ))
                        .await;
                    let mut response = Messages::query_results_header(results.len());
                    for (key, value) in results {
                        response.push_str(&Messages::query_result_item(&key, &value.to_string()));
                    }
                    response
                }
            }
            "help" => Messages::TCP_HELP_TEXT.to_string(),
            _ => Messages::unknown_command(&cmd),
        }
    }

    fn parse_query_conditions_from_string(
        &self,
        conditions_str: &str,
    ) -> Result<Vec<crate::engine::QueryCondition>, String> {
        let mut conditions = Vec::new();
        let mut current_condition = String::new();
        let mut in_quotes = false;
        let mut chars = conditions_str.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                    current_condition.push(ch);
                }
                ' ' if !in_quotes => {
                    if !current_condition.trim().is_empty() {
                        let condition = self.parse_single_condition(&current_condition.trim())?;
                        conditions.push(condition);
                        current_condition.clear();
                    }
                }
                _ => {
                    current_condition.push(ch);
                }
            }
        }

        if !current_condition.trim().is_empty() {
            let condition = self.parse_single_condition(&current_condition.trim())?;
            conditions.push(condition);
        }

        Ok(conditions)
    }

    fn parse_query_conditions(
        &self,
        condition_parts: &[&str],
    ) -> Result<Vec<crate::engine::QueryCondition>, String> {
        let mut conditions = Vec::new();

        for part in condition_parts {
            let condition = self.parse_single_condition(part)?;
            conditions.push(condition);
        }

        Ok(conditions)
    }

    fn parse_single_condition(
        &self,
        condition_str: &str,
    ) -> Result<crate::engine::QueryCondition, String> {
        if condition_str.contains("contains") {
            let parts: Vec<&str> = condition_str.split(" contains ").collect();
            if parts.len() != 2 {
                return Err(Messages::ERROR_INVALID_CONTAINS.to_string());
            }
            let field = parts[0].trim().to_string();
            let value_str = parts[1].trim().trim_matches('"');
            return Ok(QueryCondition {
                field_path: field,
                operator: QueryOperator::Contains,
                value: serde_json::Value::String(value_str.to_string()),
            });
        }

        let operators = [">=", "<=", "!=", "=", ">", "<"];
        for op in &operators {
            if condition_str.contains(op) {
                let parts: Vec<&str> = condition_str.splitn(2, op).collect();
                if parts.len() != 2 {
                    continue;
                }

                let field = parts[0].trim().to_string();
                let value_str = parts[1].trim();

                let operator = match *op {
                    "=" => QueryOperator::Equals,
                    "!=" => QueryOperator::NotEquals,
                    ">" => QueryOperator::GreaterThan,
                    "<" => QueryOperator::LessThan,
                    ">=" => QueryOperator::GreaterThanOrEqual,
                    "<=" => QueryOperator::LessThanOrEqual,
                    _ => return Err(Messages::unsupported_operator(op)),
                };

                let value = if value_str.starts_with('"') && value_str.ends_with('"') {
                    serde_json::Value::String(value_str.trim_matches('"').to_string())
                } else if let Ok(num) = value_str.parse::<f64>() {
                    serde_json::json!(num)
                } else if value_str == "true" || value_str == "false" {
                    serde_json::Value::Bool(value_str == "true")
                } else if value_str == "null" {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(value_str.to_string())
                };

                return Ok(QueryCondition {
                    field_path: field,
                    operator,
                    value,
                });
            }
        }

        Err(Messages::invalid_condition(condition_str))
    }

    pub async fn handle_connection(&self, mut stream: tokio::net::TcpStream) {
        let peer_addr = stream
            .peer_addr()
            .unwrap_or_else(|_| "unknown".parse().unwrap());
        let connection_id = format!("{}", peer_addr); // Use peer address as connection ID
        
        self.logger
            .info(&format!("New connection from: {}", peer_addr))
            .await;

        // Send welcome message requiring authentication
        let welcome_msg = Messages::AUTH_REQUIRED;
        if let Err(e) = stream.write_all(welcome_msg.as_bytes()).await {
            self.logger
                .error(&format!("Failed to send welcome message: {}", e))
                .await;
            return;
        }

        let mut buffer = [0; 1024];

        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    self.user_manager.cleanup_connection(&connection_id);
                    self.logger
                        .info(&format!("Connection closed: {}", peer_addr))
                        .await;
                    break;
                }
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buffer[..n]);
                    let command = request.trim();

                    if command.is_empty() {
                        continue;
                    }

                    self.logger
                        .debug(&format!("[{}] Received: {}", peer_addr, command))
                        .await;

                    if command.to_lowercase() == "exit" {
                        self.user_manager.cleanup_connection(&connection_id);
                        let response = Messages::SUCCESS_GOODBYE;
                        if let Err(e) = stream.write_all(response.as_bytes()).await {
                            self.logger
                                .error(&format!("Failed to send response: {}", e))
                                .await;
                        }
                        self.logger
                            .info(&format!("Client {} disconnected", peer_addr))
                            .await;
                        break;
                    }

                    let response = self.parse_command(command, &connection_id).await;

                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        self.logger
                            .error(&format!("Failed to send response: {}", e))
                            .await;
                        break;
                    }
                }
                Err(e) => {
                    self.user_manager.cleanup_connection(&connection_id);
                    self.logger
                        .error(&format!("Failed to read from socket: {}", e))
                        .await;
                    break;
                }
            }
        }
    }
}
