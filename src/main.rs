mod common;
mod routes;
mod services;
mod models;
mod database;

use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::openapi::security::{HttpAuthScheme, Http, SecurityScheme};

use routes::web::{create_router, AppState};
use database::connection::establish_connection;
use models::user::User;
use models::template::Template;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::web::register_handler,
        routes::web::login_handler,
        routes::templates::get_templates,
        routes::templates::get_template,
        routes::templates::update_template,
        routes::templates::delete_template,
        routes::templates::clone_template,
        routes::templates::create_template_from_html,
        routes::templates::create_template_from_pdf,
        routes::templates::create_template_from_docx,
        routes::templates::merge_templates,
        routes::templates::download_file,
        routes::templates::preview_file
    ),
    components(
        schemas(common::requests::RegisterRequest, common::requests::LoginRequest, common::responses::ApiResponse<User>, common::responses::ApiResponse<common::responses::LoginResponse>, common::responses::ApiResponse<Vec<Template>>, common::responses::ApiResponse<Template>)
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "templates", description = "Template management endpoints"),
        (name = "submissions", description = "Document submission endpoints"),
        (name = "submitters", description = "Submitter management endpoints")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // Load environment variables
    match dotenvy::dotenv() {
        Ok(path) => println!("Loaded .env file from: {:?}", path),
        Err(e) => println!("Failed to load .env file: {}", e),
    }

    // Check if DATABASE_URL is set
    match std::env::var("DATABASE_URL") {
        Ok(url) => println!("DATABASE_URL: {}", url),
        Err(e) => {
            println!("DATABASE_URL not set: {}", e);
            std::process::exit(1);
        }
    }

    // Initialize database connection
    let pool = establish_connection().await.expect("Failed to connect to database");
    // Skip migrations for now - tables will be created manually
    // run_migrations(&pool).await.expect("Failed to run database migrations");

    // Initialize services
    let app_state: AppState = Arc::new(Mutex::new(pool));

    // Create API routes
    let api_routes = create_router();

    // Create custom OpenAPI route with security scheme
    let openapi_json = {
        let mut openapi = ApiDoc::openapi();
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme("bearer_auth", utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::Http::new(utoipa::openapi::security::HttpAuthScheme::Bearer)
            ));
        } else {
            let mut components = utoipa::openapi::Components::new();
            components.add_security_scheme("bearer_auth", utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::Http::new(utoipa::openapi::security::HttpAuthScheme::Bearer)
            ));
            openapi.components = Some(components);
        }
        openapi
    };

    // Create Swagger routes
    let swagger_routes = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", openapi_json);

    // Combine all routes
    let app = Router::new()
        .merge(api_routes)
        .merge(swagger_routes)
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Run server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Server running on http://{}", addr);
    println!("Swagger UI: http://{}/swagger-ui", addr);
    println!("API Base URL: http://{}/api", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}