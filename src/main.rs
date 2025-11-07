mod auth;
mod config;
mod db;
mod dtos;
mod error;
mod handlers;
mod models;
mod utils;

use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web, App, HttpServer};
use config::Config;
use db::DBClient;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct AppState {
    db_client: DBClient,
    env: Config,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::logout,
        // CHANGED: Use the generic mock_authorize and mock_callback handlers
        handlers::oauth::mock_authorize,
        handlers::oauth::mock_callback,
    ),
    components(schemas(
        dtos::RegisterUserDto,
        dtos::LoginUserDto,
        dtos::FilterUserDto,
        dtos::UserData,
        dtos::UserResponseDto,
        dtos::UserLoginResponseDto,
        dtos::UserListResponseDto,
        dtos::Response,
    )),
    tags(
        (name = "Register Account Endpoint", description = "User registration endpoints"),
        (name = "Login Endpoint", description = "User login endpoints"),
        (name = "Logout Endpoint", description = "User logout endpoints"),
        // CHANGED: Use a single, generic tag for the mock server
        (name = "OAuth - Mock Server", description = "Local Mock OAuth2 authentication for testing"), 
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

// --- SecurityAddon remains unchanged ---

struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpAuthScheme::Bearer,
                ),
            )
        }
    }
}

// --- main function remains unchanged ---

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    dotenv().ok();
    env_logger::init();

    let config = Config::init();

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
    {
        Ok(pool) => {
            println!("✅ Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("❌ Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let db_client = DBClient::new(pool);

    println!("🚀 Server started successfully on http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(AppState {
                db_client: db_client.clone(),
                env: config.clone(),
            }))
            .wrap(cors)
            .wrap(Logger::default())
            .service(handlers::auth::auth_handler())
            .service(handlers::oauth::oauth_handler()) // Add OAuth routes
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}