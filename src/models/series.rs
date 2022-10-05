use crate::models::{I18nString, RoleFlags};
use crate::util;
use crate::SERIES_COLLECTION;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use third_pact::model;

#[model(
    Collection(SERIES_COLLECTION),
    GET(),
    POST(RoleFlags::OFFICE_CONTENT_ADMIN, office_id),
    PUT(RoleFlags::OFFICE_CONTENT_ADMIN, office_id),
    DELETE(RoleFlags::OFFICE_CONTENT_ADMIN, office_id)
)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    #[prim]
    #[serde(default)]
    pub id: String,

    #[partition]
    pub office_id: String,

    pub title: Vec<I18nString>,

    pub text: Vec<I18nString>,

    pub category_ids: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub images: Vec<String>,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub created: DateTime<Utc>,
}
