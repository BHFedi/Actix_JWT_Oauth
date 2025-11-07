use crate::models::{User, UserRole, OAuthProvider, OAuthAccount};
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct DBClient {
    pool: Pool<Postgres>,
}

impl DBClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        DBClient { pool }
    }
}

#[async_trait]
pub trait UserExt {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>
    ) -> Result<Option<User>, sqlx::Error>;

    async fn get_users(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<User>, sqlx::Error>;

    async fn save_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error>;

    async fn save_admin_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error>;

    // OAuth-specific methods
    async fn save_oauth_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        photo: T,
        provider: &str,
    ) -> Result<User, sqlx::Error>;

    async fn get_oauth_account(
        &self,
        provider: OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccount>, sqlx::Error>;

    async fn save_oauth_account(
        &self,
        user_id: Uuid,
        provider: OAuthProvider,
        provider_user_id: String,
        access_token: Option<String>,
        refresh_token: Option<String>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<OAuthAccount, sqlx::Error>;

    async fn update_oauth_tokens(
        &self,
        oauth_account_id: Uuid,
        access_token: Option<String>,
        refresh_token: Option<String>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl UserExt for DBClient {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>
    ) -> Result<Option<User>, sqlx::Error> {
        let mut user: Option<User> = None;
        if let Some(user_id) = user_id {
            // UPDATED: Removed COALESCE on password. auth_provider is also correctly selected.
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, photo, verified, created_at, updated_at, role as "role: UserRole", auth_provider FROM users WHERE id = $1"#,
                user_id
            ).fetch_optional(&self.pool).await?;
        }else if let Some(name) = name {
            // UPDATED: Removed COALESCE on password
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, photo, verified, created_at, updated_at, role as "role: UserRole", auth_provider FROM users WHERE name = $1"#,
                name
            ).fetch_optional(&self.pool).await?;
        } else if let Some(email) = email {
            // UPDATED: Removed COALESCE on password
            user = sqlx::query_as!(
                User,
                r#"SELECT id, name, email, password, photo, verified, created_at, updated_at, role as "role: UserRole", auth_provider FROM users WHERE email = $1"#,
                email
            ).fetch_optional(&self.pool).await?;
        }
        Ok(user)
    }

    async fn get_users(&self, page: u32, limit: usize) -> Result<Vec<User>, sqlx::Error> {
        let offset = (page - 1) * limit as u32;
        let users = sqlx::query_as!(
            User,
            // UPDATED: Removed COALESCE on password
            r#"SELECT id, name, email, password, photo, verified, created_at, updated_at, role as "role: UserRole", auth_provider FROM users
             ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            limit as i64,
            offset as i64,
        ).fetch_all(&self.pool)
        .await?;
        Ok(users)
    }
    
    // --- save_user and save_admin_user are correct for local users ---
    // They insert a non-NULL password and 'local' auth_provider.
    
    async fn save_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            // UPDATED: Removed COALESCE on password in RETURNING clause
            r#"INSERT INTO users (name, email, password, auth_provider) VALUES ($1, $2, $3, 'local') RETURNING id, name, email, password, photo, verified, created_at, updated_at, role as "role: UserRole", auth_provider"#,
            name.into(),
            email.into(),
            password.into(),
        ).fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn save_admin_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        password: T,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            // UPDATED: Removed COALESCE on password in RETURNING clause
            r#"INSERT INTO users (name, email, password, role, auth_provider) VALUES ($1, $2, $3, $4, 'local') RETURNING id, name, email, password, photo, verified, created_at, updated_at, role as "role: UserRole", auth_provider"#,
            name.into(),
            email.into(),
            password.into(),
            UserRole::Admin as UserRole,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    // --- OAuth-specific methods ---

    async fn save_oauth_user<T: Into<String> + Send>(
        &self,
        name: T,
        email: T,
        photo: T,
        provider: &str,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO users (name, email, photo, verified, auth_provider, password) 
             VALUES ($1, $2, $3, true, $4, NULL) 
             RETURNING id, name, email, password, photo, verified, created_at, updated_at, role as "role: UserRole", auth_provider"#,
            name.into(),
            email.into(),
            photo.into(),
            provider,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn get_oauth_account(
        &self,
        provider: OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccount>, sqlx::Error> {
        let oauth_account = sqlx::query_as!(
            OAuthAccount,
            r#"SELECT id, user_id, provider as "provider: OAuthProvider", provider_user_id, access_token, refresh_token, token_expires_at, created_at, updated_at 
             FROM oauth_accounts 
             WHERE provider = $1 AND provider_user_id = $2"#,
            provider as OAuthProvider,
            provider_user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(oauth_account)
    }

    async fn save_oauth_account(
        &self,
        user_id: Uuid,
        provider: OAuthProvider,
        provider_user_id: String,
        access_token: Option<String>,
        refresh_token: Option<String>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<OAuthAccount, sqlx::Error> {
        let oauth_account = sqlx::query_as!(
            OAuthAccount,
            r#"INSERT INTO oauth_accounts (user_id, provider, provider_user_id, access_token, refresh_token, token_expires_at) 
            VALUES ($1, $2, $3, $4, $5, $6) 
            RETURNING id, user_id, provider as "provider: OAuthProvider", provider_user_id, access_token, refresh_token, token_expires_at, created_at, updated_at"#,
            user_id,
            provider as OAuthProvider,
            provider_user_id,
            access_token,
            refresh_token,
            token_expires_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(oauth_account)
    }

    async fn update_oauth_tokens(
        &self,
        oauth_account_id: Uuid,
        access_token: Option<String>,
        refresh_token: Option<String>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE oauth_accounts SET access_token = $1, refresh_token = $2, token_expires_at = $3, updated_at = NOW() WHERE id = $4",
            access_token,
            refresh_token,
            token_expires_at,
            oauth_account_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}