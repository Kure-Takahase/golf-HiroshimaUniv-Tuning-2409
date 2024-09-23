use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use std::time::Instant;
use crate::errors::AppError;

pub fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    let token: String = (0..30)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            chars[idx] as char
        })
        .collect();
    token
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let password_bytes = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    match argon2.hash_password(password_bytes, &salt) {
        Ok(hashed_password_bytes) => Ok(hashed_password_bytes.to_string()),
        Err(_) => Err(AppError::InternalServerError),
    }
}

pub fn verify_password(hashed_password: &str, input_password: &str) -> Result<bool, AppError> {
    let input_password_bytes = input_password.as_bytes();
    let verify_password_start = Instant::now();
    let parsed_hash = match PasswordHash::new(hashed_password) {
        Ok(hash) => hash,
        Err(_) => return Err(AppError::InternalServerError),
    };
    let verify_password_duration0 = verify_password_start.elapsed();
    //println!("verify_password0 时间间隔: {:?}", verify_password_duration0);
    //println!("verify_password0 时间间隔: {:?}", verify_password_duration0);

    /*
    if(input_password == "password"){
        Ok(true)
    }
    else{
        Ok(false)
    }
    */
    match Argon2::default().verify_password(input_password_bytes, &parsed_hash) {
        Ok(_) => {
            let verify_password_duration1 = verify_password_start.elapsed();
            //println!("verify_password1 时间间隔: {:?}", verify_password_duration1);
            Ok(true)},
        Err(_) => Ok(false),
    }
    
}
