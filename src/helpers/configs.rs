#[derive(serde::Deserialize, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}
#[derive(serde::Deserialize, Debug)]
pub struct LoggingSetup {
    #[serde(default = "default_log_level")]
    pub levels: Vec<String>,
    #[serde(default = "default_log_path")]
    pub path: String,
    #[serde(default = "default_color")]
    pub color: bool,
}
#[derive(serde::Deserialize, Debug)]
pub struct LoggingConfig {
    #[serde(default = "default_main_logging")]
    pub main: LoggingSetup,
    #[serde(default = "default_tcp_logging")]
    pub tcp: LoggingSetup,
}

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default = "default_logging")]
    pub logging: LoggingConfig,
}

pub fn log_level_from_strings(levels: &Vec<String>) -> crate::helpers::logging::LogLevel {
    let mut log_level = crate::helpers::logging::LogLevel::empty();
    for level in levels {
        match level.as_str() {
            "INFO" => log_level |= crate::helpers::logging::LogLevel::INFO,
            "DEBUG" => log_level |= crate::helpers::logging::LogLevel::DEBUG,
            "WARNING" => log_level |= crate::helpers::logging::LogLevel::WARNING,
            "ERROR" => log_level |= crate::helpers::logging::LogLevel::ERROR,
            "ALL" => log_level = crate::helpers::logging::LogLevel::ALL,
            "NONE" => log_level = crate::helpers::logging::LogLevel::NONE,
            _ => continue,
        }
    }
    log_level
}

pub fn log_path_from_string(path: &String) -> crate::helpers::logging::LogPath {
    if path == "console" {
        crate::helpers::logging::LogPath::Console
    } else {
        crate::helpers::logging::LogPath::File(path.clone())
    }
}

pub fn load_config() -> Config {
    if !std::path::Path::new("sharknado.json").exists() {
        return Config {
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
            },
            logging: LoggingConfig {
                main: LoggingSetup {
                    levels: default_log_level(),
                    path: default_log_path(),
                    color: default_color(),
                },
                tcp: LoggingSetup {
                    levels: default_log_level(),
                    path: default_log_path(),
                    color: default_color(),
                },
            },
        };
    }
    let config = serde_json::from_str(std::fs::read_to_string("sharknado.json").unwrap().as_str());
    return config.unwrap();
}

fn default_main_logging() -> LoggingSetup {
    LoggingSetup {
        levels: default_log_level(),
        path: default_log_path(),
        color: default_color(),
    }
}

fn default_tcp_logging() -> LoggingSetup {
    LoggingSetup {
        levels: default_log_level(),
        path: default_log_path(),
        color: default_color(),
    }
}

fn default_logging() -> LoggingConfig {
    LoggingConfig {
        main: default_main_logging(),
        tcp: default_tcp_logging(),
    }
}

fn default_server() -> ServerConfig {
    ServerConfig {
        host: default_host(),
        port: default_port(),
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_log_level() -> Vec<String> {
    vec!["INFO".to_string(), "DEBUG".to_string()]
}
fn default_log_path() -> String {
    "console".to_string()
}
fn default_color() -> bool {
    true
}

pub fn create_protocol_registery() {
    register_sharknado_protocol();
}

pub fn register_sharknado_protocol() {
    #[cfg(target_os = "windows")]
    {
        register_windows_protocol();
    }

    #[cfg(not(target_os = "windows"))]
    {
        register_unix_protocol();
    }
}

#[cfg(target_os = "windows")]
fn register_windows_protocol() {
    use std::env;
    use std::process::Command;

    let exe_path = env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("sharknado.exe"));
    let exe_path_str = exe_path.to_string_lossy();

    println!("Registering sharknado:// protocol handler...");

    let commands = vec![
        format!(
            r#"reg add "HKEY_CURRENT_USER\Software\Classes\sharknado" /ve /d "Sharknado Database Protocol" /f"#
        ),
        format!(
            r#"reg add "HKEY_CURRENT_USER\Software\Classes\sharknado" /v "URL Protocol" /d "" /f"#
        ),
        format!(
            r#"reg add "HKEY_CURRENT_USER\Software\Classes\sharknado\DefaultIcon" /ve /d "{},1" /f"#,
            exe_path_str
        ),
        format!(r#"reg add "HKEY_CURRENT_USER\Software\Classes\sharknado\shell" /f"#),
        format!(r#"reg add "HKEY_CURRENT_USER\Software\Classes\sharknado\shell\open" /f"#),
        format!(
            r#"reg add "HKEY_CURRENT_USER\Software\Classes\sharknado\shell\open\command" /ve /d "\"{} --connect \"%1\"\" /f"#,
            exe_path_str
        ),
    ];

    for cmd in commands {
        match Command::new("cmd").args(&["/C", &cmd]).output() {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("Failed to execute registry command: {}", cmd);
                    eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                eprintln!("Failed to execute registry command: {}", e);
            }
        }
    }

    println!("Protocol registration complete. You can now use sharknado:// URLs!");
    println!("Example: sharknado://admin:admin123@127.0.0.1:8080");
}

#[cfg(not(target_os = "windows"))]
fn register_unix_protocol() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("sharknado"));
    let exe_path_str = exe_path.to_string_lossy();

    println!("Registering sharknado:// protocol handler...");

    let home_dir = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let desktop_dir = format!("{}/.local/share/applications", home_dir);

    if let Err(e) = fs::create_dir_all(&desktop_dir) {
        eprintln!("Warning: Could not create applications directory: {}", e);
        return;
    }

    let desktop_content = format!(
        r#"[Desktop Entry]
Name=Sharknado Database
Comment=Sharknado Database Protocol Handler
Exec={} --connect %u
Icon=application-x-executable
Terminal=false
NoDisplay=true
MimeType=x-scheme-handler/sharknado;
"#,
        exe_path_str
    );

    let desktop_file_path = format!("{}/sharknado-protocol.desktop", desktop_dir);

    match fs::write(&desktop_file_path, desktop_content) {
        Ok(()) => {
            println!("Created desktop file: {}", desktop_file_path);

            if let Err(e) = std::process::Command::new("update-desktop-database")
                .arg(&desktop_dir)
                .output()
            {
                eprintln!("Warning: Could not update desktop database: {}", e);
            }

            println!("Protocol registration complete. You can now use sharknado:// URLs!");
            println!("Example: sharknado://admin:admin123@127.0.0.1:8080");
        }
        Err(e) => {
            eprintln!("Failed to create desktop file: {}", e);
        }
    }
}
