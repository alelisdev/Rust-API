use crate::fault::Fault;
use crate::models::{AuthEmail, User};
use crate::util::{self, log, DataRequest, DataResponse, Empty};
use crate::SENDGRID_API_KEY;
use crate::{AUTH_EMAIL_COLLECTION, USER_COLLECTION};
use cosmos_utils::get;
use cosmos_utils::modify;
use sendgrid::v3::*;
use warp::reject;

pub async fn forgot_password(
    r: DataRequest<String, Empty>,
    _v: u8,
) -> Result<impl warp::Reply, warp::Rejection> {
    let email;
    if let Some(q) = r.data {
        // Normalise email.
        email = q.to_lowercase();
    } else {
        return Err(reject::custom(Fault::NoData));
    }

    // Generate new password.
    let password = util::random_string(8);

    let auth_email: AuthEmail = modify(
        AUTH_EMAIL_COLLECTION,
        [&email],
        &email,
        |mut auth_email: AuthEmail| {
            // Change password.
            auth_email.passhash = util::hash(password.as_bytes());
            Ok(auth_email)
        },
    )
    .await?;

    let (user, _etag): (User, _) =
        get(USER_COLLECTION, [&auth_email.user_id], &auth_email.user_id).await?;
    // Send email.
    let mut map = SGMap::new();
    map.insert(String::from("temporaryPassword"), password);

    let p = Personalization::new(Email::new(&user.email)).add_dynamic_template_data(map);

    let m = Message::new(Email::new("info@primecrime.se"))
        .set_template_id("d-b1a6ec1654ba478b88fa035b99c72863")
        .add_personalization(p);
    let sender = Sender::new(SENDGRID_API_KEY.to_string());
    match sender.send(&m).await {
        Ok(_) => (),
        Err(e) => {
            log(format!("Could not send forgot password email due to {}", e));
        }
    };

    Ok(warp::reply::json(&DataResponse {
        data: None::<Empty>,
        extra: None::<Empty>,
    }))
}
