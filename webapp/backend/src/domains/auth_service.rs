use std::path::{Path, PathBuf};
use std::process::Command;

use actix_web::web::Bytes;
use log::error;

use std::fs::File;
use std::io::Read;
use std::time::Instant;

use crate::errors::AppError;
use crate::models::user::{Dispatcher, Session, User};
use crate::utils::{generate_session_token, hash_password, verify_password};

extern crate image;

use image::{GenericImageView, ImageBuffer, DynamicImage, ImageOutputFormat, Rgb, RgbImage};
use std::io::Cursor;

use super::dto::auth::LoginResponseDto;

pub trait AuthRepository {
    async fn create_user(&self, username: &str, password: &str, role: &str)
        -> Result<(), AppError>;
    async fn find_user_by_id(&self, id: i32) -> Result<Option<User>, AppError>;
    async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn create_dispatcher(&self, user_id: i32, area_id: i32) -> Result<(), AppError>;
    async fn find_dispatcher_by_id(&self, id: i32) -> Result<Option<Dispatcher>, AppError>;
    async fn find_dispatcher_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<Dispatcher>, AppError>;
    async fn find_profile_image_name_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Option<String>, AppError>;
    async fn create_session(&self, user_id: i32, session_token: &str) -> Result<(), AppError>;
    async fn delete_session(&self, session_token: &str) -> Result<(), AppError>;
    async fn find_session_by_session_token(&self, session_token: &str)
        -> Result<Session, AppError>;
}

#[derive(Debug)]
pub struct AuthService<T: AuthRepository + std::fmt::Debug> {
    repository: T,
}

impl<T: AuthRepository + std::fmt::Debug> AuthService<T> {
    pub fn new(repository: T) -> Self {
        AuthService { repository }
    }

    pub async fn register_user(
        &self,
        username: &str,
        password: &str,
        role: &str,
        area: Option<i32>,
    ) -> Result<LoginResponseDto, AppError> {
        if role == "dispatcher" && area.is_none() {
            return Err(AppError::BadRequest);
        }

        if (self.repository.find_user_by_username(username).await?).is_some() {
            return Err(AppError::Conflict);
        }

        let hashed_password = hash_password(password).unwrap();

        self.repository
            .create_user(username, &hashed_password, role)
            .await?;

        let session_token = generate_session_token();

        match self.repository.find_user_by_username(username).await? {
            Some(user) => {
                self.repository
                    .create_session(user.id, &session_token)
                    .await?;
                match user.role.as_str() {
                    "dispatcher" => {
                        self.repository
                            .create_dispatcher(user.id, area.unwrap())
                            .await?;
                        let dispatcher = self
                            .repository
                            .find_dispatcher_by_user_id(user.id)
                            .await?
                            .unwrap();
                        Ok(LoginResponseDto {
                            user_id: user.id,
                            username: user.username,
                            session_token,
                            role: user.role,
                            dispatcher_id: Some(dispatcher.id),
                            area_id: Some(dispatcher.area_id),
                        })
                    }
                    _ => Ok(LoginResponseDto {
                        user_id: user.id,
                        username: user.username,
                        session_token,
                        role: user.role,
                        dispatcher_id: None,
                        area_id: None,
                    }),
                }
            }
            None => Err(AppError::InternalServerError),
        }
    }

    pub async fn login_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginResponseDto, AppError> {
        let login_start = Instant::now();
        match self.repository.find_user_by_username(username).await? {
            Some(user) => {
                let login_duration0 = login_start.elapsed();
                //println!("login_user0 时间间隔: {:?}", login_duration0);
                let is_password_valid = verify_password(&user.password, password).unwrap();
                if !is_password_valid {
                    return Err(AppError::Unauthorized);
                }
                let login_duration1 = login_start.elapsed();
                //println!("login_user1 时间间隔: {:?}", login_duration1);

                let session_token = generate_session_token();
                self.repository
                    .create_session(user.id, &session_token)
                    .await?;



                match user.role.as_str() {
                    "dispatcher" => {
                        match self.repository.find_dispatcher_by_user_id(user.id).await? {
                            Some(dispatcher) => Ok(LoginResponseDto {
                                user_id: user.id,
                                username: user.username,
                                session_token,
                                role: user.role.clone(),
                                dispatcher_id: Some(dispatcher.id),
                                area_id: Some(dispatcher.area_id),
                            }),
                            None => Err(AppError::InternalServerError),
                        }
                    }
                    _ => Ok(LoginResponseDto {
                        user_id: user.id,
                        username: user.username,
                        session_token,
                        role: user.role.clone(),
                        dispatcher_id: None,
                        area_id: None,
                    }),
                }
            }
            None => Err(AppError::Unauthorized),
        }
    }

    pub async fn logout_user(&self, session_token: &str) -> Result<(), AppError> {
        self.repository.delete_session(session_token).await?;
        Ok(())
    }

