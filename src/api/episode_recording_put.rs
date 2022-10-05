use crate::models::{Claims, Episode, RoleFlags};
use crate::util::Empty;
use crate::{EPISODE_COLLECTION, RECORDINGS_STORAGE_CONTAINER};
use chrono::Utc;
use warp::filters::multipart::FormData;

impl Episode {
    pub async fn recording_put(
        office_id: String,
        episode_id: String,
        claims: Claims,
        _v: u8,
        f: FormData,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        if !crate::util::has_role(Some(&office_id), &claims, RoleFlags::OFFICE_CONTENT_ADMIN) {
            return Err(warp::reject::custom(crate::fault::Fault::Forbidden(
                format!("Insufficient roles, caller does not have privileges for office",),
            )));
        }
        let (mut instance, etag): (Self, _) =
            cosmos_utils::get(EPISODE_COLLECTION, [&office_id], &episode_id).await?;
        let sound_id =
            cosmos_utils::upload_blob(f, "sound", "audio/", RECORDINGS_STORAGE_CONTAINER).await?;
        instance.sound_file = Some(sound_id);
        instance.modified = Utc::now();

        cosmos_utils::upsert(EPISODE_COLLECTION, [&office_id], &instance, Some(&etag)).await?;

        // TODO: Delete old sound, if any.
        Ok(warp::reply::json(&crate::util::DataResponse {
            data: Some(instance),
            extra: None::<Empty>,
        }))
    }
}
