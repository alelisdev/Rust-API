use crate::{error::Error, Gateway, Platform, ProductType, Purchase};

impl Gateway {
    pub async fn get_purchase(
        &self,
        token: String,
        product_id: Option<String>,
        package_name: Option<String>,
        r#type: ProductType,
        test: bool,
        platform: Platform,
    ) -> Result<Purchase, Error> {
        match platform {
            Platform::Apple => match self.verify_apple_receipt(token, test).await {
                Ok(receipt) => {
                    // TODO: Check whether the result is actually the right type? For now we make
                    // sure the product id match, to ensure the type is right.
                    if let Some(product_id) = product_id {
                        if product_id != receipt.product_id {
                            return Err(Error::UnexpectedProductId(format!(
                                "Unexpected product id ({} != {}).",
                                product_id, receipt.product_id
                            )));
                        }
                    }

                    match r#type {
                        ProductType::Subscription => Ok(Purchase::AppleSubscription {
                            product_id: receipt.product_id,
                            transaction_id: receipt.transaction_id,
                            original_transaction_id: receipt.original_transaction_id,
                        }),
                        ProductType::Consumable => Ok(Purchase::AppleConsumable {
                            product_id: receipt.product_id,
                            transaction_id: receipt.transaction_id,
                        }),
                        ProductType::NonConsumable => Ok(Purchase::AppleNonConsumable {
                            product_id: receipt.product_id,
                            transaction_id: receipt.transaction_id,
                        }),
                    }
                }
                Err(err) => Err(err),
            },
            Platform::Google => {
                let product_id = match product_id {
                    Some(product_id) => product_id,
                    None => {
                        return Err(Error::ParseError(format!(
                            "Product id must be present for google purchases."
                        )));
                    }
                };
                let package_name = match package_name {
                    Some(package_name) => package_name,
                    None => {
                        return Err(Error::ParseError(format!(
                            "Package name must be present for google purchases."
                        )));
                    }
                };
                match r#type {
                    ProductType::Subscription => {
                        match self
                            .get_google_subscription(&token, &product_id, &package_name, test)
                            .await
                        {
                            Ok(subscription) => Ok(Purchase::GoogleSubscription {
                                product_id,
                                token,
                                package_name,
                                order_id: subscription.order_id,
                            }),
                            Err(err) => Err(err),
                        }
                    }
                    ProductType::Consumable => {
                        match self
                            .get_google_product(&token, &product_id, &package_name, test)
                            .await
                        {
                            Ok(product) => {
                                // TODO: Check whether the result is actually a consumable?
                                // For now we make sure the product id match, to ensure the
                                // type is right.
                                if product_id == product.product_id {
                                    return Err(Error::UnexpectedProductId(format!(
                                        "Unexpected product id ({} != {}).",
                                        product_id, product.product_id
                                    )));
                                }

                                Ok(Purchase::GoogleConsumable {
                                    product_id,
                                    token,
                                    package_name,
                                    order_id: product.order_id,
                                })
                            }
                            Err(err) => Err(err),
                        }
                    }
                    ProductType::NonConsumable => {
                        match self
                            .get_google_product(&token, &product_id, &package_name, test)
                            .await
                        {
                            Ok(product) => {
                                // TODO: Check whether the result is actually a non-consumable?
                                // For now we make sure the product id match, to ensure the
                                // type is right.
                                if product_id == product.product_id {
                                    return Err(Error::UnexpectedProductId(format!(
                                        "Unexpected product id ({} != {}).",
                                        product_id, product.product_id
                                    )));
                                }

                                Ok(Purchase::GoogleNonConsumable {
                                    product_id,
                                    token,
                                    package_name,
                                    order_id: product.order_id,
                                })
                            }
                            Err(err) => Err(err),
                        }
                    }
                }
            }
        }
    }
}
