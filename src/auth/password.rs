use crate::error::{AppError, Result};

pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::InternalError);
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash)
        .map_err(|_| AppError::Authentication("Invalid password".to_string()));
}
