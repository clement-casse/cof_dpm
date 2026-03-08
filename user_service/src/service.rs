use async_trait::async_trait;

use crate::{Error, Identity, User, repo};

#[async_trait]
pub trait UserService {
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, Error>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateUserRequest {
    identity_provider: String,
    identity: Identity,
    name: String,
}

pub struct Service<R>
where
    R: repo::Repository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: repo::Repository,
{
    pub fn new(repo: R) -> Self {
        Self {
            repo,
        }
    }
}

#[async_trait]
impl<R> UserService for Service<R>
where
    R: repo::Repository,
{
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, Error> {
        todo!()
    }
}
