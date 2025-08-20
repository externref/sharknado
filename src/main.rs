use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod connection;
mod engine;
mod helpers;
mod logs;
mod user_manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cli_mode = args.contains(&"--cli".to_string());
    let register_protocol = args.contains(&"--register-protocol".to_string());
    let connect_uri = args
        .iter()
        .position(|arg| arg == "--connect")
        .and_then(|pos| args.get(pos + 1))
        .cloned();

    if register_protocol {
        helpers::configs::create_protocol_registery();
        return Ok(());
    }

    if let Some(uri) = connect_uri {
        connect_via_protocol(&uri).await?;
        return Ok(());
    }

    let database_name = if args.len() > 1 {
        let db_name = if cli_mode {
            args.iter()
                .find(|&arg| arg != "--cli" && !arg.contains(&args[0]))
                .unwrap_or(&"sharknado_default".to_string())
                .clone()
        } else {
            args[1].clone()
        };

        if db_name == "--help" || db_name == "-h" {
            println!("Sharknado Database Engine");
            println!("Usage: {} [OPTIONS] [database-name]", args[0]);
            println!("\nOptions:");
            println!("  --cli                    User management mode (create/manage users)");
            println!("  --connect <uri>          Connect using sharknado:// protocol URI");
            println!("  --register-protocol      Register sharknado:// protocol handler");
            println!("  --help, -h               Show this help message");
            println!("\nArguments:");
            println!(
                "  database-name            Name of the database to use (default: sharknado_default)"
            );
            println!("\nModes:");
            println!(
                "  Default Mode             Start TCP server - users connect with credentials"
            );
            println!("  CLI Mode (--cli)         User management only - create/manage users");
            println!("\nExamples:");
            println!(
                "  {} my-database                           # Start TCP server with 'my-database'",
                args[0]
            );
            println!(
                "  {} --cli                                 # User management mode",
                args[0]
            );
            println!(
                "  {} --register-protocol                   # Register protocol handler",
                args[0]
            );
            println!(
                "  {} --connect sharknado://admin:admin123@127.0.0.1:8080  # Connect via protocol",
                args[0]
            );
            println!("\nWorkflow:");
            println!("  1. Use --cli to create users");
            println!("  2. Start TCP server with database name");
            println!("  3. Connect using sharknado://username:password@host:port");
            std::process::exit(0);
        }
        db_name.clone()
    } else {
        "sharknado_default".to_string()
    };

    let configs = helpers::configs::load_config();
    let core_logger = helpers::logging::Logger::new(
        "sharknado::main".to_string(),
        helpers::configs::log_level_from_strings(&configs.logging.main.levels),
        helpers::configs::log_path_from_string(&configs.logging.main.path),
        configs.logging.main.color,
    );

    core_logger
        .info(&format!(
            "Starting Sharknado database engine with database: {}",
            database_name
        ))
        .await;

    let user_manager = std::sync::Arc::new(user_manager::UserManager::new());
    user_manager.ensure_default_admin();

    if cli_mode {
        start_cli_mode(database_name, user_manager, core_logger).await?;
        return Ok(());
    }

    let tcp_logger = helpers::logging::Logger::new(
        "sharknado::tcp".to_string(),
        helpers::configs::log_level_from_strings(&configs.logging.tcp.levels),
        helpers::configs::log_path_from_string(&configs.logging.tcp.path),
        configs.logging.tcp.color,
    );

    let tcp_connection = connection::TCPServer::new(
        configs.server.host.clone(),
        configs.server.port,
        tcp_logger,
        database_name.clone(),
        user_manager.clone(),
    )
    .await;
    core_logger
        .info(&format!(
            "Sharknado server is running ...\nConnect on: http://{}:{}",
            configs.server.host, configs.server.port
        ))
        .await;

    loop {
        let (socket, _) = tcp_connection.listener.accept().await?;
        tcp_connection.handle_connection(socket).await;
    }
}

