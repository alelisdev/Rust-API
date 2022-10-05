use crate::fault::Fault;
use crate::models::{
    Category, Claims, Episode, EpisodeMetadata, Office, Recommendation, Series, SeriesUserData,
    Subscription, User,
};
use crate::util::{self, DataResponse, Empty};
use crate::{
    CATEGORY_COLLECTION, EPISODE_COLLECTION, EPISODE_METADATA_COLLECTION, OFFICE_COLLECTION,
    RECOMMENDED_COLLECTION, SERIES_COLLECTION, SERIES_USER_DATA_COLLECTION,
    SUBSCRIPTION_COLLECTION, USER_COLLECTION,
};
use chrono::{DateTime, Utc};
use cosmos_utils::{get, query, CosmosErrorStruct};
use serde::Serialize;
use std::sync::Arc;
use warp::{
    http::{header, Response},
    reject,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPollDataResponse<'a> {
    #[serde(skip_serializing_if = "util::is_none")]
    pub user: Option<&'a User>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub offices: Vec<Office>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recommendations: Vec<Recommendation>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<Category>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub series: Vec<Series>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub episodes: Vec<Episode>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub episode_metadata: Vec<EpisodeMetadata>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub series_user_data: Vec<SeriesUserData>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<Subscription>,
}

