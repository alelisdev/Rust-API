use crate::fault::Fault;
use crate::models::{Claims, RoleFlags, Series};
use crate::util::{has_role, DataResponse, Empty};
use crate::{SERIES_COLLECTION, SERIES_IMAGE_STORAGE_CONTAINER};
use chrono::Utc;
use cosmos_utils::{get, upload_blob, upsert};
use warp::filters::multipart::FormData;
use warp::reject;

impl Series {
    pub async fn image(
        office_id: String,
        series_id: String,
        claims: Claims,
        _v: u8,
        f: FormData,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        if !has_role(Some(&office_id), &claims, RoleFlags::OFFICE_CONTENT_ADMIN) {
            return Err(reject::custom(Fault::Forbidden(format!(
                "Caller is not a office content admin",
            ))));
        }

        let (mut series, etag): (Series, _) =
            get(SERIES_COLLECTION, [&office_id], &series_id).await?;

        let image_id = upload_blob(f, "image", "image", SERIES_IMAGE_STORAGE_CONTAINER).await?;
        series.images.push(image_id);
        series.modified = Utc::now();

        upsert(SERIES_COLLECTION, [&office_id], &series, Some(&etag)).await?;

        // TODO: Delete old image, if any.
        Ok(warp::reply::json(&DataResponse {
            data: Some(series),
            extra: None::<Empty>,
        }))
    }
}
