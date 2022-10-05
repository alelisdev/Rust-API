use crate::{
    fault::Fault,
    models::User,
    util::{log, DataRequest, DataResponse, Empty},
    CRON_SECRET, SENDGRID_API_KEY, USER_COLLECTION,
};
use chrono::{Duration, Timelike, Utc};
use cosmos_utils::query_crosspartition;
use sendgrid::v3::*;
use warp::reject;

pub async fn new_users_email(
    r: DataRequest<Empty, String>,
    _v: u8,
) -> Result<impl warp::Reply, warp::Rejection> {
    match r.extra {
        Some(secret) => {
            if &secret != &*CRON_SECRET {
                return Err(reject::custom(Fault::Unauthorized));
            }
        }
        None => {
            return Err(reject::custom(Fault::NoExtra));
        }
    };

    let end = Utc::now();
    // Set the start to the beginnig of the day
    let start = match end
        .with_hour(0)
        .and_then(|e| e.with_minute(0))
        .and_then(|e| e.with_second(0))
    {
        Some(start) => start,
        None => {
            return Err(reject::custom(Fault::IllegalState(format!(
                "Could not set Utc::now() to start of day"
            ))));
        }
    };

    // Get all users that were created between start and end
    let q = format!(
        "SELECT * FROM {} u WHERE u.created >= \"{}\" AND u.created <= \"{}\"",
        USER_COLLECTION,
        start.to_rfc3339(),
        end.to_rfc3339()
    );
    let created_users: Vec<User> =
        query_crosspartition(USER_COLLECTION, [&()], q, -1, true).await?;

    // Send email
    let mut map = SGMap::new();
    map.insert(String::from("newUsers"), created_users.len().to_string());

    let p = Personalization::new(Email::new("alex@thirdact.se"))
        .add_cc(Email::new("jonathan.hillblom@thirdact.se"))
        .add_cc(Email::new("nicklas@thirdact.se"))
        .add_cc(Email::new("stefan@primecrime.se"))
        .add_cc(Email::new("mats.pettersson@primecrime.se"))
        .add_dynamic_template_data(map);

    let m = Message::new(Email::new("info@primecrime.se"))
        .set_template_id("d-ac75e71fc03d4658b1d5ba0448972256")
        .add_personalization(p);
    let sender = Sender::new(SENDGRID_API_KEY.to_string());
    match sender.send(&m).await {
        Ok(_) => (),
        Err(e) => {
            log(format!("Could not send new users email due to {}", e));
        }
    };

    Ok(warp::reply::json(&DataResponse {
        data: None::<Empty>,
        extra: None::<Empty>,
    }))
}
