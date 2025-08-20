# Sharknado Database Engine

A high-performance, TCP-based database engine written in Rust with built-in user authentication and protocol support.

## Features

- TCP server with authentication
- JSON-based data storage
- User management system with role-based access
- Protocol handler for sharknado:// URLs
- CLI-based user administration
- Real-time query operations
- Connection-based authentication sessions

## Installation

```bash
git clone https://github.com/externref/sharknado.git
cd sharknado
cargo build --release
```

## Quick Start

### 1. User Management (CLI Mode)

First, create users using the CLI mode:

```bash
# Start CLI for user management
cargo run -- --cli

# Create an admin user
user create admin admin123 admin

# Create a regular user
user create john password123 user

# List all users (admin only)
user list

# Update user password
user update john password newpass456

# Delete a user
user delete john

# Exit CLI mode
exit
```

### 2. Start Database Server

```bash
# Start server with default database
cargo run

# Start server with custom database name
cargo run my_database
```

### 3. Connect to Database

#### Option A: Protocol Connection
```bash
# Connect using sharknado:// protocol
cargo run -- --connect sharknado://admin:admin123@127.0.0.1:8080
```

#### Option B: TCP Client Connection
```bash
# Connect via TCP and authenticate manually
telnet 127.0.0.1 8080
LOGIN admin admin123
```

## Command Reference

### User Management Commands (CLI Mode Only)

| Command | Description | Permission |
|---------|-------------|------------|
| `user create <username> <password> <role>` | Create new user | Any |
| `user list` | List all users | Admin only |
| `user delete <username>` | Delete user | Admin only |
| `user update <username> <field> <value>` | Update user | Admin only |
| `help` | Show help | Any |
| `exit` | Exit CLI | Any |

**Roles:** `admin`, `user`

### Database Commands (TCP Mode - Requires Authentication)

| Command | Syntax | Description |
|---------|--------|-------------|
| `LOGIN` | `LOGIN <username> <password>` | Authenticate connection |
| `LOGOUT` | `LOGOUT` | End session |
| `SET` | `SET <table> <key> <json_value>` | Store data |
| `GET` | `GET <table> <key>` | Retrieve data |
| `UPDATE` | `UPDATE <table> <key> <json_value>` | Update existing data |
| `DELETE` | `DELETE <table> <key>` | Remove data |
| `QUERY` | `QUERY <table> <conditions>` | Query with conditions |

### Query Conditions

Supported operators for QUERY command:

| Operator | Example | Description |
|----------|---------|-------------|
| `=` | `name = "John"` | Equals |
| `!=` | `age != 25` | Not equals |
| `>` | `score > 100` | Greater than |
| `<` | `price < 50.0` | Less than |
| `>=` | `age >= 18` | Greater than or equal |
| `<=` | `count <= 10` | Less than or equal |
| `contains` | `tags contains "rust"` | String contains |

## Usage Examples

### Basic Data Operations

```bash
# After authentication, store some data
SET users john {"name": "John Doe", "age": 30, "email": "john@example.com"}
SET users jane {"name": "Jane Smith", "age": 25, "email": "jane@example.com"}
SET products laptop {"name": "Gaming Laptop", "price": 1299.99, "category": "electronics"}

# Retrieve data
GET users john
# Output: {"name": "John Doe", "age": 30, "email": "john@example.com"}

GET products laptop
# Output: {"name": "Gaming Laptop", "price": 1299.99, "category": "electronics"}

# Update data
UPDATE users john {"name": "John Doe", "age": 31, "email": "john.doe@example.com"}

# Delete data
DELETE users jane
```

### Advanced Queries

```bash
# Find users older than 25
QUERY users age > 25

# Find products in electronics category
QUERY products category = "electronics"

# Find users with specific email domain
QUERY users email contains "@example.com"

# Find products under $100
QUERY products price < 100.0

# Complex queries with multiple conditions
QUERY users age >= 18 name contains "John"
```

### JSON Data Examples

```bash
# Store complex nested JSON
SET inventory item001 {
  "name": "Wireless Headphones",
  "price": 199.99,
  "stock": 50,
  "specs": {
    "battery": "30 hours",
    "connectivity": ["bluetooth", "usb-c"],
    "weight": "250g"
  },
  "reviews": [
    {"rating": 5, "comment": "Excellent sound quality"},
    {"rating": 4, "comment": "Good value for money"}
  ]
}

# Query nested fields
QUERY inventory specs.battery contains "30"
QUERY inventory price <= 200.0
```

## Configuration

### Command Line Options

```bash
sharknado [OPTIONS] [database-name]

OPTIONS:
    --cli                    User management mode
    --connect <uri>          Connect using sharknado:// protocol
    --register-protocol      Register sharknado:// protocol handler
    --help, -h               Show help message

ARGUMENTS:
    database-name           Database name (default: sharknado_default)
```

### Protocol Registration

Register the sharknado:// protocol for system-wide URL handling:

```bash
cargo run -- --register-protocol
```

After registration, you can use URLs like:
- `sharknado://admin:admin123@localhost:8080`
- `sharknado://user:password@192.168.1.100:8080/my_database`

## Architecture

### Modes of Operation

1. **CLI Mode (`--cli`)**
   - User management only
   - Create, list, update, delete users
   - No database operations

2. **TCP Server Mode (default)**
   - Database operations with authentication
   - Requires valid user credentials
   - Persistent data storage

3. **Client Connection Mode (`--connect`)**
   - Connect to remote Sharknado server
   - Automatic authentication
   - Interactive session

### Authentication Flow

1. Start CLI mode to create users
2. Start TCP server with database
3. Clients connect and authenticate with LOGIN
4. Perform database operations
5. LOGOUT or disconnect to end session

### Data Storage

- JSON-based document storage
- Table-based organization
- Key-value pairs within tables
- Persistent storage to disk
- Automatic log replay on startup

## Error Handling

Common error messages and solutions:

| Error | Solution |
|-------|----------|
| `ERROR: Authentication required` | Use LOGIN command first |
| `ERROR: Invalid credentials` | Check username/password |
| `ERROR: User not found` | Create user in CLI mode |
| `ERROR: Insufficient permissions` | Contact admin for role update |
| `ERROR: Invalid JSON` | Check JSON syntax |

## Performance

- High-performance TCP server
- Concurrent connection handling
- Memory-efficient JSON processing
- Fast query operations
- Minimal latency for local connections

## Security

- Password-based authentication
- Role-based access control
- Connection-based sessions
- Secure credential storage
- Admin-only user management

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo check
```

### Project Structure

```
src/
├── main.rs              # Entry point and connection handling
├── connection.rs        # TCP server implementation
├── engine.rs           # Database engine core
├── user_manager.rs     # User authentication system
├── logs.rs            # Logging system
└── helpers/
    ├── configs.rs      # Protocol registration
    ├── logging.rs      # Log utilities
    ├── messages.rs     # Response messages
    └── mod.rs         # Module declarations
```

## License

This project is licensed under the MIT License.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Support

For issues and questions:
- Create an issue on GitHub
- Check existing documentation
- Review command syntax above