pub async fn user_poll(
    user_id: String,
    claims: Claims,
    _v: u8,
    _range: u16,
    since: Option<DateTime<Utc>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if user_id != claims.sub {
        return Err(reject::custom(Fault::Forbidden(format!(
            "User id does not match signed in user ({} != {}).",
            user_id, claims.sub
        ))));
    }

    let (user, _etag): (User, _) = get(USER_COLLECTION, [&user_id], user_id.clone()).await?;
    let office_ids: Vec<_> = user
        .office_ids
        .iter()
        .cloned()
        .map(|o| Arc::new(o))
        .collect();
    let user_id = Arc::new(user_id);

    let new_user = Some(&user);
    //// Remove all of the entries that have not been updated since `since`.
    //if let Some(since) = since {
    //    new_user = if user.modified >= since {
    //        Some(&user)
    //    } else {
    //        None
    //    };
    //}
    let user = new_user;

    let mut offices = vec![];
    for office_id in &office_ids {
        // Offices
        let office_id = office_id.clone();
        offices.push(async move {
            let (off, _): (Office, _) = get(
                OFFICE_COLLECTION,
                [&office_id.as_ref()],
                &office_id.as_ref(),
            )
            .await?;
            let off = if let Some(since) = since {
                if off.modified >= since {
                    Some(off)
                } else {
                    None
                }
            } else {
                Some(off)
            };
            Result::<_, CosmosErrorStruct>::Ok(off)
        });
    }
    let offices = futures::future::join_all(offices);

    let since = match since {
        Some(since) => format!(r#" WHERE o.modified >= "{}""#, since.to_rfc3339()),
        None => String::from(""),
    };

    let mut recommendations = vec![];
    for office_id in &office_ids {
        // Recommendations
        let q = format!("SELECT * FROM {} o{}", RECOMMENDED_COLLECTION, since);
        let office_id = office_id.clone();
        recommendations.push(async move {
            let rec: Vec<Recommendation> =
                query(RECOMMENDED_COLLECTION, [&office_id.as_ref()], q, -1).await?;
            Result::<_, CosmosErrorStruct>::Ok(rec)
        });
    }
    let recommendations = futures::future::join_all(recommendations);

    let mut categories = vec![];
    for office_id in &office_ids {
        // Categories
        let q = format!("SELECT * FROM {} o{}", &*CATEGORY_COLLECTION, since);
        let office_id = office_id.clone();
        categories.push(async move {
            let cat: Vec<Category> =
                query(CATEGORY_COLLECTION, [&office_id.as_ref()], q, -1).await?;
            Result::<_, CosmosErrorStruct>::Ok(cat)
        });
    }
    let categories = futures::future::join_all(categories);

    let mut series = vec![];
    for office_id in &office_ids {
        // Series
        let q = format!("SELECT * FROM {} o{}", &*SERIES_COLLECTION, since);
        let office_id = office_id.clone();
        series.push(async move {
            let ser: Vec<Series> = query(SERIES_COLLECTION, [&office_id.as_ref()], q, -1).await?;
            Result::<_, CosmosErrorStruct>::Ok(ser)
        });
    }
    let series = futures::future::join_all(series);

    let mut episodes = vec![];
    for office_id in &office_ids {
        // Episodes
        let q = format!("SELECT * FROM {} o{}", &*EPISODE_COLLECTION, since);
        let office_id = office_id.clone();
        episodes.push(async move {
            let epi: Vec<Episode> = query(EPISODE_COLLECTION, [&office_id.as_ref()], q, -1).await?;
            Result::<_, CosmosErrorStruct>::Ok(epi)
        });
    }
    let episodes = futures::future::join_all(episodes);

    // Episode metadata
    let q = format!("SELECT * FROM {} o{}", &*EPISODE_METADATA_COLLECTION, since);
    let mov = user_id.clone();
    let episode_metadata = async move {
        let epi: Vec<EpisodeMetadata> =
            query(EPISODE_METADATA_COLLECTION, [&mov.as_ref()], q, -1).await?;
        Result::<_, CosmosErrorStruct>::Ok(epi)
    };

    // Series user data
    let q = format!("SELECT * FROM {} o{}", &*SERIES_USER_DATA_COLLECTION, since);
    let mov = user_id.clone();
    let series_user_data = async move {
        let sud: Vec<SeriesUserData> =
            query(SERIES_USER_DATA_COLLECTION, [&mov.as_ref()], q, -1).await?;
        Result::<_, CosmosErrorStruct>::Ok(sud)
    };

    // Subscriptions
    let q = format!("SELECT * FROM {} o{}", &*SUBSCRIPTION_COLLECTION, since);
    let mov = user_id.clone();
    let subscriptions = async move {
        let sub: Vec<Subscription> = query(SUBSCRIPTION_COLLECTION, [&mov.as_ref()], q, -1).await?;
        Result::<_, CosmosErrorStruct>::Ok(sub)
    };

    let (
        offices_r,
        recommendations_r,
        categories_r,
        series_r,
        episodes_r,
        episode_metadata_r,
        series_user_data_r,
        subscriptions_r,
    ) = tokio::join!(
        offices,
        recommendations,
        categories,
        series,
        episodes,
        episode_metadata,
        series_user_data,
        subscriptions
    );

    let mut offices: Vec<Office> = vec![];
    for office in offices_r {
        let office = office?;
        if let Some(office) = office {
            offices.push(office);
        }
    }
    let mut recommendations: Vec<Recommendation> = vec![];
    for recommendation in recommendations_r {
        recommendations.extend(recommendation?);
    }
    let mut categories: Vec<Category> = vec![];
    for category in categories_r {
        categories.extend(category?);
    }
    let mut series: Vec<Series> = vec![];
    for serie in series_r {
        series.extend(serie?);
    }
    let mut episodes: Vec<Episode> = vec![];
    for episode in episodes_r {
        episodes.extend(episode?);
    }
    let episode_metadata = episode_metadata_r?;
    let series_user_data = series_user_data_r?;
    let mut subscriptions = subscriptions_r?;

    // FIXME(J): This is hiding the payments from the user, we need this temporary fix for launch,
    // but we should make the app code be able to handle getting the payment array as soon as
    // possible
    for sub in &mut subscriptions {
        sub.payments = vec![];
    }

    for episode in &mut episodes {
        let mut has_sub = false;
        for sub in &subscriptions {
            if sub.office_id == episode.office_id {
                has_sub = true;
                break;
            }
        }
        // TODO: if !has_sub set sound_file to None
        if false {
            episode.sound_file = None;
        }
    }

    let res = match serde_json::to_string(&DataResponse {
        data: Some(&UserPollDataResponse {
            user,
            offices,
            recommendations,
            categories,
            series,
            episodes,
            episode_metadata,
            series_user_data,
            subscriptions,
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
