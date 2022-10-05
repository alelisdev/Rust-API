use crate::fault::Fault;
use crate::models::{Category, Claims, Episode, Office, Recommendation, RoleFlags, Series};
use crate::util::{self, has_role, DataResponse, Empty};
use crate::{
    CATEGORY_COLLECTION, EPISODE_COLLECTION, OFFICE_COLLECTION, RECOMMENDED_COLLECTION,
    SERIES_COLLECTION,
};
use chrono::{DateTime, Utc};
use cosmos_utils::{get, query, CosmosErrorStruct};
use serde::Serialize;
use warp::{
    http::{header, Response},
    reject,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficePollDataResponse<'a> {
    #[serde(skip_serializing_if = "util::is_none")]
    pub office: Option<&'a Office>,
    #[serde(skip_serializing_if = "util::is_none")]
    pub recommendation: Option<&'a Recommendation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<Category>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub series: Vec<Series>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub episodes: Vec<Episode>,
}

pub async fn office_poll(
    office_id: String,
    claims: Claims,
    _v: u8,
    _range: u16,
    since: Option<DateTime<Utc>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !has_role(Some(&office_id), &claims, RoleFlags::OFFICE_CONTENT_ADMIN) {
        return Err(reject::custom(Fault::Forbidden(format!(
            "User is not an office content admin for {}.",
            office_id,
        ))));
    }

    // Office
    let (office, _): (Office, _) = get(OFFICE_COLLECTION, [&office_id], &office_id).await?;

    let mut new_office = Some(&office);
    // Remove all of the entries that have not been updated since `since`.
    // TODO: This filtering could be done in the queries rather than after getting them from the
    // database
    if let Some(since) = since {
        new_office = if office.modified >= since {
            Some(&office)
        } else {
            None
        };
    }
    let office = new_office;

    let since = match since {
        Some(since) => format!(" AND o.modified >= {}", since.timestamp()),
        None => String::from(""),
    };

    // recommendation
    let q = format!(
        "SELECT * FROM {} o WHERE o.officeId = \"{}\"{}",
        &*RECOMMENDED_COLLECTION, &office_id, since
    );
    let recommendation = async {
        let rec: Vec<Recommendation> = query(RECOMMENDED_COLLECTION, [&office_id], q, -1).await?;
        Result::<_, CosmosErrorStruct>::Ok(rec)
    };
    // categories
    let q = format!(
        "SELECT * FROM {} o WHERE o.officeId = \"{}\"{}",
        &*CATEGORY_COLLECTION, &office_id, since
    );
    let categories = async {
        let cat: Vec<Category> = query(CATEGORY_COLLECTION, [&office_id], q, -1).await?;
        Result::<_, CosmosErrorStruct>::Ok(cat)
    };
    // series
    let q = format!(
        "SELECT * FROM {} o WHERE o.officeId = \"{}\"{}",
        &*SERIES_COLLECTION, &office_id, since
    );
    let series = async {
        let ser: Vec<Series> = query(SERIES_COLLECTION, [&office_id], q, -1).await?;
        Result::<_, CosmosErrorStruct>::Ok(ser)
    };
    // episodes
    let q = format!(
        "SELECT * FROM {} o WHERE o.officeId = \"{}\"{}",
        &*EPISODE_COLLECTION, &office_id, since
    );
    let episodes = async {
        let epi: Vec<Episode> = query(EPISODE_COLLECTION, [&office_id], q, -1).await?;
        Result::<_, CosmosErrorStruct>::Ok(epi)
    };

    let (recommendation, categories, series, episodes) =
        tokio::join!(recommendation, categories, series, episodes,);
    let mut recommendation = recommendation?;
    let categories = categories?;
    let series = series?;
    let episodes = episodes?;

    // There should only be 1 recommendation per office
    let res = match serde_json::to_string(&DataResponse {
        data: Some(&OfficePollDataResponse {
            office,
            categories,
            series,
            episodes,
            recommendation: recommendation.pop().as_ref(),
        }),
        extra: None::<Empty>,
    }) {
        Ok(g) => g,
        Err(err) => {
            return Err(reject::custom(Fault::Unspecified(format!(
                "Could not serialize response into json: {}.",
                err.to_string()
            ))));
        }
    };

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::LAST_MODIFIED,
            format!("{}", Utc::now().format("%a, %d %b %Y %H:%M:%S GMT")),
        )
        .body(res))
}
