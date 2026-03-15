use axum::{
	extract::Request,
	http::{StatusCode, header::AUTHORIZATION},
	middleware::Next,
	response::Response,
};

use crate::{
	dtos::claims::Claims,
	utils::jwt::verify_jwt,
};

pub async fn auth_guard(mut req: Request, next: Next) -> Result<Response, StatusCode> {
	let auth_header = req
		.headers()
		.get(AUTHORIZATION)
		.and_then(|value| value.to_str().ok())
		.ok_or(StatusCode::UNAUTHORIZED)?;

	let token = auth_header
		.strip_prefix("Bearer ")
		.ok_or(StatusCode::UNAUTHORIZED)?;

	let claims: Claims = verify_jwt(token)?;
	req.extensions_mut().insert(claims);

	Ok(next.run(req).await)
}
