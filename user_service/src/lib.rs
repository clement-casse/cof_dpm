pub mod repo;
pub mod service;

use std::{collections::HashMap, fmt::Display};

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("the provided User ID cannot be parsed")]
    UserIdParseError,

    #[error("the identity provided already exists")]
    UserWithIdentityAlreadyExists,

    #[error("cannot authenticate user")]
    AuthenticationFailure,

    #[error(transparent)]
    Underlying(#[from] anyhow::Error),
}

/// An identity represents the external ID used by the authentifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserId(Uuid);

#[allow(clippy::new_without_default)]
impl UserId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parses the provided value as a `UserId`, it must be a UUID v4.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provided value cannot be parsed.
    pub fn parse(value: &str) -> Result<Self, Error> {
        Uuid::parse_str(value)
            .map_err(|_| Error::UserIdParseError)
            .map(UserId)
    }
}

impl From<Uuid> for UserId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl AsRef<Uuid> for UserId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Materializes a user in the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub identities: HashMap<String, Identity>,
}
