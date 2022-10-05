use crate::models::RoleFlags;
use crate::util;
use crate::SERIES_USER_DATA_COLLECTION;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use third_pact::model;

#[model(
    Collection(SERIES_USER_DATA_COLLECTION),
    GET(SELF, user_id),
    PUT(SELF, user_id),
    DELETE(RoleFlags::OFFICE_CONTENT_ADMIN)
)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SeriesUserData {
    #[prim]
    #[serde(default)]
    pub id: String,

    #[partition]
    pub user_id: String,

    pub office_id: String,

    pub series_id: String,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub favourite: bool,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,
}