async fn start_cli_mode(
    database_name: String,
    user_manager: std::sync::Arc<user_manager::UserManager>,
    logger: helpers::logging::Logger,
) -> Result<(), Box<dyn std::error::Error>> {
    use helpers::messages::Messages;
    use std::io::{self, Write};

    println!(
        "Sharknado CLI User Management Mode - Database: {}",
        database_name
    );
    println!("This mode is only for user management. Use TCP mode for database operations.");
    println!("Type 'help' for available commands, 'exit' to quit");

    logger.info("CLI mode started").await;

    loop {
        print!("sharknado-users> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let command = input.trim();

        if command.is_empty() {
            continue;
        }

        if command.to_lowercase() == "exit" {
            println!("{}", Messages::SUCCESS_GOODBYE.trim());
            logger.info("CLI mode exited").await;
            break;
        }

        let response = parse_cli_command(command, &user_manager).await;
        print!("{}", response);
    }

    Ok(())
}

async fn parse_cli_command(
    command: &str,
    user_manager: &std::sync::Arc<user_manager::UserManager>,
) -> String {
    use helpers::messages::Messages;

    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    if parts.is_empty() {
        return Messages::ERROR_EMPTY_COMMAND.to_string();
    }

    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "user" => parse_user_command(&parts[1..], user_manager).await,
        "help" => {
            format!(
                "Sharknado CLI User Management Commands:\n\
                user create <username> <password> <role>  - Create a new user (admin/user)\n\
                user list                                  - List all users (admin only)\n\
                user delete <username>                    - Delete a user (admin only)\n\
                user update <username> <field> <value>    - Update user password or role (admin only)\n\
                help                                       - Show this help message\n\
                exit                                       - Exit CLI mode\n\n\
                Note: For database operations, start the TCP server and connect with:\n\
                sharknado://username:password@127.0.0.1:8080\n"
            )
        }
        _ => {
            format!(
                "Unknown command: '{}'\n\
                This CLI mode is only for user management.\n\
                Use 'help' to see available commands.\n\
                For database operations, connect to the TCP server using:\n\
                sharknado://username:password@127.0.0.1:8080\n",
                cmd
            )
        }
    }
}

async fn parse_user_command(
    parts: &[&str],
    user_manager: &std::sync::Arc<user_manager::UserManager>,
) -> String {
    use helpers::messages::Messages;

    if parts.is_empty() {
        return Messages::ERROR_INVALID_USER_COMMAND.to_string();
    }

    let user_cmd = parts[0].to_lowercase();

    match user_cmd.as_str() {
        "create" => {
            if parts.len() != 4 {
                return format!(
                    "ERROR: USER CREATE requires 3 arguments: USER CREATE <username> <password> <role>\nReceived {} arguments\n",
                    parts.len() - 1
                );
            }

            let username = parts[1].to_string();
            let password = parts[2].to_string();
            let role_str = parts[3];

            if let Some(role) = user_manager::UserRole::from_str(role_str) {
                match user_manager.create_user(username, password, role) {
                    Ok(()) => Messages::USER_CREATED.to_string(),
                    Err(_) => Messages::ERROR_USER_EXISTS.to_string(),
                }
            } else {
                Messages::ERROR_INVALID_ROLE.to_string()
            }
        }
        "list" => {
            if !user_manager.is_admin() {
                return Messages::ERROR_INSUFFICIENT_PERMISSIONS.to_string();
            }

            let users = user_manager.list_users();
            if users.is_empty() {
                "No users found\n".to_string()
            } else {
                let mut response = Messages::user_list_header(users.len());
                for user in users {
                    response.push_str(&Messages::user_list_item(
                        &user.username,
                        &user.role.to_string(),
                        &user.created_at,
                    ));
                }
                response
            }
        }
        "delete" => {
            if parts.len() != 2 {
                return Messages::ERROR_USER_DELETE_ARGS.to_string();
            }

            let username = parts[1];

            match user_manager.delete_user(username) {
                Ok(()) => Messages::USER_DELETED.to_string(),
                Err(err) => {
                    if err.contains("permission") {
                        Messages::ERROR_INSUFFICIENT_PERMISSIONS.to_string()
                    } else {
                        Messages::ERROR_USER_NOT_FOUND.to_string()
                    }
                }
            }
        }
        "update" => {
            if parts.len() != 4 {
                return Messages::ERROR_USER_UPDATE_ARGS.to_string();
            }

            let username = parts[1];
            let field = parts[2];
            let value = parts[3];

            if field != "password" && field != "role" {
                return Messages::ERROR_INVALID_UPDATE_FIELD.to_string();
            }

            if field == "role" {
                if user_manager::UserRole::from_str(value).is_none() {
                    return Messages::ERROR_INVALID_ROLE.to_string();
                }
            }

            match user_manager.update_user(username, field, value) {
                Ok(()) => Messages::USER_UPDATED.to_string(),
                Err(err) => {
                    if err.contains("permission") {
                        Messages::ERROR_INSUFFICIENT_PERMISSIONS.to_string()
                    } else if err.contains("not found") {
                        Messages::ERROR_USER_NOT_FOUND.to_string()
                    } else {
                        format!("ERROR: {}\n", err)
                    }
                }
            }
        }
        _ => {
            format!(
                "Invalid user command: '{}'\n\
                Available commands:\n\
                  user create <username> <password> <role>\n\
                  user list\n\
                  user delete <username>\n\
                  user update <username> <field> <value>\n",
                user_cmd
            )
        }
    }
}

