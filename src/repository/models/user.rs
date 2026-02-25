// ============================================================
// User Model
// ============================================================
//! User entity and related structures.
//!
//! This module defines the User model and related types
//! for user authentication and management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User role for authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[repr(i32)]
pub enum UserRole {
    /// Regular user
    User = 0,
    /// Administrator with full access
    Admin = 1,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::User
    }
}

/// User account entity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    /// Unique user ID
    pub id: Uuid,
    /// Unique username
    pub username: String,
    /// Email address (must be unique)
    pub email: String,
    /// Password hash (bcrypt)
    pub password_hash: String,
    /// Display name (optional)
    pub display_name: Option<String>,
    /// User role
    pub role: UserRole,
    /// Whether the account is active
    pub is_active: bool,
    /// Email verification status
    pub is_verified: bool,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Last login timestamp (optional)
    pub last_login_at: Option<DateTime<Utc>>,
}

/// User creation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    /// Desired username
    pub username: String,
    /// Email address
    pub email: String,
    /// Plain text password (will be hashed)
    pub password: String,
    /// Optional display name
    pub display_name: Option<String>,
}

/// User update data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    /// Optional new username
    pub username: Option<String>,
    /// Optional new email
    pub email: Option<String>,
    /// Optional new display name
    pub display_name: Option<String>,
    /// Optional new password (will be hashed if provided)
    pub password: Option<String>,
}

/// User login credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCredentials {
    /// Username or email
    pub identifier: String,
    /// Plain text password
    pub password: String,
}

/// User profile response (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User ID
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// Display name
    pub display_name: Option<String>,
    /// User role
    pub role: UserRole,
    /// Account status
    pub is_active: bool,
    /// Verification status
    pub is_verified: bool,
    /// Account creation date
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            role: user.role,
            is_active: user.is_active,
            is_verified: user.is_verified,
            created_at: user.created_at,
        }
    }
}

impl User {
    /// Creates a new user from creation data
    ///
    /// # Arguments
    ///
    /// * `data` - User creation data
    ///
    /// # Returns
    ///
    /// A new User instance with hashed password
    ///
    /// # Errors
    ///
    /// Returns an error if password hashing fails
    pub fn create(data: CreateUser) -> Result<Self, bcrypt::BcryptError> {
        use bcrypt::{hash, DEFAULT_COST};

        let password_hash = hash(&data.password, DEFAULT_COST)?;
        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            username: data.username,
            email: data.email,
            password_hash,
            display_name: data.display_name,
            role: UserRole::default(),
            is_active: true,
            is_verified: false,
            created_at: now,
            updated_at: now,
            last_login_at: None,
        })
    }

    /// Updates the user with new data
    ///
    /// # Arguments
    ///
    /// * `data` - Update data
    ///
    /// # Returns
    ///
    /// The updated user (or error if password hashing fails)
    ///
    /// # Errors
    ///
    /// Returns an error if password hashing fails
    pub fn update(&mut self, data: UpdateUser) -> Result<(), bcrypt::BcryptError> {
        use bcrypt::{hash, DEFAULT_COST};

        if let Some(username) = data.username {
            self.username = username;
        }
        if let Some(email) = data.email {
            self.email = email;
        }
        if let Some(display_name) = data.display_name {
            self.display_name = Some(display_name);
        }
        if let Some(password) = data.password {
            self.password_hash = hash(&password, DEFAULT_COST)?;
        }
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Verifies the password against the hash
    ///
    /// # Arguments
    ///
    /// * `password` - Plain text password to verify
    ///
    /// # Returns
    ///
    /// true if the password matches
    #[must_use]
    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }

    /// Converts to a user profile (without sensitive data)
    #[must_use]
    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            display_name: self.display_name.clone(),
            role: self.role,
            is_active: self.is_active,
            is_verified: self.is_verified,
            created_at: self.created_at,
        }
    }

    /// Records a successful login
    pub fn record_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Checks if the user is an admin
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }

    /// Checks if the user account is active and verified
    #[must_use]
    pub const fn can_login(&self) -> bool {
        self.is_active && self.is_verified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_default() {
        assert_eq!(UserRole::default(), UserRole::User);
    }

    #[test]
    fn test_user_role_values() {
        assert_eq!(UserRole::User as i32, 0);
        assert_eq!(UserRole::Admin as i32, 1);
    }

    #[test]
    fn test_create_user() {
        let data = CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            display_name: Some("Test User".to_string()),
        };

        let user = User::create(data);
        assert!(user.is_ok());

        let user = user.unwrap();
        assert!(!user.id.is_nil());
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(!user.password_hash.is_empty());
        assert_eq!(user.display_name, Some("Test User".to_string()));
        assert!(user.is_active);
        assert!(!user.is_verified);
    }

    #[test]
    fn test_verify_password() {
        let data = CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            display_name: None,
        };

        let user = User::create(data).unwrap();
        assert!(user.verify_password("password123"));
        assert!(!user.verify_password("wrongpassword"));
    }

    #[test]
    fn test_user_permissions() {
        let mut user = User::create(CreateUser {
            username: "regular".to_string(),
            email: "regular@example.com".to_string(),
            password: "pass".to_string(),
            display_name: None,
        }).unwrap();

        assert!(!user.is_admin());
        assert!(user.can_login()); // Active but not verified is okay for login check
        user.is_verified = true;
        assert!(user.can_login());

        user.role = UserRole::Admin;
        assert!(user.is_admin());
    }

    #[test]
    fn test_to_profile() {
        let user = User::create(CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            display_name: Some("Test User".to_string()),
        }).unwrap();

        let profile = user.to_profile();
        assert_eq!(profile.id, user.id);
        assert_eq!(profile.username, user.username);
        assert_eq!(profile.email, user.email);
        // Profile should not include password hash
    }

    #[test]
    fn test_record_login() {
        let mut user = User::create(CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            display_name: None,
        }).unwrap();

        assert!(user.last_login_at.is_none());
        user.record_login();
        assert!(user.last_login_at.is_some());
    }
}
