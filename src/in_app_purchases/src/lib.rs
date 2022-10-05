mod error;
pub use error::Error;
mod get_apple_subscription_status;
mod get_google_product;
mod get_google_subscription;
mod get_purchase;
mod purchase;
mod util;
mod verify_apple_receipt;
pub use purchase::Purchase;
mod platform;
pub use platform::Platform;
mod apple_receipt;
pub use apple_receipt::AppleInAppReceipt;
//AppleReceipt
// mod apple_receipt_type;
// pub use apple_receipt_type::AppleReceiptType;
mod apple_receipt_status;
pub use apple_receipt_status::AppleReceiptStatus;
mod apple_subscription_status;
pub use apple_subscription_status::AppleSubscriptionStatus;
mod google_subscription_purchase;
pub use google_subscription_purchase::GoogleSubscriptionPurchase;
mod google_product_purchase;
pub use google_product_purchase::GoogleProductPurchase;
mod environment;
pub use environment::Environment;
mod apple_api_error_code;
pub use apple_api_error_code::AppleApiErrorCode;
mod claims;
use claims::Claims;
mod product_type;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
pub use product_type::ProductType;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const BASE_ANDROID_URL: &str = "https://androidpublisher.googleapis.com/androidpublisher/v3";
const BASE_STOREKIT_URL_PLAY: &str = "https://api.storekit-sandbox.itunes.apple.com/inApps/v1";
const BASE_STOREKIT_URL_LIVE: &str = "https://api.storekit.itunes.apple.com/inApps/v1";
const BASE_ITUNES_URL_PLAY: &str = "https://sandbox.itunes.apple.com";
const BASE_ITUNES_URL_LIVE: &str = "https://buy.itunes.apple.com";

pub struct Gateway {
    client: reqwest::Client,
    apple_bundle_id: String,
    apple_key_id: String,
    apple_key: String,
    apple_password: String, // Shared app secret.
    apple_issuer: String,
    google_service_account_email: String,
    google_key: String,
}

impl Gateway {
    pub async fn new(
        apple_bundle_id: String,
        apple_key_id: String,
        apple_key: String,
        apple_password: String,
        apple_issuer: String,
        google_service_account_email: String,
        google_key: String,
        timeout: Option<std::time::Duration>,
    ) -> Result<Gateway, Error> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let timeout = match timeout {
            Some(t) => t,
            None => std::time::Duration::new(60, 0),
        };

