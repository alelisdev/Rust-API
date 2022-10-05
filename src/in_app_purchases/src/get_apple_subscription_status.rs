use crate::{
    error::Error, AppleSubscriptionStatus, Gateway, Platform, BASE_STOREKIT_URL_LIVE,
    BASE_STOREKIT_URL_PLAY,
};
use serde::Deserialize;

impl Gateway {
    // cf.
    // https://developer.apple.com/documentation/appstoreserverapi/get_all_subscription_statuses
    pub async fn get_apple_subscription_status(
        &self,
        original_transaction_id: &str,
        test: bool,
    ) -> Result<AppleSubscriptionStatus, Error> {
        let base = match test {
            false => BASE_STOREKIT_URL_LIVE,
            true => BASE_STOREKIT_URL_PLAY,
        };

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct LastTransactionsItem {
            pub original_transaction_id: String,
            pub status: AppleSubscriptionStatus,
            // pub signed_transaction_info: String,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct SubscriptionGroupIdentifierItem {
            //pub subscription_group_identifier: String,
            pub last_transactions: Vec<LastTransactionsItem>,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            //pub environment: Environment,
            pub data: Vec<SubscriptionGroupIdentifierItem>,
        }

        let url = format!("{}/subscriptions/{}", base, original_transaction_id);
        let res: Response = self.get(&url, Platform::Apple).await?;

        for data in &res.data {
            for transaction in &data.last_transactions {
                if transaction.original_transaction_id == original_transaction_id {
                    return Ok(transaction.status);
                }
            }
        }

        Err(Error::SubscriptionNotFound)
    }
}
