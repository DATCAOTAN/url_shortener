use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health_handler::liveness,
        crate::handlers::health_handler::readiness,
        crate::handlers::user_handler::register_user,
        crate::handlers::user_handler::login_user,
        crate::handlers::user_handler::refresh_token,
        crate::handlers::user_handler::logout_user,
        crate::handlers::user_handler::get_me,
        crate::handlers::user_handler::get_user,
        crate::handlers::link_handler::create_link,
        crate::handlers::link_handler::redirect_link,
        crate::handlers::link_handler::get_my_links,
        crate::handlers::link_handler::delete_link,
        crate::handlers::link_handler::get_daily_analytics
    ),
    components(
        schemas(
            crate::dtos::user::RegisterUser,
            crate::dtos::user::LoginUser,
            crate::dtos::user::LoginResponse,
            crate::dtos::user::RefreshTokenRequest,
            crate::dtos::user::RefreshTokenResponse,
            crate::dtos::user::LogoutRequest,
            crate::dtos::user::LogoutResponse,
            crate::dtos::user::UserResponse,
            crate::dtos::link::CreateLinkRequest,
            crate::dtos::link::LinkResponse,
            crate::dtos::link::DeleteLinkResponse,
            crate::dtos::link::DailyAnalyticsResponse,
            crate::handlers::link_handler::AnalyticsQuery,
            crate::handlers::health_handler::HealthResponse,
            crate::handlers::health_handler::ReadyResponse,
            crate::error::ErrorResponse
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Users", description = "User profile endpoints"),
        (name = "Links", description = "URL shortener endpoints")
    )
)]
pub struct ApiDoc;
