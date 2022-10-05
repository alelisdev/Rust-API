use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[repr(i32)]
pub enum RealUserStatus {
    Unsupported = 0,
    Unknown = 1,
    LikelyReal = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub id_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub id_token: String,
    pub token_type: String,
}

/// https://developer.apple.com/documentation/sign_in_with_apple/sign_in_with_apple_rest_api/authenticating_users_with_sign_in_with_apple
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdToken {
    pub iss: String,
    pub sub: i64,
    pub aud: String,
    pub iat: String,
    pub exp: String,
    pub nonce: String,
    pub nonce_supported: bool,
    pub email: String,
    pub email_verified: bool,
    pub is_private_email: bool,
    pub real_user_status: RealUserStatus,
    pub transfer_sub: String,
}

impl Client {
    pub async fn authenticate(&self, code: String) -> Result<IdToken, PlatformSigninError> {
        let grant_type = "authorization_code";
        let body = format!(
            "client_id:{}\nclient_secret:{}\ncode:{}\ngrant_type:{}",
            self.client_id, self.client_secret, code, grant_type
        );
            dbg!(&body);
        let resp = self
            .client
            .post("https://appleid.apple.com/auth/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|_| PlatformSigninError::ReqwestError)?;
        let token: AuthTokenResponse = if resp.status() == 200 {
            let text = resp
                .text()
                .await
                .map_err(|_| PlatformSigninError::ReqwestError)?;
            dbg!(&text);
            serde_json::from_str(&text).map_err(|_| PlatformSigninError::Serialize)?
        } else {
            return Err(PlatformSigninError::BadRequest);
        };
        let validation = Validation {
            algorithms: vec![Algorithm::ES256],
            ..Default::default()
        };
        let token_data = decode::<IdToken>(
            &token.id_token,
            &DecodingKey::from_base64_secret(&code)
                .map_err(|_| PlatformSigninError::SecretError)?,
            &validation,
        )
        .map_err(|_| PlatformSigninError::ValidationError)?;
        let id_token = token_data.claims;
        Ok(id_token)
    }

    pub fn validate(&self, refresh_token: String) -> Result<IdToken, PlatformSigninError> {
        let body = format!(
            "client_id:{}\nclient_secret:{}\nrefresh_token:{}\ngrant_type:{}",
            self.client_id, self.client_secret, refresh_token, grant_type
        );
        let grant_type = "refresh_token";
        let resp = self
            .client
            .post("https://appleid.apple.com/auth/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|_| PlatformSigninError::ReqwestError)?;
        let token: ValidTokenResponse = if resp.status() == 200 {
            let text = resp
                .text()
                .await
                .map_err(|_| PlatformSigninError::ReqwestError)?;
            dbg!(&text);
            serde_json::from_str(&text).map_err(|_| PlatformSigninError::Serialize)?
        } else {
            return Err(PlatformSigninError::BadRequest);
        };
        let validation = Validation {
            algorithms: vec![Algorithm::ES256],
            ..Default::default()
        };
        let token_data = decode::<IdToken>(
            &token.id_token,
            &DecodingKey::from_base64_secret(&code)
                .map_err(|_| PlatformSigninError::SecretError)?,
            &validation,
        )
        .map_err(|_| PlatformSigninError::ValidationError)?;
        let id_token = token_data.claims;
        Ok(id_token)
    }
}

#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Clone)]
pub enum PlatformSigninError {
    ReqwestError,
    JwtError,
    BadRequest,
    Serialize,
    ValidationError,
    SecretError,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Claims {
    iss: String,
    iat: i64,
    exp: i64,
    aud: String,
    sub: String,
}

pub const APPLE_SERVER: &str = "https://appleid.apple.com";
pub const EXPIRATION_TIME_SECS: i64 = 1200;

impl Client {
    pub fn new(
        client_id: String,
        apple_private_key: String,
        apple_kid: String,
        team_id: String,
    ) -> Result<Self, PlatformSigninError> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|_| PlatformSigninError::ReqwestError)?;
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(apple_kid);
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let claims = Claims {
            iss: team_id,
            iat: now,
            exp: now + EXPIRATION_TIME_SECS,
            aud: APPLE_SERVER.to_string(),
            sub: client_id.clone(),
        };

        let client_secret = encode(
            &header,
            &claims,
            &EncodingKey::from_ec_pem(apple_private_key.as_ref()).map_err(|_| PlatformSigninError::JwtError)?,
        )
        .map_err(|_| PlatformSigninError::JwtError)?;
        Ok(Self {
            client,
            client_id,
            client_secret,
        })
    }
}
