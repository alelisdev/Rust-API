use crate::models::{Claims, Episode, EpisodeMetadata};
use crate::util::{DataRequest, Empty};
use crate::{EPISODE_COLLECTION, EPISODE_METADATA_COLLECTION};
use cosmos_utils::{get, upsert, CosmosErrorKind};

impl EpisodeMetadata {
    pub async fn post(
        user_id: String,
        r: DataRequest<EpisodeMetadata, Empty>,
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
        let (mut episode, etag): (Episode, _) = match get(
            EPISODE_COLLECTION,
            [&instance.office_id],
            &instance.episode_id,
        )
        .await
        {
            Ok(e) => e,
            Err(e) => match e.kind {
                CosmosErrorKind::NotFound => {
                    return Err(warp::reject::custom(crate::fault::Fault::IllegalArgument(
                        format!(
                            "That episode [{}] does not exist in that office [{}].",
                            instance.episode_id, instance.office_id
                        ),
                    )));
                }
                _ => return Err(e.into()),
            },
        };
        if instance.series_id != episode.series_id {
            return Err(warp::reject::custom(crate::fault::Fault::IllegalArgument(
                format!(
                    "series_id does not match episode ({} != {}).",
                    instance.series_id, episode.series_id
                ),
            )));
        }

        if claims.sub != user_id {
            return Err(warp::reject::custom(crate::fault::Fault::Forbidden(
                format!(
                    "Calling user does not have the privilege, {} != {}",
                    claims.sub, user_id
                ),
            )));
        }

        // NOTE: We make the id for this the concatenation of the office-id to the episode-id with
        // a + between
        // This in order to be able to cheaply be able to detect duplicates in the database
        instance.id = format!("{}+{}", instance.office_id, instance.episode_id);
        instance.modified = chrono::Utc::now();
        match cosmos_utils::insert(
            EPISODE_METADATA_COLLECTION,
            [&instance.user_id],
            &instance,
            None,
        )
        .await
        {
            Ok(_metadata) => {}
            Err(e) => match e.kind {
                CosmosErrorKind::Conflict => {
                    // In the case where the episodemetadata already exists we return that
                    let (metadata, _): (EpisodeMetadata, _) = get(
                        EPISODE_METADATA_COLLECTION,
                        [&instance.user_id],
                        &instance.id,
                    )
                    .await?;
                    return Ok(warp::reply::json(&crate::util::DataResponse {
                        data: Some(metadata),
                        extra: None::<crate::util::Empty>,
                    }));
                }
                _ => {
                    return Err(e.into());
                }
            },
        };

        // Make sure this happens after metadata has been posted so we can't duplicate the metadata
        // to make the likes go up more than it should
        if instance.favourite {
            // Increment the likes count
            episode.likes += 1;
            episode.modified = chrono::Utc::now();
            upsert(
                EPISODE_COLLECTION,
                [&episode.office_id],
                &episode,
                Some(&etag),
            )
            .await?;
        }
        Ok(warp::reply::json(&crate::util::DataResponse {
            data: Some(instance),
            extra: None::<crate::util::Empty>,
        }))
    }
}
