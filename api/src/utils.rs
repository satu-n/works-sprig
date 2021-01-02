use argon2;

use crate::errors;

lazy_static::lazy_static! {
pub static ref SECRET_KEY: String = std::env::var("SECRET_KEY")
    .unwrap_or_else(|_| "0123".repeat(8)); // TODO secret key
}

pub fn env_var(x: &str) -> String {
    std::env::var(x).expect(format!("{} must be set", x).as_str())
}

pub fn hash(password: &str) -> Result<String, errors::ServiceError> {
    argon2::hash_encoded(
        password.as_bytes(),
        SECRET_KEY.as_bytes(),
        &argon2::Config::default(),
    ).map_err(|err| {
        dbg!(err);
        errors::ServiceError::InternalServerError
    })
}

pub fn verify(hash: &str, password: &str) -> Result<bool, errors::ServiceError> {
    argon2::verify_encoded(
        hash,
        password.as_bytes(),
    ).map_err(|err| {
        dbg!(err);
        errors::ServiceError::InternalServerError
    })
}
