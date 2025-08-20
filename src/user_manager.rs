use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    User,
}

impl UserRole {
    pub fn from_str(role: &str) -> Option<UserRole> {
        match role.to_lowercase().as_str() {
            "admin" => Some(UserRole::Admin),
            "user" => Some(UserRole::User),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::User => "user".to_string(),
        }
    }
}

pub struct UserManager {
    users: RwLock<HashMap<String, User>>,
    current_user: RwLock<Option<String>>, // For CLI mode
    authenticated_connections: RwLock<HashMap<String, String>>, // connection_id -> username for TCP
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            users: RwLock::new(HashMap::new()),
            current_user: RwLock::new(None),
            authenticated_connections: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_user(
        &self,
        username: String,
        password: String,
        role: UserRole,
    ) -> Result<(), String> {
        let mut users = self.users.write().unwrap();

        if users.contains_key(&username) {
            return Err("User already exists".to_string());
        }

        let password_hash = self.hash_password(&password);
        let user = User {
            username: username.clone(),
            password_hash,
            role,
            created_at: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        };

        users.insert(username, user);
        Ok(())
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Result<(), String> {
        let users = self.users.read().unwrap();

        if let Some(user) = users.get(username) {
            if self.verify_password(password, &user.password_hash) {
                let mut current_user = self.current_user.write().unwrap();
                *current_user = Some(username.to_string());
                Ok(())
            } else {
                Err("Invalid credentials".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn logout(&self) {
        let mut current_user = self.current_user.write().unwrap();
        *current_user = None;
    }

    pub fn get_current_user(&self) -> Option<User> {
        let current_user = self.current_user.read().unwrap();
        if let Some(username) = current_user.as_ref() {
            let users = self.users.read().unwrap();
            users.get(username).cloned()
        } else {
            None
        }
    }

    pub fn is_authenticated(&self) -> bool {
        let current_user = self.current_user.read().unwrap();
        current_user.is_some()
    }

    pub fn is_admin(&self) -> bool {
        if let Some(user) = self.get_current_user() {
            user.role == UserRole::Admin
        } else {
            false
        }
    }

    pub fn delete_user(&self, username: &str) -> Result<(), String> {
        if !self.is_admin() {
            return Err("Insufficient permissions".to_string());
        }

        let mut users = self.users.write().unwrap();

        if users.remove(username).is_some() {
            let current_user = self.current_user.read().unwrap();
            if let Some(current) = current_user.as_ref() {
                if current == username {
                    drop(current_user);
                    self.logout();
                }
            }
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn update_user(&self, username: &str, field: &str, value: &str) -> Result<(), String> {
        if !self.is_admin()
            && self.get_current_user().map(|u| u.username) != Some(username.to_string())
        {
            return Err("Insufficient permissions".to_string());
        }

        let mut users = self.users.write().unwrap();

        if let Some(user) = users.get_mut(username) {
            match field {
                "password" => {
                    user.password_hash = self.hash_password(value);
                }
                "role" => {
                    if !self.is_admin() {
                        return Err("Only admins can change roles".to_string());
                    }
                    if let Some(role) = UserRole::from_str(value) {
                        user.role = role;
                    } else {
                        return Err("Invalid role".to_string());
                    }
                }
                _ => return Err("Invalid field".to_string()),
            }
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn list_users(&self) -> Vec<User> {
        let users = self.users.read().unwrap();
        users.values().cloned().collect()
    }

    fn hash_password(&self, password: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn verify_password(&self, password: &str, hash: &str) -> bool {
        self.hash_password(password) == hash
    }

    pub fn ensure_default_admin(&self) {
        let users = self.users.read().unwrap();
        if users.is_empty() {
            drop(users);
            let _ = self.create_user("admin".to_string(), "admin123".to_string(), UserRole::Admin);
        }
    }

    pub fn authenticate_connection(
        &self,
        connection_id: &str,
        username: &str,
        password: &str,
    ) -> Result<(), String> {
        let users = self.users.read().unwrap();

        if let Some(user) = users.get(username) {
            if self.verify_password(password, &user.password_hash) {
                let mut connections = self.authenticated_connections.write().unwrap();
                connections.insert(connection_id.to_string(), username.to_string());
                Ok(())
            } else {
                Err("Invalid credentials".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn logout_connection(&self, connection_id: &str) {
        let mut connections = self.authenticated_connections.write().unwrap();
        connections.remove(connection_id);
    }

    pub fn is_connection_authenticated(&self, connection_id: &str) -> bool {
        let connections = self.authenticated_connections.read().unwrap();
        connections.contains_key(connection_id)
    }

    pub fn get_connection_user(&self, connection_id: &str) -> Option<User> {
        let connections = self.authenticated_connections.read().unwrap();
        if let Some(username) = connections.get(connection_id) {
            let users = self.users.read().unwrap();
            users.get(username).cloned()
        } else {
            None
        }
    }

    pub fn cleanup_connection(&self, connection_id: &str) {
        self.logout_connection(connection_id);
    }
}
