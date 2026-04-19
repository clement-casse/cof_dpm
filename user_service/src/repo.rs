use async_trait::async_trait;

use crate::{Error, Identity, User};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Repository: Send + Sync + 'static {
    async fn get_user_from_identity(&self, identity: Identity) -> Result<User, Error>;
}
