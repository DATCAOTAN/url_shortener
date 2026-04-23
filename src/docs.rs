use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use axum::response::Html;

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
        crate::handlers::link_handler::create_link,
        crate::handlers::link_handler::redirect_link,
        crate::handlers::link_handler::get_my_links,
        crate::handlers::link_handler::delete_link,
        crate::handlers::link_handler::get_daily_analytics,
        crate::handlers::admin_handler::list_users,
        crate::handlers::admin_handler::get_user_by_id,
        crate::handlers::admin_handler::soft_delete_user,
        crate::handlers::admin_handler::hard_delete_user,
        crate::handlers::admin_handler::list_links,
        crate::handlers::admin_handler::disable_link
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
        (name = "Links", description = "URL shortener endpoints"),
        (name = "Admin", description = "Administrative endpoints")
    )
)]
pub struct ApiDoc;

#[allow(dead_code)]
pub fn docs_home() -> Html<&'static str> {
        Html(
                r#"<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>URL Shortener API Docs</title>
    <style>
        :root {
            --bg-1: #f7f7f2;
            --bg-2: #e8f0e8;
            --ink: #1f2a1f;
            --ink-soft: #4a5b4a;
            --accent: #1f8a5b;
            --accent-2: #1a6f4a;
            --card: #ffffffcc;
            --border: #bdd0bf;
        }

        * { box-sizing: border-box; }

        body {
            margin: 0;
            min-height: 100vh;
            font-family: "Trebuchet MS", "Segoe UI", sans-serif;
            color: var(--ink);
            background:
                radial-gradient(circle at 10% 10%, #dbead9 0%, transparent 28%),
                radial-gradient(circle at 80% 20%, #d5ebdf 0%, transparent 30%),
                linear-gradient(135deg, var(--bg-1), var(--bg-2));
            display: grid;
            place-items: center;
            padding: 24px;
        }

        .card {
            width: min(820px, 100%);
            background: var(--card);
            border: 1px solid var(--border);
            border-radius: 20px;
            box-shadow: 0 18px 48px rgba(25, 56, 30, 0.12);
            backdrop-filter: blur(4px);
            padding: clamp(20px, 5vw, 42px);
        }

        .eyebrow {
            display: inline-block;
            font-size: 12px;
            font-weight: 700;
            letter-spacing: 0.08em;
            text-transform: uppercase;
            color: #2b5f3c;
            border: 1px solid #9ebda6;
            border-radius: 999px;
            padding: 6px 10px;
            margin-bottom: 12px;
        }

        h1 {
            margin: 0;
            font-size: clamp(28px, 5vw, 44px);
            line-height: 1.1;
        }

        p {
            margin-top: 14px;
            color: var(--ink-soft);
            font-size: 16px;
            line-height: 1.6;
            max-width: 68ch;
        }

        .actions {
            display: flex;
            flex-wrap: wrap;
            gap: 12px;
            margin-top: 24px;
        }

        .btn {
            text-decoration: none;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            min-height: 44px;
            border-radius: 12px;
            padding: 10px 16px;
            font-weight: 700;
            font-size: 15px;
            border: 1px solid transparent;
            transition: transform .2s ease, box-shadow .2s ease;
        }

        .btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 18px rgba(22, 93, 63, 0.2);
        }

        .btn-primary {
            background: var(--accent);
            color: #fff;
        }

        .btn-primary:hover { background: var(--accent-2); }

        .btn-secondary {
            background: #fff;
            color: #244530;
            border-color: #9ebda6;
        }

        code {
            display: inline-block;
            margin-top: 26px;
            background: #f2f7f3;
            border: 1px solid #d2e0d4;
            border-radius: 8px;
            padding: 10px 12px;
            color: #274534;
            font-size: 13px;
            word-break: break-all;
        }
    </style>
</head>
<body>
    <main class="card">
        <span class="eyebrow">Developer Portal</span>
        <h1>URL Shortener API Documentation</h1>
        <p>
            Tài liệu API đã sẵn sàng. Bạn có thể mở Swagger UI để test trực tiếp endpoint,
            hoặc lấy OpenAPI JSON để import vào Postman, Insomnia, hoặc các tool khác.
        </p>

        <div class="actions">
            <a class="btn btn-primary" href="/docs/swagger">Mở Swagger UI</a>
            <a class="btn btn-secondary" href="/api-docs/openapi.json" target="_blank" rel="noopener">Xem OpenAPI JSON</a>
        </div>

        <code>Tip: Swagger UI path = /docs/swagger</code>
    </main>
</body>
</html>
"#,
        )
}

pub fn swagger_ui_page() -> Html<&'static str> {
        Html(
    r##"<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Swagger UI</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
    <style>
        html, body { margin: 0; padding: 0; }
        #swagger-ui { min-height: 100vh; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js" crossorigin></script>
    <script>
        window.onload = function () {
            window.ui = SwaggerUIBundle({
                url: "/api-docs/openapi.json",
                dom_id: "#swagger-ui",
                deepLinking: true,
                displayRequestDuration: true,
                persistAuthorization: true,
            });
        };
    </script>
</body>
</html>
"##,
        )
}