        let client = match reqwest::ClientBuilder::new()
            .default_headers(headers)
            .https_only(true)
            .timeout(timeout)
            .build()
        {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::Unspecified(format!(
                    "Could not create reqwest client ({}).",
                    err.to_string()
                )))
            }
        };

        let c = Gateway {
            client,
            apple_bundle_id,
            apple_key_id,
            apple_key,
            apple_password,
            apple_issuer,
            google_service_account_email,
            google_key,
        };
        Ok(c)
    }

    // cf. https://developer.apple.com/documentation/appstoreserverapi/generating_tokens_for_api_requests
    fn get_apple_token(&self) -> Result<String, Error> {
        let iat = Utc::now();
        let exp = iat + chrono::Duration::minutes(20);
        let aud = "appstoreconnect-v1";
        let nonce = uuid::Uuid::new_v4().to_string();
        let claims = Claims::new(
            &self.apple_issuer,
            iat,
            None,
            aud,
            &nonce,
            &self.apple_bundle_id,
            exp,
        );

        let encoding_key = match EncodingKey::from_ec_pem(self.apple_key.as_bytes()) {
            Ok(key) => key,
            Err(err) => {
                return Err(Error::ParseError(format!(
                    "Could not create JWT encoding key ({}).",
                    err.to_string()
                )));
            }
        };

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.apple_key_id.clone());

        let token = match encode(&header, &claims, &encoding_key) {
            Ok(token) => token,
            Err(err) => {
                return Err(Error::ParseError(format!(
                    "Could not encode JWT token ({}).",
                    err.to_string()
                )));
            }
        };

        Ok(token)
    }

    // cf. https://developers.google.com/identity/protocols/oauth2/service-account#httprest
    async fn get_google_token(&self) -> Result<String, Error> {
        let iat = Utc::now();
        let exp = iat + chrono::Duration::minutes(20);
        let aud = "https://oauth2.googleapis.com/token";
        let nonce = uuid::Uuid::new_v4().to_string();
        let claims = Claims::new(
            &self.google_service_account_email,
            iat,
            Some(String::from(
                "https://www.googleapis.com/auth/androidpublisher",
            )),
            aud,
            &nonce,
            &self.apple_bundle_id,
            exp,
        );

        let encoding_key = match EncodingKey::from_rsa_pem(self.google_key.as_bytes()) {
            Ok(key) => key,
            Err(err) => {
                return Err(Error::ParseError(format!(
                    "Could not create JWT encoding key ({}).",
                    err.to_string()
                )));
            }
        };

        let header = Header::new(Algorithm::RS256);

        let token = match encode(&header, &claims, &encoding_key) {
            Ok(token) => token,
            Err(err) => {
                return Err(Error::ParseError(format!(
                    "Could not encode JWT token ({}).",
                    err.to_string()
                )));
            }
        };

        let url = String::from("https://oauth2.googleapis.com/token");
        let client = reqwest::Client::new();
        let res = match client
            .post(url)
            .body(format!(
                "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={}",
                token
            ))
            .header(
                "Content-Type",
                String::from("application/x-www-form-urlencoded"),
            )
            .send()
            .await
        {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::NetworkError(format!(
                    "Could not send message ({}).",
                    err.to_string()
                )))
            }
        };

        let status = res.status();
        let text = res
            .text()
            .await
            .unwrap_or_else(|_| String::from("Could not retrieve body text."));

        if status != 200 {
            #[derive(Deserialize, Debug, Clone, PartialEq)]
            struct GoogleApiErrorDetails {
                pub code: i32,
                pub message: String,
            }

            #[derive(Deserialize, Debug, Clone, PartialEq)]
            struct GoogleApiError {
                pub error: GoogleApiErrorDetails,
            }

            let api_error: GoogleApiError =
                serde_json::from_str(&text).unwrap_or_else(|_| GoogleApiError {
                    error: GoogleApiErrorDetails {
                        code: 0,
                        message: format!("Unknown error ({}: {})", status, text),
                    },
                });
            return Err(Error::GoogleApiError(
                api_error.error.code,
                api_error.error.message,
            ));
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct GetTokenResponse {
            pub access_token: String,
            // pub scope: String,
            // pub token_type: String,
            // pub expires_in: String,
        }

        let res: GetTokenResponse = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::SerializationError(format!(
                    "Could not deserialize response ({}) from {}.",
                    err.to_string(),
                    &text,
                )))
            }
        };

        Ok(res.access_token)
    }

    async fn post<'a, T: DeserializeOwned>(
        &self,
        url: &str,
        body: impl Serialize,
        platform: Platform,
    ) -> Result<T, Error> {
        let token = match platform {
            Platform::Apple => self.get_apple_token()?,
            Platform::Google => self.get_google_token().await?,
        };
        let g = format!("Bearer {}", token);
        let bearer = match HeaderValue::from_str(&g) {
            Ok(bearer) => bearer,
            Err(err) => {
                return Err(Error::Unspecified(format!(
                    "Could not parse auth token header value ({}).",
                    err.to_string()
                )));
            }
        };

        let res = match self
            .client
            .post(url)
            .json(&body)
            .header("Authorization", bearer)
            .send()
            .await
        {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::NetworkError(format!(
                    "Could not send request ({}).",
                    err.to_string()
                )))
            }
        };

        let status = res.status();
        let text = res
            .text()
            .await
            .unwrap_or_else(|_| String::from("Could not retrieve body text."));

        if status != 200 {
            #[derive(Deserialize, Debug, Clone, PartialEq)]
            #[serde(rename_all = "camelCase")]
            struct AppleApiError {
                pub error_code: AppleApiErrorCode,
                pub error_message: String,
            }

            let api_error: AppleApiError =
                serde_json::from_str(&text).unwrap_or_else(|_| AppleApiError {
                    error_code: AppleApiErrorCode::Unknown,
                    error_message: format!("Unknown error ({}: {})", status, text),
                });
            return Err(Error::AppleApiError(
                api_error.error_code,
                api_error.error_message,
            ));
        }

        let body: T = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::SerializationError(format!(
                    "Could not deserialize response ({}) from {}.",
                    err.to_string(),
                    &text,
                )))
            }
        };
        Ok(body)
    }

    async fn get<'a, T: DeserializeOwned>(
        &self,
        url: &str,
        platform: Platform,
    ) -> Result<T, Error> {
        let token = match platform {
            Platform::Apple => self.get_apple_token()?,
            Platform::Google => self.get_google_token().await?,
        };
        let g = format!("Bearer {}", token);

        let bearer = match HeaderValue::from_str(&g) {
            Ok(bearer) => bearer,
            Err(err) => {
                return Err(Error::Unspecified(format!(
                    "Could not parse auth token header value ({}).",
                    err.to_string()
                )));
            }
        };

        let res = match self
            .client
            .get(url)
            .header("Authorization", bearer)
            .send()
            .await
        {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::NetworkError(format!(
                    "Could not send request ({}).",
                    err.to_string()
                )))
            }
        };

        let status = res.status();
        let text = res
            .text()
            .await
            .unwrap_or_else(|_| String::from("Could not retrieve body text."));

        if status != 200 {
            #[derive(Deserialize, Debug, Clone, PartialEq)]
            #[serde(rename_all = "camelCase")]
            struct AppleApiError {
                pub error_code: AppleApiErrorCode,
                pub error_message: String,
            }

            let api_error: AppleApiError =
                serde_json::from_str(&text).unwrap_or_else(|_| AppleApiError {
                    error_code: AppleApiErrorCode::Unknown,
                    error_message: format!("Unknown error ({}: {})", status, text),
                });
            return Err(Error::AppleApiError(
                api_error.error_code,
                api_error.error_message,
            ));
        }

        let body: T = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::SerializationError(format!(
                    "Could not deserialize response ({}) from {}.",
                    err.to_string(),
                    &text,
                )))
            }
        };
        Ok(body)
    }
}
