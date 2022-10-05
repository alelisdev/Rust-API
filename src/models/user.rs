use crate::{models::Device, util, Role};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(default = "util::new_guid_v4")]
    pub id: String,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub test: bool,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub roles: Vec<Role>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub devices: Vec<Device>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub images: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub preferred_name: Option<String>,

    pub last_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub middle_names: Option<String>,

    pub first_name: String,

    #[serde(default)]
    pub office_ids: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub saved_series: Vec<String>,

    pub email: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub phone: Option<String>,

    // TODO: Should we perhaps have this in the episode metadata?
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub favourite_episode_ids: Vec<String>,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub created: DateTime<Utc>,
}
