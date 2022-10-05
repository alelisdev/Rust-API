use crate::util;
use serde::{Deserialize, Serialize};

// cf. https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.subscriptions#SubscriptionPurchase
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSubscriptionPurchase {
    pub start_time_millis: String,

    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub expiry_time_millis: Option<String>,

    pub auto_renewing: bool,

    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub cancel_reason: Option<i32>,

    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub developer_payload: Option<String>,

    pub order_id: String,
}
