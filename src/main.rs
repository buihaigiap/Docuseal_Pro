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
        routes::templates::get_template_full_info,
        routes::templates::update_template,
        routes::templates::delete_template,
        routes::templates::clone_template,
        routes::templates::create_template_from_html,
        routes::templates::create_template_from_pdf,
        routes::templates::create_template_from_docx,
        routes::templates::merge_templates,
        routes::templates::download_file,
        routes::templates::preview_file,
        routes::templates::get_template_fields,
        routes::templates::create_template_field,
        routes::templates::upload_template_field_file,
        routes::templates::update_template_field,
        routes::templates::delete_template_field,
        routes::submissions::create_submission,
        routes::submitters::get_public_submitter_fields,
        routes::submitters::get_public_submitter_signatures,
        routes::submitters::get_public_submitter,
        routes::submitters::update_public_submitter,
        routes::submitters::submit_bulk_signatures,
        routes::submitters::get_submitters,
        routes::submitters::get_submitter,
        routes::submitters::update_submitter,
        routes::submitters::delete_submitter,
        routes::submitters::get_me
        // routes::subscription::get_subscription_status,
        // routes::subscription::get_payment_link
    ),
    components(
        schemas(
            common::requests::RegisterRequest,
            common::requests::LoginRequest,
            common::responses::ApiResponse<User>,
            common::responses::ApiResponse<common::responses::LoginResponse>,
            common::responses::ApiResponse<Vec<Template>>,
            common::responses::ApiResponse<Template>,
            common::responses::ApiResponse<models::submitter::Submitter>,
            common::responses::ApiResponse<Vec<models::submitter::Submitter>>,
            common::responses::ApiResponse<String>,
            common::responses::ApiResponse<Vec<models::template::TemplateField>>,
            common::responses::ApiResponse<models::template::TemplateField>,
            models::role::Role,
            models::template::CreateTemplateFieldRequest,
            models::template::UpdateTemplateFieldRequest,
            models::template::FieldPosition,
            models::template::TemplateField,
            models::submitter::PublicSubmitterFieldsResponse,
            models::submitter::PublicSubmitterSignaturesResponse
            // models::user::UserSubscriptionStatus,
            // models::user::CreatePaymentRequest,
            // routes::subscription::SubscriptionStatusResponse,
            // routes::subscription::PaymentLinkResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "templates", description = "Template management endpoints"),
        (name = "template_fields", description = "Template field management endpoints"),
        (name = "submissions", description = "Document submission endpoints"),
        (name = "submitters", description = "Submitter management endpoints")
        // (name = "subscription", description = "Subscription and billing endpoints")
    ),
    security(("bearer_auth" = [])),
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

    // Run database migrations automatically on startup
    println!("Running database migrations...");
    run_migrations(&pool).await.expect("Failed to run database migrations");
    println!("✅ Database migrations completed successfully");

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
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running on http://{}", addr);
    println!("Swagger UI: http://{}/swagger-ui", addr);
    println!("API Base URL: http://{}/api", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}

