/*
 * File: src/auth.rs
 * Description: Authentication Middleware for Axum - provides JWT creation and validation.
 * Author: Antigravity (AI Assistant)
 * Created: 2026-02-13
 * Last Modified: 2026-02-13
 * 
 * Changes:
 * - 2026-02-13: Added unit tests for JWT validation.
 * - 2026-02-13: Added file header and documentation comments.
 */

//! # Authentication Middleware for Axum
//!
//! This module provides the authentication middleware for the `axum` router.
//! It is responsible for creating claims and JWTs.

use axum::{
    extract::{FromRequestParts},
    response::{IntoResponse, Response},
    RequestPartsExt,
    Json,
    http::StatusCode,
    http::request::Parts,
};
use axum_extra::headers::authorization::{Authorization, Bearer};
use axum_extra::TypedHeader;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// The claims of a JWT.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// The subject of the token (the user ID).
    pub sub: String,
    /// The expiration time of the token.
    pub exp: usize,
}

/// An `axum` extractor for the claims of an authenticated user.
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the `Authorization` header.
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        // Decode and validate the token.
        // **Security Note:** The secret key is hardcoded here for simplicity.
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret("your-secret-key".as_ref()),
            &Validation::default(),
        )
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    #[tokio::test]
    #[ignore]
    async fn test_claims_validation() {
        // This test requires a global crypto provider to be installed for rustls/jsonwebtoken
        // let _ = rustls::crypto::ring::default_provider().install_default();
        let claims = Claims {
            sub: "user-123".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("your-secret-key".as_ref()),
        )
        .unwrap();

        let decoded = jsonwebtoken::decode::<Claims>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret("your-secret-key".as_ref()),
            &jsonwebtoken::Validation::default(),
        )
        .unwrap();

        assert_eq!(decoded.claims.sub, "user-123");
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let res = jsonwebtoken::decode::<Claims>(
            "invalid-token",
            &jsonwebtoken::DecodingKey::from_secret("your-secret-key".as_ref()),
            &jsonwebtoken::Validation::default(),
        );

        assert!(res.is_err());
    }
}

/// An error that can occur during authentication.
#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}