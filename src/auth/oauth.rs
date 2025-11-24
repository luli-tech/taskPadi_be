use crate::error::{AppError, Result};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};

pub fn create_oauth_client(
    client_id: String,
    client_secret: String,
    redirect_uri: String,
) -> Result<BasicClient> {
    let google_client_id = ClientId::new(client_id);
    let google_client_secret = ClientSecret::new(client_secret);
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .map_err(|_| AppError::InternalError)?;
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .map_err(|_| AppError::InternalError)?;
    let redirect_url = RedirectUrl::new(redirect_uri)
        .map_err(|_| AppError::InternalError)?;

    Ok(BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(redirect_url))
}

#[derive(serde::Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}
