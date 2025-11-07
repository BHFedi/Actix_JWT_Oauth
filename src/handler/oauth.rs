use actix_web::{
    cookie::{time::Duration as ActixWebDuration, Cookie},
    web, HttpResponse, Scope,
};
use serde::{Deserialize, Serialize};
use url::Url;
use crate::{
    db::UserExt,
    error::HttpError,
    // Note: We use the mock-only enum here
    models::OAuthProvider, 
    // Note: We use the generic OAuth2Client struct
    utils::{oauth_client::{OAuth2Client, generate_state}, token}, 
    AppState,
};

// --- Type Definitions (Remain the same) ---

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    code: String,
    state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthAuthorizeResponse {
    pub authorization_url: String,
}

// ============================================================================
// Mock OAuth2 Handler (Replaces GitHub/Google Handlers)
// ============================================================================

pub fn oauth_handler() -> Scope {
    web::scope("/api/oauth")
        // Use single, generic routes pointing to the mock handler
        .route("/mock/authorize", web::get().to(mock_authorize))
        .route("/mock/callback", web::get().to(mock_callback))
}

#[utoipa::path(
    get,
    path = "/api/oauth/mock/authorize",
    tag = "OAuth - Mock Server",
    responses(
        (status = 302, description = "Redirect to Mock authorization page"),
        (status = 500, description = "Internal Server Error"),
    )
)]
pub async fn mock_authorize(app_state: web::Data<AppState>) -> Result<HttpResponse, HttpError> {
    let state = generate_state();
    let oauth_config = &app_state.env.mock_oauth;

    let mut auth_url = Url::parse(&oauth_config.authorize_url)
        .map_err(|e| HttpError::server_error(format!("Invalid Auth URL: {}", e)))?;

    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &oauth_config.client_id)
        .append_pair("redirect_uri", &oauth_config.redirect_uri)
        // Standard OIDC scopes
        .append_pair("scope", "openid email profile") 
        .append_pair("state", &state)
        .append_pair("response_type", "code");
        
    // In a production app, you might set a cookie here to store 'state' for CSRF protection.

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url.as_str()))
        .finish())
}

#[utoipa::path(
    get,
    path = "/api/oauth/mock/callback",
    tag = "OAuth - Mock Server",
    params(
        ("code" = String, Query, description = "Authorization code from Mock Server"),
        ("state" = Option<String>, Query, description = "State parameter for CSRF protection")
    ),
    responses(
        (status = 302, description = "Redirect to frontend with authentication token"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error"),
    )
)]
pub async fn mock_callback(
    app_state: web::Data<AppState>,
    query: web::Query<OAuthCallbackQuery>,
) -> Result<HttpResponse, HttpError> {
    let oauth_client = OAuth2Client::new();
    let oauth_config = &app_state.env.mock_oauth;
    
    // 1. Exchange code for access token using generic method and mock config
    let token_response = oauth_client
        .exchange_code(
            &query.code,
            &oauth_config.client_id,
            &oauth_config.client_secret,
            &oauth_config.redirect_uri,
            &oauth_config.token_url,
        )
        .await
        .map_err(|e| HttpError::server_error(format!("Failed to exchange code with Mock Server: {}", e)))?;

    // 2. Get user info using generic method and mock config
    let user_info = oauth_client
        .get_user_info(&token_response.access_token, &oauth_config.userinfo_url)
        .await
        .map_err(|e| HttpError::server_error(format!("Failed to get user info from Mock Server: {}", e)))?;
        
    let email = user_info.email;

    // The provider for the database is always 'Mock'
    let db_provider = OAuthProvider::Mock; 
    
    // The provider_user_id is the 'sub' field from the UserInfo response
    let provider_user_id = user_info.sub;

    // 3. Check if OAuth account already exists
    let oauth_account = app_state
        .db_client
        .get_oauth_account(db_provider, &provider_user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = if let Some(oauth_acc) = oauth_account {
        // OAuth account exists, retrieve the user
        app_state
            .db_client
            .get_user(Some(oauth_acc.user_id), None, None)
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?
            .ok_or_else(|| HttpError::server_error("Linked user not found".to_string()))?
    } else {
        // 4. Create or link user
        
        // Check if user with this email already exists (linking accounts)
        let existing_user = app_state
            .db_client
            .get_user(None, None, Some(&email))
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;

        let user = if let Some(existing) = existing_user {
            // Link OAuth account to existing user
            existing
        } else {
            // Create new user (Just-in-Time Provisioning)
            let name = user_info.name;
            let photo = user_info.picture.unwrap_or_else(|| "default.png".to_string());
            
            // Note: We use the string "mock" for the database column
            app_state 
                .db_client
                .save_oauth_user(&name, &email, &photo, "mock") 
                .await
                .map_err(|e| HttpError::server_error(e.to_string()))?
        };

        // Save new OAuth account link
        app_state
            .db_client
            .save_oauth_account(
                user.id,
                db_provider,
                provider_user_id.clone(),
                Some(token_response.access_token),
                token_response.refresh_token,
                token_response.expires_in.map(|exp| chrono::Utc::now() + chrono::Duration::seconds(exp)),
            )
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;
            
        user
    };

    // 5. Generate and send JWT token
    let jwt_token = token::create_token(
        &user.id.to_string(),
        app_state.env.jwt_secret.as_bytes(),
        app_state.env.jwt_maxage,
    )
    .map_err(|e| HttpError::server_error(e.to_string()))?;

    // Set cookie
    let cookie = Cookie::build("token", jwt_token.clone())
        .path("/")
        .max_age(ActixWebDuration::new(60 * app_state.env.jwt_maxage, 0))
        .http_only(true)
        .finish();

    // Redirect to frontend with token
    let redirect_url = format!("{}?token={}", app_state.env.oauth_success_redirect, jwt_token);
    
    Ok(HttpResponse::Found()
        .cookie(cookie)
        .append_header(("Location", redirect_url))
        .finish())
}