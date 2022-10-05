use crate::models::RoleFlags;
use crate::util;
use crate::RECOMMENDED_COLLECTION;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use third_pact::model;

#[model(
    Collection(RECOMMENDED_COLLECTION),
    GET(),
    POST(RoleFlags::OFFICE_CONTENT_ADMIN),
    PUT(RoleFlags::OFFICE_CONTENT_ADMIN),
    DELETE(RoleFlags::OFFICE_CONTENT_ADMIN)
)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    #[prim]
    #[serde(default)]
    pub id: String,

    #[partition]
    pub office_id: String,

    // Series ID
    pub highlighted: String,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,
}
