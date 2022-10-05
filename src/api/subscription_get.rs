use crate::{
    fault::Fault,
    models::{Claims, RoleFlags, Subscription},
    util::{self, DataResponse, Empty},
    SUBSCRIPTION_COLLECTION,
};
use cosmos_utils::get;
use warp::reject;

pub async fn subscription_get(
    user_id: String,
    subscription_id: String,
    claims: Claims,
    _v: u8,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (mut subscription, _): (Subscription, _) =
        get(SUBSCRIPTION_COLLECTION, [&user_id], &subscription_id).await?;

    // Is user either owner or member?
    if subscription.user_id != claims.sub
        && !util::has_role(
            Some(&subscription.office_id),
            &claims,
            RoleFlags::OFFICE_PERSONNEL_ADMIN,
        )
    {
        return Err(reject::custom(Fault::Forbidden(format!(
            "User is not the owner, nor a personnel admin for office {}.",
            subscription.office_id
        ))));
    }

    // FIXME(J): This is hiding the payments from the user, we need this temporary fix for launch,
    // but we should make the app code be able to handle getting the payment array as soon as
    // possible
    subscription.payments = vec![];

    Ok(warp::reply::json(&DataResponse {
        data: Some(subscription),
        extra: None::<Empty>,
    }))
}