#[derive(Debug)]
struct SharknadorUri {
    username: String,
    password: String,
    host: String,
    port: u16,
    database: Option<String>,
}

impl SharknadorUri {
    fn parse(uri: &str) -> Result<Self, String> {
        if !uri.starts_with("sharknado://") {
            return Err("URI must start with 'sharknado://'".to_string());
        }

        let uri_body = &uri[12..];
        let parts: Vec<&str> = uri_body.split('@').collect();
        if parts.len() != 2 {
            return Err("URI must contain username:password@host:port".to_string());
        }
        let auth_parts: Vec<&str> = parts[0].split(':').collect();
        if auth_parts.len() != 2 {
            return Err("Authentication must be in format username:password".to_string());
        }

        let username = auth_parts[0].to_string();
        let password = auth_parts[1].to_string();
        let host_port_db = parts[1];
        let (host_port, database) = if host_port_db.contains('/') {
            let split: Vec<&str> = host_port_db.splitn(2, '/').collect();
            (split[0], Some(split[1].to_string()))
        } else {
            (host_port_db, None)
        };

        let host_port_parts: Vec<&str> = host_port.split(':').collect();
        if host_port_parts.len() != 2 {
            return Err("Host must be in format host:port".to_string());
        }

        let host = host_port_parts[0].to_string();
        let port = host_port_parts[1]
            .parse::<u16>()
            .map_err(|_| "Port must be a valid number".to_string())?;

        Ok(SharknadorUri {
            username,
            password,
            host,
            port,
            database,
        })
    }
}

async fn connect_via_protocol(uri: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_uri = SharknadorUri::parse(uri)?;

    println!("Connecting to Sharknado database...");
    println!("Host: {}:{}", parsed_uri.host, parsed_uri.port);
    println!("User: {}", parsed_uri.username);
    if let Some(db) = &parsed_uri.database {
        println!("Database: {}", db);
    }

    use tokio::net::TcpStream;

    let addr = format!("{}:{}", parsed_uri.host, parsed_uri.port);
    let mut stream = TcpStream::connect(&addr).await?;

    println!("Connected! Authenticating...");

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    let welcome = String::from_utf8_lossy(&buffer[..n]);
    print!("{}", welcome);

    let login_cmd = format!("LOGIN {} {}\n", parsed_uri.username, parsed_uri.password);
    stream.write_all(login_cmd.as_bytes()).await?;

    let n = stream.read(&mut buffer).await?;
    let login_response = String::from_utf8_lossy(&buffer[..n]);
    print!("{}", login_response);

    if login_response.contains("successful") {
        println!("Authentication successful! Starting interactive session...");
        start_interactive_client_session(stream).await?;
    } else {
        println!("Authentication failed!");
        return Err("Authentication failed".into());
    }

    Ok(())
}

async fn start_interactive_client_session(
    mut stream: tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    println!("Interactive mode started. Type 'exit' to disconnect.");

    let mut buffer = [0; 1024];

    loop {
        print!("sharknado> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let command = input.trim();

        if command.is_empty() {
            continue;
        }

        if command.to_lowercase() == "exit" {
            stream.write_all(b"exit\n").await?;
            break;
        }

        stream
            .write_all(format!("{}\n", command).as_bytes())
            .await?;

        let n = stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]);
        print!("{}", response);
    }

    println!("Disconnected from Sharknado database.");
    Ok(())
}
