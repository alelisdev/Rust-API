use crate::{error::Error, Gateway, GoogleSubscriptionPurchase, Platform, BASE_ANDROID_URL};

impl Gateway {
    // cf.
    // https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.subscriptions/get
    pub async fn get_google_subscription(
        &self,
        token: &str,
        subscription_id: &str,
        package_name: &str,
        _test: bool,
    ) -> Result<GoogleSubscriptionPurchase, Error> {
        let url = format!(
            "{}/applications/{}/purchases/subscriptions/{}/tokens/{}",
            BASE_ANDROID_URL, package_name, subscription_id, token
        );
        let res: GoogleSubscriptionPurchase = self.get(&url, Platform::Google).await?;
        Ok(res)
    }
}
