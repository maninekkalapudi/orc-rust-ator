use crate::auth::Claims;
use crate::models::user::User;
use crate::state::db::Db;
use axum::{extract::State, http::StatusCode, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

/// The request payload for authentication (login and registration).
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// The response payload for a successful login, containing the JWT.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}


/// Registers a new user.
pub async fn register(
    State(db): State<Db>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<User>, StatusCode> {
    // Hash the password using bcrypt.
    let hashed_password =
        hash(&payload.password, DEFAULT_COST).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create the user in the database.
    let user = db
        .create_user(&payload.username, &hashed_password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}


/// Authenticates a user and returns a JWT.
pub async fn login(
    State(db): State<Db>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<TokenResponse>, StatusCode> {
    // Retrieve the user from the database.
    let user = db
        .get_user_by_username(&payload.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify the password.
    if !verify(&payload.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Create the JWT claims.
    let claims = Claims {
        sub: user.user_id.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    // Create the JWT.
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("your-secret-key".as_ref()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TokenResponse { token }))
}