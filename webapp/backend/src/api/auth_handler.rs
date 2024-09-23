use crate::domains::auth_service::AuthService;
use crate::domains::dto::auth::{LoginRequestDto, LogoutRequestDto, RegisterRequestDto};
use crate::errors::AppError;
use crate::repositories::auth_repository::AuthRepositoryImpl;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;
use std::path::Path;

use std::time::Instant;

#[derive(Deserialize, Debug)]
pub struct ValidateSessionQueryParams {
    session_token: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ValidationResponse {
    is_valid: bool,
}

pub async fn validate_session_handler(
    service: web::Data<AuthService<AuthRepositoryImpl>>,
    query: web::Query<ValidateSessionQueryParams>,
) -> Result<HttpResponse, AppError> {
    match &query.session_token {
        Some(session_token) => match service.validate_session(session_token.as_str()).await {
            Ok(is_valid) => Ok(HttpResponse::Ok().json(ValidationResponse { is_valid })),
            Err(_) => Ok(HttpResponse::Ok().json(ValidationResponse { is_valid: false })),
        },
        None => Ok(HttpResponse::Ok().json(ValidationResponse { is_valid: false })),
    }
}

pub async fn register_handler(
    service: web::Data<AuthService<AuthRepositoryImpl>>,
    req: web::Json<RegisterRequestDto>,
) -> Result<HttpResponse, AppError> {
    match service
        .register_user(&req.username, &req.password, &req.role, req.area_id)
        .await
    {
        Ok(response) => Ok(HttpResponse::Created().json(response)),
        Err(err) => Err(err),
    }
}

pub async fn login_handler(
    service: web::Data<AuthService<AuthRepositoryImpl>>,
    req: web::Json<LoginRequestDto>,
) -> Result<HttpResponse, AppError> {
    //println!("hello,world!");
    let start = Instant::now();

    match service.login_user(&req.username, &req.password).await {
        Ok(response) => {
            // 打印计算结果和运行时间
            let duration = start.elapsed();
            println!("login_handler 时间间隔: {:?}", duration);
            Ok(HttpResponse::Ok().json(response))
        },
        Err(err) => Err(err),
    }


}

pub async fn logout_handler(
    service: web::Data<AuthService<AuthRepositoryImpl>>,
    req: web::Json<LogoutRequestDto>,
) -> Result<HttpResponse, AppError> {
    match service.logout_user(&req.session_token).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(_) => Ok(HttpResponse::Ok().finish()),
    }
}

#[derive(Deserialize, Debug)]
pub struct UserProfileImageQueryParams {
    w: Option<i32>,
    h: Option<i32>,
}



pub async fn user_profile_image_handler(
    service: web::Data<AuthService<AuthRepositoryImpl>>,
    path: web::Path<i32>,
    query: web::Query<UserProfileImageQueryParams>,
) -> Result<HttpResponse, AppError> {

    let image_start = Instant::now();
    let user_id = path.into_inner();
    let width = query.w.unwrap_or(500);
    let height = query.h.unwrap_or(500);

    
    let profile_image_byte = service
        .get_resized_profile_image_byte(user_id, width, height)
        .await?;

    // 打印计算结果和运行时间
    //let image_duration = image_start.elapsed();
    //println!("user_profile_image_handler 时间间隔: {:?}", image_duration);

    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .append_header(("Cache-Control", "max-age=3600"))
        .body(profile_image_byte))
    



    

}
