use crate::{error::Error, Gateway, GoogleProductPurchase, Platform, BASE_ANDROID_URL};

impl Gateway {
    // cf.
    // https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.products/get
    pub async fn get_google_product(
        &self,
        token: &str,
        product_id: &str,
        package_name: &str,
        _test: bool,
    ) -> Result<GoogleProductPurchase, Error> {
        let url = format!(
            "{}/applications/{}/purchases/products/{}/tokens/{}",
            BASE_ANDROID_URL, package_name, product_id, token
        );
        let res: GoogleProductPurchase = self.get(&url, Platform::Google).await?;
        Ok(res)
    }
}
