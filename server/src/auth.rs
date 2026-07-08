use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: i64,
    pub exp: usize,
}

pub fn create_token(user_id: i64, username: &str, secret: &str) -> Result<String, StatusCode> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[allow(dead_code)]
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 从 state 获取 secret
    let state = request.extensions().get::<crate::AppState>().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(token_data.claims);
    Ok(next.run(request).await)
}
