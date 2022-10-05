use crate::models::{Claims, Episode, EpisodeMetadata};
use crate::util::{DataRequest, Empty};
use crate::{EPISODE_COLLECTION, EPISODE_METADATA_COLLECTION};
use cosmos_utils::{modify, modify_async};

impl EpisodeMetadata {
    pub async fn put(
        user_id: String,
        episode_metadata_id: String,
        r: DataRequest<EpisodeMetadata, Empty>,
        claims: Claims,
        _v: u8,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let new_instance;
        if let Some(q) = r.data {
            new_instance = q;
        } else {
            return Err(warp::reject::custom(crate::fault::Fault::NoData));
        }
        if new_instance.user_id != user_id {
            return Err(warp::reject::custom(crate::fault::Fault::IllegalArgument(
                format!(
                    "user_id does not match url ({} != {}).",
                    new_instance.user_id, user_id
                ),
            )));
        }
        if new_instance.id != episode_metadata_id {
            return Err(warp::reject::custom(crate::fault::Fault::IllegalArgument(
                format!(
                    "episode_metadata_id does not match url ({} != {}).",
                    new_instance.id, episode_metadata_id
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
        let instance = modify_async(
            EPISODE_METADATA_COLLECTION,
            [&user_id],
            &episode_metadata_id,
            |old_instance: Self| {
                let mut instance = new_instance.clone();
                async move {
                    // Increment the likes on the episode based on change in favourite status
                    if old_instance.favourite != instance.favourite {
                        if old_instance.favourite {
                            modify(
                                EPISODE_COLLECTION,
                                [&old_instance.office_id],
                                &old_instance.episode_id,
                                |mut old_episode: Episode| {
                                    old_episode.likes -= 1;
                                    old_episode.modified = chrono::Utc::now();
                                    Ok(old_episode)
                                },
                            )
                            .await?;
                        } else {
                            modify(
                                EPISODE_COLLECTION,
                                [&old_instance.office_id],
                                &old_instance.episode_id,
                                |mut old_episode: Episode| {
                                    old_episode.likes += 1;
                                    old_episode.modified = chrono::Utc::now();
                                    Ok(old_episode)
                                },
                            )
                            .await?;
                        }
                    }
                    // NOTE: Office, series and episode ID are not allowed to change. Nor is
                    // deleted.
                    instance.office_id = old_instance.office_id;
                    instance.series_id = old_instance.series_id;
                    instance.episode_id = old_instance.episode_id;
                    instance.deleted = old_instance.deleted;
                    instance.modified = chrono::Utc::now();
                    Ok(instance)
                }
            },
        )
        .await?;
        Ok(warp::reply::json(&crate::util::DataResponse {
            data: Some(instance),
            extra: None::<crate::util::Empty>,
        }))
    }
}
