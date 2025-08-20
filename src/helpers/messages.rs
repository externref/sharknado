pub struct Messages;

impl Messages {
    pub const HELP_TEXT: &'static str = "Available commands:\n\
        SET <table> <key> <json_value> - Insert or update a record\n\
        GET <table> <key> - Retrieve a record\n\
        UPDATE <table> <key> <json_value> - Update a record\n\
        DELETE <table> <key> - Delete a record\n\
        QUERY <table> <field>=<value> [<field2>><value2>...] - Query records by field values\n\
        \n\
        User Management (CLI only):\n\
        USER CREATE <username> <password> <role> - Create a new user\n\
        USER LIST - List all users\n\
        USER DELETE <username> - Delete a user\n\
        USER UPDATE <username> <field> <value> - Update user field (password, role)\n\
        USER LOGIN <username> <password> - Authenticate user\n\
        USER LOGOUT - Log out current user\n\
        USER WHOAMI - Show current logged in user\n\
        \n\
        HELP - Show this help message\n\
        \n\
        Query operators: = != > < >= <= contains\n\
        Examples:\n\
          QUERY users name=\"John\"\n\
          QUERY products price>100\n\
          QUERY users age>=18 status=\"active\"\n\
          QUERY posts title contains \"database\"\n\
          USER CREATE admin admin123 admin\n\
          USER LOGIN admin admin123\n";

    pub const TCP_HELP_TEXT: &'static str = "Available commands:\n\
        LOGIN <username> <password> - Authenticate to access database\n\
        SET <table> <key> <json_value> - Insert or update a record (requires login)\n\
        GET <table> <key> - Retrieve a record (requires login)\n\
        UPDATE <table> <key> <json_value> - Update a record (requires login)\n\
        DELETE <table> <key> - Delete a record (requires login)\n\
        QUERY <table> <field>=<value> [<field2>><value2>...] - Query records (requires login)\n\
        LOGOUT - Log out from current session\n\
        WHOAMI - Show current logged in user\n\
        HELP - Show this help message\n\
        \n\
        Note: You must login before using database commands.\n\
        Default admin user: username='admin', password='admin123'\n\
        \n\
        Query operators: = != > < >= <= contains\n\
        Examples:\n\
          LOGIN admin admin123\n\
          QUERY users name=\"John\"\n\
          QUERY products price>100\n";

    // Success messages
    pub const SUCCESS_OK: &'static str = "OK\n";
    pub const SUCCESS_NULL: &'static str = "NULL\n";
    pub const SUCCESS_GOODBYE: &'static str = "Goodbye!\n";

    // Error messages - Command format errors
    pub const ERROR_EMPTY_COMMAND: &'static str = "ERROR: Empty command\n";
    pub const ERROR_SET_ARGS: &'static str =
        "ERROR: SET requires 3 arguments: SET <table> <key> <value>\n";
    pub const ERROR_GET_ARGS: &'static str = "ERROR: GET requires 2 arguments: GET <table> <key>\n";
    pub const ERROR_UPDATE_ARGS: &'static str =
        "ERROR: UPDATE requires 3 arguments: UPDATE <table> <key> <value>\n";
    pub const ERROR_DELETE_ARGS: &'static str =
        "ERROR: DELETE requires 2 arguments: DELETE <table> <key>\n";
    pub const ERROR_QUERY_ARGS: &'static str =
        "ERROR: QUERY requires at least 2 arguments: QUERY <table> <conditions...>\n";

    // Error messages - JSON errors
    pub const ERROR_INVALID_JSON: &'static str = "ERROR: Invalid JSON value\n";

    // Authentication messages
    pub const ERROR_LOGIN_ARGS: &'static str = "ERROR: LOGIN requires 2 arguments: LOGIN <username> <password>\n";
    pub const AUTH_REQUIRED: &'static str = "Authentication required. Please use: LOGIN <username> <password>\n";
    pub const LOGIN_SUCCESS: &'static str = "Login successful\n";
    pub const LOGOUT_SUCCESS: &'static str = "Logged out\n";

    // Error messages - Query errors
    pub const ERROR_INVALID_CONTAINS: &'static str =
        "Invalid contains condition format. Use: field contains \"value\"";

    // Query response messages
    pub const QUERY_NO_RESULTS: &'static str = "No results found\n";

    // User management messages
    pub const USER_CREATED: &'static str = "User created successfully\n";
    pub const USER_DELETED: &'static str = "User deleted successfully\n";
    pub const USER_UPDATED: &'static str = "User updated successfully\n";
    pub const USER_LOGIN_SUCCESS: &'static str = "Login successful\n";
    pub const USER_LOGOUT_SUCCESS: &'static str = "Logged out successfully\n";
    
    // User management errors
    pub const ERROR_USER_EXISTS: &'static str = "ERROR: User already exists\n";
    pub const ERROR_USER_NOT_FOUND: &'static str = "ERROR: User not found\n";
    pub const ERROR_INVALID_CREDENTIALS: &'static str = "ERROR: Invalid username or password\n";
    pub const ERROR_NOT_AUTHENTICATED: &'static str = "ERROR: Not authenticated. Please login first\n";
    pub const ERROR_INSUFFICIENT_PERMISSIONS: &'static str = "ERROR: Insufficient permissions\n";
    pub const ERROR_USER_CREATE_ARGS: &'static str = "ERROR: USER CREATE requires 3 arguments: USER CREATE <username> <password> <role>\n";
    pub const ERROR_USER_DELETE_ARGS: &'static str = "ERROR: USER DELETE requires 1 argument: USER DELETE <username>\n";
    pub const ERROR_USER_UPDATE_ARGS: &'static str = "ERROR: USER UPDATE requires 3 arguments: USER UPDATE <username> <field> <value>\n";
    pub const ERROR_USER_LOGIN_ARGS: &'static str = "ERROR: USER LOGIN requires 2 arguments: USER LOGIN <username> <password>\n";
    pub const ERROR_INVALID_USER_COMMAND: &'static str = "ERROR: Invalid USER command. Use: CREATE, LIST, DELETE, UPDATE, LOGIN, LOGOUT, WHOAMI\n";
    pub const ERROR_INVALID_ROLE: &'static str = "ERROR: Invalid role. Valid roles: admin, user\n";
    pub const ERROR_INVALID_UPDATE_FIELD: &'static str = "ERROR: Invalid field. Valid fields: password, role\n";

    // Helper methods for dynamic messages
    pub fn unknown_command(cmd: &str) -> String {
        format!(
            "ERROR: Unknown command '{}'. Type HELP for available commands.\n",
            cmd
        )
    }

    pub fn query_error(err: &str) -> String {
        format!("ERROR: {}\n", err)
    }

    pub fn invalid_condition(condition: &str) -> String {
        format!("Invalid condition format: {}", condition)
    }

    pub fn unsupported_operator(op: &str) -> String {
        format!("Unsupported operator: {}", op)
    }

    pub fn query_results_header(count: usize) -> String {
        format!("Found {} results:\n", count)
    }

    pub fn query_result_item(key: &str, value: &str) -> String {
        format!("{}: {}\n", key, value)
    }

    pub fn user_list_header(count: usize) -> String {
        format!("Found {} users:\n", count)
    }

    pub fn user_list_item(username: &str, role: &str, created_at: &str) -> String {
        format!("  {} (role: {}, created: {})\n", username, role, created_at)
    }

    pub fn user_whoami_response(username: &str, role: &str) -> String {
        format!("Logged in as: {} (role: {})\n", username, role)
    }

    pub fn no_user_logged_in() -> String {
        "No user currently logged in\n".to_string()
    }
}
