use crate::models::RoleFlags;
use crate::util;
use crate::EPISODE_METADATA_COLLECTION;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use third_pact::model;

#[model(
    Collection(EPISODE_METADATA_COLLECTION),
    GET(SELF, user_id),
    DELETE(RoleFlags::OFFICE_CONTENT_ADMIN)
)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeMetadata {
    #[prim]
    #[serde(default)]
    pub id: String,

    #[partition]
    pub user_id: String,

    pub office_id: String,

    pub series_id: String,

    pub episode_id: String,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub favourite: bool,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub finished: bool,

    #[serde(default)]
    pub times_listened: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub current_time_secs: Option<Duration>,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,
}
