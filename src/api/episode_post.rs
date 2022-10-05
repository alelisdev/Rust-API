use crate::models::{Claims, Episode, RoleFlags};
use crate::util::{has_role, DataRequest, Empty};
use crate::EPISODE_COLLECTION;

impl Episode {
    pub async fn post(
        office_id: String,
        r: DataRequest<Episode, Empty>,
        claims: Claims,
        _v: u8,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut instance;
        if let Some(q) = r.data {
            instance = q;
        } else {
            return Err(warp::reject::custom(crate::fault::Fault::NoData));
        }
        if instance.office_id != office_id {
            return Err(warp::reject::custom(crate::fault::Fault::IllegalArgument(
                format!(
                    "office_id does not match url ({} != {}).",
                    instance.office_id, office_id
                ),
            )));
        }
        if !has_role(None, &claims, RoleFlags::OFFICE_CONTENT_ADMIN) {
            return Err(warp::reject::custom(crate::fault::Fault::Forbidden(
                format!("Calling user does not have the privilege.",),
            )));
        }
        instance.id = uuid::Uuid::new_v4().to_string();
        instance.published = chrono::Utc::now();
        instance.modified = chrono::Utc::now();
        cosmos_utils::insert(EPISODE_COLLECTION, [&instance.office_id], &instance, None).await?;
        Ok(warp::reply::json(&crate::util::DataResponse {
            data: Some(instance),
            extra: None::<crate::util::Empty>,
        }))
    }
}
