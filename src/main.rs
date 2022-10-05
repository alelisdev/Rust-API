// NOTE: We set an unusually high recursion limit in order to allow warp to have a lot of endpoints
#![recursion_limit = "256"]
#![type_length_limit = "2000000"]
use appinsights::{InMemoryChannel, TelemetryClient, TelemetryConfig};
use cosmos_utils::{set_state, CosmosState};
use lazy_static::lazy_static;
use std::time::Duration;
use warp::{http::Method, Filter};
mod api;
mod models;
use models::*;
mod fault;
mod filters;
mod push;
mod util;
#[macro_use]
extern crate bitflags;

#[cfg(debug_assertions)]
lazy_static! {
    static ref PRODUCTION_ENVIRONMENT: bool =
        std::env::var("PRODUCTION_ENVIRONMENT").unwrap() == "true";
    static ref ACCESS_TOKEN_SECRET: String = std::env::var("ACCESS_TOKEN_SECRET").unwrap();
    static ref REFRESH_TOKEN_SECRET: String = std::env::var("REFRESH_TOKEN_SECRET").unwrap();
    static ref COSMOS_MASTER_KEY: String = std::env::var("COSMOS_MASTER_KEY").unwrap();
    static ref COSMOS_ACCOUNT: String = std::env::var("COSMOS_ACCOUNT").unwrap();
    static ref STORAGE_ACCOUNT: String = std::env::var("STORAGE_ACCOUNT").unwrap();
    static ref STORAGE_MASTER_KEY: String = std::env::var("STORAGE_MASTER_KEY").unwrap();
    static ref SENDGRID_API_KEY: String = std::env::var("SENDGRID_API_KEY").unwrap();
    static ref NOTIFICATION_HUB_ACCOUNT: push::Account = push::Account {
        key_name: "notification-hub-name".to_string(),
        key: "notification-hub-key".to_string(),
    };
    static ref CERTIFICATE_STORAGE_CONTAINER: String =
        std::env::var("CERTIFICATE_STORAGE_CONTAINER").unwrap();
    static ref APPLICATION_INSIGHTS_INSTRUMENTATION_KEY: String =
        std::env::var("APPLICATION_INSIGHTS_INSTRUMENTATION_KEY").unwrap();
    static ref APPLICATION_INSIGHTS_INGESTION_ENDPOINT: String =
        std::env::var("APPLICATION_INSIGHTS_INGESTION_ENDPOINT").unwrap();
    static ref APPLICATION_INSIGHTS_TELEMETRY_CLIENT: TelemetryClient<InMemoryChannel> = {
        let config = TelemetryConfig::builder()
            .i_key(APPLICATION_INSIGHTS_INSTRUMENTATION_KEY.to_string())
            .interval(Duration::from_secs(2))
            .endpoint(APPLICATION_INSIGHTS_INGESTION_ENDPOINT.to_string())
            .build();
        TelemetryClient::<InMemoryChannel>::from_config(config)
    };
}

