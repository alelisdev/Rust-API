use crate::models::{I18nString, RoleFlags};
use crate::util;
use crate::EPISODE_COLLECTION;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use third_pact::model;

#[model(
    Collection(EPISODE_COLLECTION),
    PUT(RoleFlags::OFFICE_CONTENT_ADMIN),
    DELETE(RoleFlags::OFFICE_CONTENT_ADMIN)
)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    #[prim]
    #[serde(default)]
    pub id: String,

    #[partition]
    pub office_id: String,

    pub series_id: String,

    pub title: Vec<I18nString>,

    pub text: Vec<I18nString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub sound_file: Option<String>,

    #[serde(default)]
    pub total_duration: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub views: usize,

    #[serde(default)]
    pub likes: usize,

    #[serde(default)]
    pub dislikes: usize,

    #[serde(skip_serializing_if = "util::is_empty")]
    #[serde(default)]
    pub images: Vec<String>,

    #[serde(default = "Utc::now")]
    pub published: DateTime<Utc>,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub created: DateTime<Utc>,
}
