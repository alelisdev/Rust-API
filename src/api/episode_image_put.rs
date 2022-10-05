use crate::fault::Fault;
use crate::models::{Claims, Episode, RoleFlags};
use crate::util::{has_role, DataResponse, Empty};
use crate::{EPISODE_COLLECTION, EPISODE_IMAGE_STORAGE_CONTAINER};
use chrono::Utc;
use cosmos_utils::{get, upload_blob, upsert};
use warp::filters::multipart::FormData;
use warp::reject;

impl Episode {
    pub async fn image(
        office_id: String,
        episode_id: String,
        claims: Claims,
        _v: u8,
        f: FormData,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        if !has_role(Some(&office_id), &claims, RoleFlags::OFFICE_CONTENT_ADMIN) {
            return Err(reject::custom(Fault::Forbidden(format!(
                "Caller is not a office content admin",
            ))));
        }

        let (mut episode, etag): (Episode, _) =
            get(EPISODE_COLLECTION, [&office_id], &episode_id).await?;

        let image_id = upload_blob(f, "image", "image", EPISODE_IMAGE_STORAGE_CONTAINER).await?;
        episode.images.push(image_id);
        episode.modified = Utc::now();

        upsert(EPISODE_COLLECTION, [&office_id], &episode, Some(&etag)).await?;

        // TODO: Delete old image, if any.
        Ok(warp::reply::json(&DataResponse {
            data: Some(episode),
            extra: None::<Empty>,
        }))
    }
}
