use crate::{
    fault::Fault,
    models::{AuthEmail, Claims, User},
    util::{self, DataRequest, DataResponse, Empty},
    ACCESS_TOKEN_SECRET, APPLICATION_INSIGHTS_INSTRUMENTATION_KEY, AUTH_EMAIL_COLLECTION,
    DEFAULT_OFFICE_ID, PRODUCTION_ENVIRONMENT, REFRESH_TOKEN_SECRET, SENDGRID_API_KEY,
    USER_COLLECTION,
};
use appinsights::TelemetryClient;
use chrono::{prelude::*, Duration};
use cosmos_utils::{delete, insert};
use jsonwebtoken::{encode, EncodingKey, Header};
use sendgrid::v3::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::reject;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response<'a> {
    pub access_token: &'a str,
    pub refresh_token: &'a str,
    pub user_id: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupData {
    password: Option<String>,
    //apple_authentication_code: Option<String>,
}

pub async fn signup(
    r: DataRequest<User, SignupData>,
    _v: u8,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut user = if let Some(q) = r.data {
        q
    } else {
        return Err(reject::custom(Fault::NoData));
    };
    let signup_data = if let Some(q) = r.extra {
        q
    } else {
        return Err(reject::custom(Fault::NoExtra));
    };
    let password;
    //if signup_data.apple_authentication_code.is_some() {
    //    let authentication_code = signup_data.apple_authentication_code.unwrap();
    //    let client = Client::new(
    //        APPLE_SIGNIN_CLIENT_ID.to_string(),
    //        APPLE_SIGNIN_PRIVATE_KEY.to_string(),
    //        APPLE_SIGNIN_KID.to_string(),
    //        APPLE_SIGNIN_TEAM_ID.to_string(),
    //    )
    //    .unwrap();
    //    client.authenticate(authentication_code).await.unwrap();
    //    unimplemented!();
    if signup_data.password.is_some() {
        password = signup_data.password.unwrap();
    } else {
        return Err(reject::custom(Fault::NoExtra));
    }

    if user.id == "" {
        user.id = Uuid::new_v4().to_string();
    }

    // If no preferred name is given then the preferred name is set to the first name
    if let None = user.preferred_name {
        user.preferred_name = Some(user.first_name.clone());
    }

    // // Make date from IANA location.
    // let tz: Tz = "Europe/Stockholm".parse().unwrap(); // "Antarctica/South_Pole"
    // let dt = tz.ymd(1977, 10, 26).and_hms(7, 0, 0);
    // let fo = dt.with_timezone(&dt.offset().fix());
    // println!("tz = {}", dt);
    // user.dob = Some(fo);

    // Automatically make user a test user if on the test server.
    if !*PRODUCTION_ENVIRONMENT {
        user.test = true;
    }

    user.office_ids = vec![DEFAULT_OFFICE_ID.to_string()];

    user.created = chrono::Utc::now();

    // Set at.
    user.modified = chrono::Utc::now();

    let user_etag = insert(USER_COLLECTION, [&user.id], &user, None).await?;

    // Normalise email.
    let email = user.email.clone().to_lowercase();

    // Add email auth.
    let email_auth = AuthEmail {
        id: email,
        passhash: util::hash(password.as_bytes()), // Calculate pass hash.
        user_id: user.id.clone(),
    };

    // TODO(Jonathan): Should we try to make sure the email is a real email?
    match insert(AUTH_EMAIL_COLLECTION, [&email_auth.id], &email_auth, None).await {
        Ok(_) => (),
        Err(e) => {
            // NOTE: Error here should hard delete the previously inserted document.
            delete(USER_COLLECTION, [&user.id], &user.id, Some(user_etag)).await?;
            return Err(e.into());
        }
    }

    // Report event.
    let application_insight =
        TelemetryClient::new(APPLICATION_INSIGHTS_INSTRUMENTATION_KEY.to_string());
    application_insight.track_event("User signed up.");

    // Send welcome email.
    let mut map = SGMap::new();
    map.insert(String::from("firstName"), user.first_name.clone());

    //let p = Personalization::new(Email::new(SUPPORT_EMAIL_SENDER.as_str()))
    //    .add_to(Email::new(&user.email))
    //    .add_dynamic_template_data(map);

    //let m = Message::new(Email::new(SUPPORT_EMAIL_SENDER.as_str()))
    //    .set_template_id("d-ec5746d2bcbe4ecb91082021c1dfb7d4")
    //    .add_personalization(p);
    //let sender = Sender::new(SENDGRID_API_KEY.to_string());
    //match sender.send(&m).await {
    //    Ok(_) => {}
    //    Err(e) => {
    //        // TODO(Jonathan): Log could not send welcome email.
    //        println!("{:?}", e);
    //    }
    //};

    let iat = Utc::now();
    let exp = iat + Duration::minutes(20);
    let claims = Claims::new(&user.id, exp, &vec![]);

    let access_token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(ACCESS_TOKEN_SECRET.as_ref()),
    ) {
        Ok(token) => token,
        Err(error) => {
            return Err(reject::custom(Fault::Unspecified(format!(
                "Could not encode access token: {}.",
                error.to_string()
            ))));
        }
    };

    let refresh_token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(REFRESH_TOKEN_SECRET.as_ref()),
    ) {
        Ok(token) => token,
        Err(error) => {
            return Err(reject::custom(Fault::Unspecified(format!(
                "Could not encode refresh token: {}.",
                error.to_string()
            ))));
        }
    };

    // Send email.
    let mut map = SGMap::new();
    map.insert(String::from("firstName"), user.first_name);

    let p = Personalization::new(Email::new(&user.email)).add_dynamic_template_data(map);

    let m = Message::new(Email::new("info@primecrime.se"))
        .set_template_id("d-22ced70464074d07bba3d6c66da17b71")
        .add_personalization(p);
    let sender = Sender::new(SENDGRID_API_KEY.to_string());
    match sender.send(&m).await {
        _ => {}
    };

    Ok(warp::reply::json(&DataResponse {
        data: Some(&Response {
            access_token: &access_token,
            refresh_token: &refresh_token,
            user_id: &email_auth.user_id,
        }),
        extra: None::<Empty>,
    }))
}
