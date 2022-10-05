use crate::models::{Claims, Office};
use crate::util::{DataResponse, Empty};
use crate::OFFICE_COLLECTION;
use chrono::{DateTime, Utc};
use cosmos_utils::{query_crosspartition, CosmosErrorStruct};

pub async fn get_all_offices(
    _claims: Claims,
    _v: u8,
    since: Option<DateTime<Utc>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let since = match since {
        Some(since) => format!(" WHERE o.modified >= {}", since.timestamp()),
        None => String::from(""),
    };

    // Offices
    let q = format!("SELECT * FROM {} o{}", &*OFFICE_COLLECTION, since);
    let offices = async {
        let offices: Vec<Office> =
            query_crosspartition(OFFICE_COLLECTION, &[()], q, -1, true).await?;
        Result::<_, CosmosErrorStruct>::Ok(offices)
    }
    .await?;

    Ok(warp::reply::json(&DataResponse {
        data: Some(&offices),
        extra: None::<Empty>,
    }))
}
