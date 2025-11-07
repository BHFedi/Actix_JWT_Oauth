use chrono::prelude::*;
use serde::{Deserialize, Serialize};

// --- UserRole remains unchanged ---

#[derive(Debug, Deserialize, Serialize, Clone, Copy, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Moderator,
    User
}

impl UserRole {
    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::Moderator => "moderator",
            UserRole::User => "user"
        }
    }
}

// --- Updated User struct ---

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    // CHANGED: Must be an Option<String> for OAuth-only users
    pub password: Option<String>, 
    pub role: UserRole,
    pub photo: String,
    pub verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    // CHANGED: auth_provider is now NOT NULL with a default in the DB, 
    // but Option<String> is safer if you use it in queries that might return null. 
    // Based on your UP SQL (NOT NULL DEFAULT 'local'), we'll keep it as String for consistency, 
    // but if you remove the DEFAULT, it should be Option<String>. 
    // Sticking to Option<String> for maximum flexibility.
    pub auth_provider: Option<String>,
}

// --- Updated OAuthProvider enum (Mock Only) ---

// Corresponds to the 'oauth_provider' enum in the database with only 'mock'
#[derive(Debug, Deserialize, Serialize, Clone, Copy, sqlx::Type, PartialEq)]
#[sqlx(type_name = "oauth_provider", rename_all = "lowercase")]
pub enum OAuthProvider {
    Mock,
}

impl OAuthProvider {
    pub fn to_str(&self) -> &str {
        match self {
            // Only the mock variant remains
            OAuthProvider::Mock => "mock",
        }
    }
    
    // Simplifed from_str implementation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mock" => Some(OAuthProvider::Mock),
            _ => None,
        }
    }
}

// --- OAuthAccount model remains largely the same ---

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, Clone)]
pub struct OAuthAccount {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    // The provider field is now the Mock-only enum
    pub provider: OAuthProvider, 
    pub provider_user_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}