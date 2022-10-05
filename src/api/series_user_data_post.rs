use crate::models::{Claims, Series, SeriesUserData};
use crate::util::{DataRequest, Empty};
use crate::{SERIES_COLLECTION, SERIES_USER_DATA_COLLECTION};
use cosmos_utils::{get, CosmosErrorKind};

impl SeriesUserData {
    pub async fn post(
        user_id: String,
        r: DataRequest<SeriesUserData, Empty>,
        claims: Claims,
        _v: u8,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut instance;
        if let Some(q) = r.data {
            instance = q;
        } else {
            return Err(warp::reject::custom(crate::fault::Fault::NoData));
        }
        if instance.user_id != user_id {
            return Err(warp::reject::custom(crate::fault::Fault::IllegalArgument(
                format!(
                    "user_id does not match url ({} != {}).",
                    instance.user_id, user_id
                ),
            )));
        }
        // Check that this episode exists and has the right series ID
        let (_, _): (Series, _) = match get(
            SERIES_COLLECTION,
            [&instance.office_id],
            &instance.series_id,
        )
        .await
        {
            Ok(e) => e,
            Err(e) => match e.kind {
                CosmosErrorKind::NotFound => {
                    return Err(warp::reject::custom(crate::fault::Fault::IllegalArgument(
                        format!(
                            "That series [{}] does not exist in that office [{}].",
                            instance.series_id, instance.office_id
                        ),
                    )));
                }
                _ => return Err(e.into()),
            },
        };

        if claims.sub != user_id {
            return Err(warp::reject::custom(crate::fault::Fault::Forbidden(
                format!(
                    "Calling user does not have the privilege, {} != {}",
                    claims.sub, user_id
                ),
            )));
        }

        // NOTE: We make the id for this the concatenation of the office-id to the series-id with
        // a + between
        // This in order to be able to cheaply detect duplicates in the database
        instance.id = format!("{}+{}", instance.office_id, instance.series_id);
        instance.modified = chrono::Utc::now();
        match cosmos_utils::insert(
            SERIES_USER_DATA_COLLECTION,
            [&instance.user_id],
            &instance,
            None,
        )
        .await
        {
            Ok(_sud) => {}
            Err(e) => match e.kind {
                CosmosErrorKind::Conflict => {
                    // In the case where the episodemetadata already exists we return that
                    let (sud, _): (SeriesUserData, _) = get(
                        SERIES_USER_DATA_COLLECTION,
                        [&instance.user_id],
                        &instance.id,
                    )
                    .await?;
                    return Ok(warp::reply::json(&crate::util::DataResponse {
                        data: Some(sud),
                        extra: None::<crate::util::Empty>,
                    }));
                }
                _ => {
                    return Err(e.into());
                }
            },
        };

        Ok(warp::reply::json(&crate::util::DataResponse {
            data: Some(instance),
            extra: None::<crate::util::Empty>,
        }))
    }
}
