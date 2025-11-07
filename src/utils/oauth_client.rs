use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- 1. Generic Token Response remains the same ---
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}

// --- 2. Generic User Info Struct (Replaces GitHub/Google) ---
// This reflects the standard claims returned by the OIDC UserInfo endpoint,
// which your mock server should emulate.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericUserInfo {
    // The subject identifier for the user (required)
    pub sub: String, 
    // The user's full name
    pub name: String, 
    // The user's primary email address
    pub email: String, 
    // URL of the user's picture
    pub picture: Option<String>, 
    // Boolean indicating if the email address is verified
    pub email_verified: Option<bool>,
}

// --- 3. Updated OAuth2Client Implementation (Generic Endpoints) ---
pub struct OAuth2Client {
    client: Client,
}

impl OAuth2Client {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Exchanges an authorization code for an OAuth token using dynamically provided URLs.
    pub async fn exchange_code(
        &self,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        token_url: &str, // Dynamic URL from Config
    ) -> Result<OAuthTokenResponse, Box<dyn std::error::Error>> {
        let mut params = HashMap::new();
        params.insert("client_id", client_id);
        params.insert("client_secret", client_secret);
        params.insert("code", code);
        params.insert("redirect_uri", redirect_uri);
        params.insert("grant_type", "authorization_code"); // Standard for most OAuth2 flows

        let response = self
            .client
            .post(token_url) // Use dynamic token_url
            .form(&params)
            .send()
            .await?;

        // Check for common error structure before attempting to deserialize success
        if response.status().is_client_error() || response.status().is_server_error() {
            let error_text = response.text().await?;
            return Err(format!("Token exchange failed: {}", error_text).into());
        }

        let token_response: OAuthTokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Fetches user profile information using dynamically provided URLs.
    pub async fn get_user_info(
        &self,
        access_token: &str,
        userinfo_url: &str, // Dynamic URL from Config
    ) -> Result<GenericUserInfo, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(userinfo_url) // Use dynamic userinfo_url
            .header("Authorization", format!("Bearer {}", access_token))
            // User-Agent might be required by some mock servers, but is often optional
            // .header("User-Agent", "Rust-OAuth-App") 
            .send()
            .await?;

        // Check for errors
        if response.status().is_client_error() || response.status().is_server_error() {
            let error_text = response.text().await?;
            return Err(format!("UserInfo fetch failed: {}", error_text).into());
        }
            
        let user_info: GenericUserInfo = response.json().await?;
        Ok(user_info)
    }
}

// --- Helper function remains the same ---
// You will need to add rand as a dependency if you haven't already.
pub fn generate_state() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const STATE_LEN: usize = 32;
    let mut rng = rand::thread_rng();
    (0..STATE_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}