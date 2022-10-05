use crate::{
    error::Error, util, AppleInAppReceipt, AppleReceiptStatus, Gateway, Platform,
    BASE_ITUNES_URL_LIVE, BASE_ITUNES_URL_PLAY,
};
use serde::{Deserialize, Serialize};

impl Gateway {
    // cf. https://developer.apple.com/documentation/appstorereceipts/verifyreceipt
    pub async fn verify_apple_receipt(
        &self,
        receipt: String,
        test: bool,
    ) -> Result<AppleInAppReceipt, Error> {
        let base = match test {
            false => BASE_ITUNES_URL_LIVE,
            true => BASE_ITUNES_URL_PLAY,
        };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request {
            #[serde(rename = "receipt-data")]
            pub receipt_data: String,

            pub password: String,

            #[serde(skip_serializing_if = "util::is_false")]
            #[serde(rename = "exclude-old-transactions")]
            pub exclude_old_transactions: bool,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            //pub environment: Environment,
            #[serde(rename = "is-retryable")]
            #[serde(default)]
            #[allow(dead_code)]
            pub is_retryable: bool,

            #[serde(default)]
            #[allow(dead_code)]
            pub latest_receipt: Option<String>,

            // latest_receipt_info

            // pending_renewal_info
            #[serde(default)]
            pub receipt: Option<AppleInAppReceipt>,

            pub status: AppleReceiptStatus,
        }

        let request = Request {
            receipt_data: receipt,
            password: self.apple_password.clone(),
            exclude_old_transactions: false,
        };

        let url = format!("{}/verifyReceipt", base);
        let res: Response = self.post(&url, &request, Platform::Apple).await?;

        if res.status != AppleReceiptStatus::Valid {
            return Err(Error::InvalidAppleReceipt(format!("Receipt is not valid.")));
        }

        let receipt = match res.receipt {
            Some(receipt) => receipt,
            None => return Err(Error::ParseError(format!("No receipt received."))),
        };

        Ok(receipt)
    }
}