#[cfg(not(debug_assertions))]
lazy_static! {
    static ref PRODUCTION_ENVIRONMENT: bool =
        std::env::var("PRODUCTION_ENVIRONMENT").unwrap() == "true";
    static ref ACCESS_TOKEN_SECRET: String = std::env::var("ACCESS_TOKEN_SECRET").unwrap();
    static ref REFRESH_TOKEN_SECRET: String = std::env::var("REFRESH_TOKEN_SECRET").unwrap();
    static ref COSMOS_MASTER_KEY: String = std::env::var("COSMOS_MASTER_KEY").unwrap();
    static ref COSMOS_ACCOUNT: String = std::env::var("COSMOS_ACCOUNT").unwrap();
    static ref STORAGE_ACCOUNT: String = std::env::var("STORAGE_ACCOUNT").unwrap();
    static ref STORAGE_MASTER_KEY: String = std::env::var("STORAGE_MASTER_KEY").unwrap();
    static ref SENDGRID_API_KEY: String = std::env::var("SENDGRID_API_KEY").unwrap();
    static ref NOTIFICATION_HUB_ACCOUNT: push::Account = push::Account {
        key_name: "DefaultFullSharedAccessSignature".to_string(),
        key: std::env::var("PUSH_NOTIFICATION_HUB_KEY").unwrap(),
    };
    static ref CERTIFICATE_STORAGE_CONTAINER: String =
        std::env::var("CERTIFICATE_STORAGE_CONTAINER").unwrap();
    static ref APPLICATION_INSIGHTS_INSTRUMENTATION_KEY: String =
        std::env::var("APPLICATION_INSIGHTS_INSTRUMENTATION_KEY").unwrap();
    static ref APPLICATION_INSIGHTS_INGESTION_ENDPOINT: String =
        std::env::var("APPLICATION_INSIGHTS_INGESTION_ENDPOINT").unwrap();
    static ref APPLICATION_INSIGHTS_TELEMETRY_CLIENT: TelemetryClient<InMemoryChannel> = {
        let config = TelemetryConfig::builder()
            .i_key(APPLICATION_INSIGHTS_INSTRUMENTATION_KEY.to_string())
            .interval(Duration::from_secs(2))
            .endpoint(APPLICATION_INSIGHTS_INGESTION_ENDPOINT.to_string())
            .build();
        TelemetryClient::<InMemoryChannel>::from_config(config)
    };
}

// NOTE(Jonathan): This is so that we can box (higher runtime cost lower
// compiletime) when compiling and not box when compiling the release.
#[cfg(debug_assertions)]
macro_rules! maybe_box {
    ($expression:expr) => {
        $expression.boxed()
    };
}

#[cfg(not(debug_assertions))]
macro_rules! maybe_box {
    ($expression:expr) => {
        $expression.boxed()
    };
}

lazy_static! {
    static ref COSMOS_DATABASE: String = std::env::var("COSMOS_DATABASE").unwrap();
    static ref DEFAULT_OFFICE_ID: String = std::env::var("DEFAULT_OFFICE_ID").unwrap();
    static ref IN_APP_PURCHASES_APPLE_BUNDLE_ID: String =
        std::env::var("IN_APP_PURCHASES_APPLE_BUNDLE_ID").unwrap();
    static ref IN_APP_PURCHASES_APPLE_KEY_ID: String =
        std::env::var("IN_APP_PURCHASES_APPLE_KEY_ID").unwrap();
    static ref IN_APP_PURCHASES_APPLE_KEY: String =
        std::env::var("IN_APP_PURCHASES_APPLE_KEY").unwrap();
    static ref IN_APP_PURCHASES_APPLE_PASSWORD: String =
        std::env::var("IN_APP_PURCHASES_APPLE_PASSWORD").unwrap();
    static ref IN_APP_PURCHASES_APPLE_ISSUER: String =
        std::env::var("IN_APP_PURCHASES_APPLE_ISSUER").unwrap();
    static ref IN_APP_PURCHASES_GOOGLE_SERVICE_ACCOUNT: String =
        std::env::var("IN_APP_PURCHASES_GOOGLE_SERVICE_ACCOUNT").unwrap();
    static ref IN_APP_PURCHASES_GOOGLE_KEY: String =
        std::env::var("IN_APP_PURCHASES_GOOGLE_KEY").unwrap();
    static ref APPLE_SIGNIN_KID: String = std::env::var("APPLE_SIGNIN_KID").unwrap();
    static ref APPLE_SIGNIN_TEAM_ID: String = std::env::var("APPLE_SIGNIN_TEAM_ID").unwrap();
    static ref APPLE_SIGNIN_CLIENT_ID: String = std::env::var("APPLE_SIGNIN_CLIENT_ID").unwrap();
    static ref APPLE_SIGNIN_PRIVATE_KEY: String =
        std::env::var("APPLE_SIGNIN_PRIVATE_KEY").unwrap();
    static ref CRON_SECRET: String = std::env::var("CRON_SECRET").unwrap();
}

