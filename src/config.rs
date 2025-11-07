use serde::Deserialize;

// Define a separate struct for the generic/mock OAuth provider details
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthProviderConfig {
    pub client_id: String,
    pub client_secret: String,
    pub authorize_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub redirect_uri: String,
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE")
            .expect("JWT_MAXAGE must be set")
            .parse::<i64>()
            .expect("JWT_MAXAGE must be a number");
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        
        // --- 1. Load MOCK OAuth Configuration ---
        let mock_client_id = std::env::var("MOCK_OAUTH_CLIENT_ID").expect("MOCK_OAUTH_CLIENT_ID must be set");
        let mock_client_secret = std::env::var("MOCK_OAUTH_CLIENT_SECRET").expect("MOCK_OAUTH_CLIENT_SECRET must be set");
        
        // Note: These URLs were in your suggested .env setup
        let mock_authorize_url = std::env::var("MOCK_OAUTH_AUTHORIZE_URL").expect("MOCK_OAUTH_AUTHORIZE_URL must be set");
        let mock_token_url = std::env::var("MOCK_OAUTH_TOKEN_URL").expect("MOCK_OAUTH_TOKEN_URL must be set");
        let mock_userinfo_url = std::env::var("MOCK_OAUTH_USERINFO_URL").expect("MOCK_OAUTH_USERINFO_URL must be set");
        
        // Define the default redirect URI for the mock server
        let mock_redirect_uri = std::env::var("MOCK_OAUTH_REDIRECT_URI")
            .unwrap_or_else(|_| format!("{}/api/oauth/mock/callback", app_url));

        let mock_oauth = OAuthProviderConfig {
            client_id: mock_client_id,
            client_secret: mock_client_secret,
            authorize_url: mock_authorize_url,
            token_url: mock_token_url,
            userinfo_url: mock_userinfo_url,
            redirect_uri: mock_redirect_uri,
        };

        // --- 2. Load Frontend Redirect ---
        let oauth_success_redirect = std::env::var("OAUTH_SUCCESS_REDIRECT")
            .unwrap_or_else(|_| "http://localhost:3000/dashboard".to_string());

        Config {
            database_url,
            jwt_secret,
            jwt_maxage,
            app_url,
            // Only include the mock config
            mock_oauth, 
            oauth_success_redirect,
        }
    }
}&

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_maxage: i64,

    // Core Application URL
    pub app_url: String,
    
    // MOCK OAuth2 Configuration
    // Use a new field to hold the structured mock config
    pub mock_oauth: OAuthProviderConfig,

    // Frontend redirect after OAuth success
    pub oauth_success_redirect: String,
}