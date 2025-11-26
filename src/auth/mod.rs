pub mod jwt;
pub mod oauth;
pub mod password;
pub mod auth_models;
pub mod auth_dto;
pub mod auth_repository;
pub mod auth_handlers;
pub mod auth_service;

pub use jwt::{create_jwt, create_access_token, create_refresh_token, verify_jwt, Claims};
pub use oauth::create_oauth_client;
pub use password::{hash_password, verify_password};
pub use auth_models::RefreshToken;
pub use auth_dto::{AuthResponse, LoginRequest, RegisterRequest, RefreshTokenRequest, RefreshTokenResponse};
pub use auth_repository::RefreshTokenRepository;
pub use auth_handlers::{register, login, google_login, google_callback, refresh_token, logout};
pub use auth_service::AuthService;
