use crate::models::{I18nString, RoleFlags};
use crate::util;
use crate::OFFICE_COLLECTION;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use third_pact::model;

#[model(
    Collection(OFFICE_COLLECTION),
    GET(),
    POST(RoleFlags::GLOBAL_CONTENT_ADMIN),
    PUT(RoleFlags::OFFICE_CONTENT_ADMIN, id),
    DELETE(RoleFlags::OFFICE_CONTENT_ADMIN, id)
)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Office {
    #[prim]
    #[partition]
    #[serde(default)]
    pub id: String,

    pub titles: Vec<I18nString>,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,
}
