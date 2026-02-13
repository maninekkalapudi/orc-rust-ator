// In src/models/user.rs

//! # User Data Model
//!
//! This module defines the `User` struct, which represents a user in the
//! database.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a user in the database.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    /// The unique identifier for the user.
    pub user_id: Uuid,
    /// The username of the user.
    pub username: String,
    /// The hashed password of the user.
    ///
    /// This field is skipped during serialization to avoid exposing the
    /// password hash in API responses.
    #[serde(skip)]
    pub password_hash: String,
    /// The timestamp when the user was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The timestamp when the user was last updated.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}