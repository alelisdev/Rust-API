use crate::util;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Payment {
    #[serde(rename_all = "camelCase")]
    AppleInAppSubscriptionPurchase {
        from: DateTime<Utc>,

        #[serde(skip_serializing_if = "util::is_none")]
        #[serde(default)]
        to: Option<DateTime<Utc>>,

        original_transaction_id: String,

        original_purchase_date: DateTime<Utc>,

        product_id: String,

        #[serde(default = "Utc::now")]
        modified: DateTime<Utc>,
    },

    #[serde(rename_all = "camelCase")]
    GoogleInAppSubscriptionPurchase {
        from: DateTime<Utc>,

        #[serde(skip_serializing_if = "util::is_none")]
        #[serde(default)]
        to: Option<DateTime<Utc>>,

        token: String,

        package_name: String,

        original_purchase_date: DateTime<Utc>,

        product_id: String,

        #[serde(default = "Utc::now")]
        modified: DateTime<Utc>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    #[serde(default = "util::new_guid_v4")]
    pub id: String,

    #[serde(skip_serializing_if = "util::is_false")]
    #[serde(default)]
    pub deleted: bool,

    // Denormalized for db queries.
    pub office_id: String,

    pub user_id: String,

    #[serde(default = "Utc::now")]
    pub start: DateTime<Utc>,

    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub end: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "util::is_empty")]
    #[serde(default)]
    pub payments: Vec<Payment>,

    #[serde(default = "Utc::now")]
    pub created: DateTime<Utc>,

    #[serde(default = "Utc::now")]
    pub modified: DateTime<Utc>,
}
