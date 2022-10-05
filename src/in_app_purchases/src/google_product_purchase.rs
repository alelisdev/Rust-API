use crate::util;
use serde::{Deserialize, Serialize};

// cf. https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.products
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GoogleProductPurchase {
    pub product_id: String,

    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub developer_payload: Option<String>,

    pub order_id: String,
}