const RECORDINGS_STORAGE_CONTAINER: &str = "episodes/recordings";
const SERIES_IMAGE_STORAGE_CONTAINER: &str = "series-images";
const EPISODE_IMAGE_STORAGE_CONTAINER: &str = "episode-images";

const USER_COLLECTION: &str = "users";
const AUTH_EMAIL_COLLECTION: &str = "auth_emails";
const EPISODE_COLLECTION: &str = "episodes";
const SERIES_COLLECTION: &str = "series";
const SERIES_USER_DATA_COLLECTION: &str = "series_user_data";
const CATEGORY_COLLECTION: &str = "categories";
const EPISODE_METADATA_COLLECTION: &str = "episode_metadata";
const OFFICE_COLLECTION: &str = "offices";
const SUBSCRIPTION_COLLECTION: &str = "subscriptions";
const RECOMMENDED_COLLECTION: &str = "recommended";

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let users = warp::path("users");
    let offices = warp::path("offices");
    let episodes = warp::path("episodes");
    let categories = warp::path("categories");
    let series = warp::path("series");
    let series_user_data = warp::path("series_user_data");
    let episode_metadata = warp::path("episode_metadata");
    let password = warp::path("password");
    let recommendations = warp::path("recommendations");

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[
            Method::OPTIONS,
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers(vec![
            "User-Agent",
            "Authorization",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Accept",
            "Range",
            "If-Range",
            "Content-Type",
            "Content-Length",
        ])
        .max_age(600);
    let user_get = maybe_box!(users
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(api::user_get));
    let user_delete = maybe_box!(users
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(api::user_delete));
    let user_put = maybe_box!(users
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(api::user_put));
    let user_image_put = maybe_box!(users
        .and(warp::path::param())
        .and(warp::path("image"))
        .and(warp::path::end())
        .and(warp::put())
        .and(filters::with_token())
        .and(filters::with_version())
        .and(warp::body::content_length_limit(1024 * 1000 * 16)) // 16 mb.
        .and(warp::filters::multipart::form().max_length(1024 * 1000 * 16)) // 16 mb.
        .and_then(api::user_image_put));
    let signup = maybe_box!(users
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_version())
        .and_then(api::signup));
    let signin = maybe_box!(users
        .and(warp::path("signin"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_version())
        .and_then(api::signin));
    let refresh_token = maybe_box!(users
        .and(warp::path::param())
        .and(warp::path("token"))
        .and(warp::path("refresh"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_version())
        .and_then(api::refresh_token));
    let user_poll = maybe_box!(users
        .and(warp::path::param())
        .and(warp::path("poll"))
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and(filters::with_range())
        .and(filters::with_since())
        .and_then(api::user_poll));
    let change_password = maybe_box!(users
        .and(warp::path::param())
        .and(password)
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_optional_token())
        .and(filters::with_version())
        .and_then(api::change_password));
    let forgot_password = maybe_box!(users
        .and(password)
        .and(warp::path("forgot"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_version())
        .and_then(api::forgot_password));
    let user_roles_put = maybe_box!(users
        .and(warp::path::param())
        .and(warp::path("roles"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(api::user_roles_put));
    let user_device_post = maybe_box!(users
        .and(warp::path::param())
        .and(warp::path("devices"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(api::user_device_post));
    let office_post = maybe_box!(offices
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Office::post));
    let office_put = maybe_box!(offices
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Office::put));
    let office_get = maybe_box!(offices
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Office::get));
    let office_delete = maybe_box!(offices
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Office::delete));
    let office_poll = maybe_box!(offices
        .and(warp::path::param())
        .and(warp::path("poll"))
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and(filters::with_range())
        .and(filters::with_since())
        .and_then(api::office_poll));
    let get_all_offices = maybe_box!(offices
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and(filters::with_since())
        .and_then(api::get_all_offices));
    let series_post = maybe_box!(offices
        .and(warp::path::param())
        .and(series)
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Series::post));
    let series_put = maybe_box!(offices
        .and(warp::path::param())
        .and(series)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Series::put));
    let series_get = maybe_box!(offices
        .and(warp::path::param())
        .and(series)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Series::get));
    let series_delete = maybe_box!(offices
        .and(warp::path::param())
        .and(series)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Series::delete));
    let series_image = maybe_box!(offices
        .and(warp::path::param())
        .and(series)
        .and(warp::path::param())
        .and(warp::path("image"))
        .and(warp::path::end())
        .and(warp::put())
        .and(filters::with_token())
        .and(filters::with_version())
        .and(warp::body::content_length_limit(1024 * 1000 * 16)) // 16 mb.
        .and(warp::filters::multipart::form().max_length(1024 * 1000 * 16)) // 16 mb.
        .and_then(Series::image));
    let episode_post = maybe_box!(offices
        .and(warp::path::param())
        .and(episodes)
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Episode::post));
    let episode_put = maybe_box!(offices
        .and(warp::path::param())
        .and(episodes)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Episode::put));
    let episode_get = maybe_box!(offices
        .and(warp::path::param())
        .and(episodes)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Episode::get));
    let episode_delete = maybe_box!(offices
        .and(warp::path::param())
        .and(episodes)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Episode::delete));
    let episode_image = maybe_box!(offices
        .and(warp::path::param())
        .and(episodes)
        .and(warp::path::param())
        .and(warp::path("image"))
        .and(warp::path::end())
        .and(warp::put())
        .and(filters::with_token())
        .and(filters::with_version())
        .and(warp::body::content_length_limit(1024 * 1000 * 16)) // 16 mb.
        .and(warp::filters::multipart::form().max_length(1024 * 1000 * 16)) // 16 mb.
        .and_then(Episode::image));
    let episode_recording = maybe_box!(offices
        .and(warp::path::param())
        .and(episodes)
        .and(warp::path::param())
        .and(warp::path("sound"))
        .and(warp::path::end())
        .and(warp::put())
        .and(filters::with_token())
        .and(filters::with_version())
        .and(warp::body::content_length_limit(1024 * 1000 * 750)) // 750 mb.
        .and(warp::filters::multipart::form().max_length(1024 * 1000 * 750)) // 750 mb.
        .and_then(Episode::recording_put));
    let episode_meta_post = maybe_box!(users
        .and(warp::path::param())
        .and(episode_metadata)
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(EpisodeMetadata::post));
    let episode_meta_put = maybe_box!(users
        .and(warp::path::param())
        .and(episode_metadata)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(EpisodeMetadata::put));
    let episode_meta_get = maybe_box!(users
        .and(warp::path::param())
        .and(episode_metadata)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(EpisodeMetadata::get));
    let episode_meta_delete = maybe_box!(users
        .and(warp::path::param())
        .and(episode_metadata)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(EpisodeMetadata::delete));
    let series_user_data_post = maybe_box!(users
        .and(warp::path::param())
        .and(series_user_data)
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(SeriesUserData::post));
    let series_user_data_put = maybe_box!(users
        .and(warp::path::param())
        .and(series_user_data)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(SeriesUserData::put));
    let series_user_data_get = maybe_box!(users
        .and(warp::path::param())
        .and(series_user_data)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(SeriesUserData::get));
    let series_user_data_delete = maybe_box!(users
        .and(warp::path::param())
        .and(series_user_data)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(SeriesUserData::delete));
    let recommended_post = maybe_box!(offices
        .and(warp::path::param())
        .and(recommendations)
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Recommendation::post));
    let recommended_put = maybe_box!(offices
        .and(warp::path::param())
        .and(recommendations)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Recommendation::put));
    let recommended_get = maybe_box!(offices
        .and(warp::path::param())
        .and(recommendations)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Recommendation::get));
    let recommended_delete = maybe_box!(offices
        .and(warp::path::param())
        .and(recommendations)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Recommendation::delete));
    let category_post = maybe_box!(offices
        .and(warp::path::param())
        .and(categories)
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Category::post));
    let category_put = maybe_box!(offices
        .and(warp::path::param())
        .and(categories)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::put())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Category::put));
    let category_get = maybe_box!(offices
        .and(warp::path::param())
        .and(categories)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Category::get));
    let category_delete = maybe_box!(offices
        .and(warp::path::param())
        .and(categories)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::delete())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(Category::delete));

    let subscriptions = warp::path("subscriptions");
    let subscription_get = users
        .and(warp::path::param())
        .and(subscriptions)
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::get())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(api::subscription_get)
        .boxed();
    let subscription_post = users
        .and(warp::path::param())
        .and(subscriptions)
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_token())
        .and(filters::with_version())
        .and_then(api::subscription_post)
        .boxed();

    let cron = warp::path("cron")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_version())
        .and_then(api::cron)
        .boxed();

    let users_registered_in_period = warp::path("new_users_email")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(filters::with_version())
        .and_then(api::new_users_email)
        .boxed();

    let webhooks = warp::path("webhooks");
    let webhook_subscription_apple = webhooks
        .and(subscriptions)
        .and(warp::path("apple"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and_then(api::webhook_subscription_apple)
        .boxed();

    // Required by Azure health checks.
    let main = warp::path::end().map(|| warp::reply());

    // Used by CORS preflight requests.
    let options = warp::any().and(warp::options()).map(|| warp::reply());

    let routes = maybe_box!(main
        .or(user_get)
        .or(user_delete)
        .or(user_put)
        .or(user_image_put)
        .or(user_roles_put)
        .or(user_device_post)
        .or(signin)
        .or(signup)
        .or(refresh_token)
        .or(user_poll)
        .or(forgot_password)
        .or(change_password)
        .or(series_post)
        .or(series_put)
        .or(series_get)
        .or(series_delete)
        .or(series_image)
        .or(episode_post)
        .or(episode_put)
        .or(episode_get)
        .or(episode_delete)
        .or(episode_image)
        .or(episode_recording)
        .or(episode_meta_post)
        .or(episode_meta_put)
        .or(episode_meta_get)
        .or(episode_meta_delete)
        .or(series_user_data_post)
        .or(series_user_data_put)
        .or(series_user_data_get)
        .or(series_user_data_delete)
        .or(office_post)
        .or(office_put)
        .or(office_get)
        .or(office_delete)
        .or(office_poll)
        .or(get_all_offices)
        .or(subscription_post)
        .or(subscription_get)
        .or(webhook_subscription_apple)
        .or(recommended_post)
        .or(recommended_put)
        .or(recommended_get)
        .or(recommended_delete)
        .or(category_post)
        .or(category_put)
        .or(category_get)
        .or(category_delete)
        .or(cron)
        .or(users_registered_in_period)
        .or(options)
        .recover(filters::handle_rejection)
        .with(&cors));

    routes
}

#[tokio::main]
async fn main() {
    let routes = routes();
    let cosmos_state = CosmosState {
        cosmos_account: COSMOS_ACCOUNT.to_string(),
        cosmos_database: COSMOS_DATABASE.to_string(),
        cosmos_master_key: COSMOS_MASTER_KEY.to_string(),
        storage_account: Some(STORAGE_ACCOUNT.to_string()),
        storage_master_key: Some(STORAGE_MASTER_KEY.to_string()),
        image_storage_container: None,
    };
    set_state(cosmos_state);

    if cfg!(debug_assertions) {
        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
    } else {
        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await
    }
}