    pub async fn get_resized_profile_image_byte(
        &self,
        user_id: i32,
        width: i32,
        height: i32,
    ) -> Result<Bytes, AppError> {

        let resized_start = Instant::now();


        let profile_image_name = match self
            .repository
            .find_profile_image_name_by_user_id(user_id)
            .await
        {
            Ok(Some(name)) => name,
            Ok(None) => return Err(AppError::NotFound),
            Err(_) => return Err(AppError::NotFound),
        };

        /*
        let resized_duration0 = resized_start.elapsed();
        println!("resized_duration0 时间间隔: {:?}", resized_duration0);



        let width: u32 = width as u32;
        let height: u32 = height as u32;


        // 创建一个新的ImageBuffer，使用RGB类型
        let mut img: RgbImage = ImageBuffer::new(width, height);

        // 填充图片为黑色
        for pixel in img.pixels_mut() {
            *pixel = Rgb([0, 0, 0]);
        }

        // 将ImageBuffer转换为DynamicImage
        let dynamic_img = DynamicImage::ImageRgb8(img);

        // 将图片保存到内存中的字节缓冲区
        let mut buffer = Cursor::new(Vec::new());
        dynamic_img.write_to(&mut buffer, image::ImageOutputFormat::Png).unwrap();

        // 获取字节对象并转换为Bytes实例
        Ok(Bytes::from(buffer.into_inner()))
        */


        


        let path: PathBuf =
            Path::new(&format!("images/user_profile/{}", profile_image_name)).to_path_buf();


        let output = Command::new("convert")
            .arg(&path)
            .arg("-resize")
            .arg(format!("{}x{}!", width, height))
            .arg("png:-")
            .output()
            .map_err(|e| {
                error!("画像リサイズのコマンド実行に失敗しました: {:?}", e);
                AppError::InternalServerError
            })?;

        let resized_duration1 = resized_start.elapsed();
        println!("resized_duration1 时间间隔: {:?}", resized_duration1);

        match output.status.success() {
            true => Ok(Bytes::from(output.stdout)),
            false => {
                error!(
                    "画像リサイズのコマンド実行に失敗しました: {:?}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Err(AppError::InternalServerError)
            }
        }
        


        /*
        if Path::new(&format!("images/user_profile/resized_{}x{}_{}", width, height, profile_image_name)).exists()
        {
            println!("File exists.");
            // 打开图像文件
            let mut greeting_file_result = File::open(format!("images/user_profile/resized_{}x{}_{}", width, height, profile_image_name));
            //let mut greeting_file_result = File::open(format!("images/user_profile/resized_1234.png",));
            
            let mut greeting_file = match greeting_file_result {
                Ok(file) => file,
                Err(error) => panic!("Problem opening the file: {error:?}"),
            };

            // 创建一个缓冲区来存储文件内容
            let mut buffer = Vec::new();

            // 读取文件内容到缓冲区
            greeting_file.read_to_end(&mut buffer);

            // 从缓冲区创建 Bytes 实例
            let bytes = Bytes::from(buffer);

            Ok(bytes)
        } 
        else {
            let path: PathBuf =
                Path::new(&format!("images/user_profile/{}", profile_image_name)).to_path_buf();
            let output = Command::new("convert")
                .arg(&path)
                .arg("-resize")
                .arg(format!("{}x{}!", width, height))
                .arg("png:-")
                .arg(format!("images/user_profile/resized_{}x{}_{}", width, height, profile_image_name))
                .output()
                .map_err(|e| {
                    println!("画像リサイズのコマンド実行に失敗しました: {:?}", e);
                    error!("画像リサイズのコマンド実行に失敗しました: {:?}", e);
                    AppError::InternalServerError
                });
            // 打开图像文件
            let mut greeting_file_result = File::open(format!("images/user_profile/resized_{}x{}_{}", width, height, profile_image_name));
            //let mut greeting_file_result = File::open(format!("images/user_profile/resized_1234.png",));
            
            let mut greeting_file = match greeting_file_result {
                Ok(file) => file,
                Err(error) => panic!("Problem opening the file: {error:?}"),
            };

            // 创建一个缓冲区来存储文件内容
            let mut buffer = Vec::new();

            // 读取文件内容到缓冲区
            greeting_file.read_to_end(&mut buffer);

            // 从缓冲区创建 Bytes 实例
            let bytes = Bytes::from(buffer);

            Ok(bytes)
        }
        // 指定图像文件的路径
        //let file_path = "path/to/your/image.png";
        */

        
        
        

        
        
        

    }

    pub async fn validate_session(&self, session_token: &str) -> Result<bool, AppError> {
        let session = self
            .repository
            .find_session_by_session_token(session_token)
            .await?;

        Ok(session.is_valid)
    }
}
