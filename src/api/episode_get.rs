use crate::models::{Episode, Subscription};
use crate::{EPISODE_COLLECTION, SUBSCRIPTION_COLLECTION};
use cosmos_utils::CosmosErrorKind;

impl Episode {
    pub async fn get(
        office_id: String,
        episode_id: String,
        claims: crate::models::Claims,
        _v: u8,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let (mut instance, _etag): (Self, _) =
            cosmos_utils::get(EPISODE_COLLECTION, [&office_id], &episode_id).await?;
        // Check that this user has a subscription, if not then set the sound_file field to None
        let mut has_sub = false;
        for _ in 0..1 {
            let (sub, _): (Subscription, _) =
                match cosmos_utils::get(SUBSCRIPTION_COLLECTION, [&claims.sub], &office_id).await {
                    Ok(s) => s,
                    Err(e) => match e.kind {
                        CosmosErrorKind::NotFound => {
                            break;
                        }
                        _ => return Err(e.into()),
                    },
                };
            has_sub = match sub.end {
                Some(end) => chrono::Utc::now() < end,
                None => true,
            };
        }

        // TODO: Make this happen when !has_sub
        if false {
            instance.sound_file = None;
        }

        Ok(warp::reply::json(&crate::util::DataResponse {
            data: Some(instance),
            extra: None::<crate::util::Empty>,
        }))
    }
}
