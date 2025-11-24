use crate::models::user::CurUser;

pub mod chat_type;
pub mod event;
pub mod middlewares;
pub mod models;
pub mod utils;

pub trait TokenVerify {
    fn verify_token(&self, token: &str) -> Result<CurUser, jwt_simple::Error>;
}
