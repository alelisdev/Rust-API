use crate::models::{I18nString, RoleFlags};
use crate::util;
use crate::CATEGORY_COLLECTION;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use third_pact::model;

#[model(
    Collection(CATEGORY_COLLECTION),
    GET(),
    POST(RoleFlags::OFFICE_CONTENT_ADMIN),
    PUT(RoleFlags::OFFICE_CONTENT_ADMIN),
    DELETE(RoleFlags::OFFICE_CONTENT_ADMIN)
)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    #[prim]
    #[serde(default)]
    pub id: String,

    #[partition]
    pub office_id: String,

    pub title: Vec<I18nString>,

    // TODO(J): We probably want an enum for the tags
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,
}
