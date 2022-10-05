use crate::{
    fault::Fault,
    models::{Claims, Payment, Subscription, User},
    util::{DataRequest, DataResponse, Empty},
    APPLICATION_INSIGHTS_TELEMETRY_CLIENT, IN_APP_PURCHASES_APPLE_BUNDLE_ID,
    IN_APP_PURCHASES_APPLE_ISSUER, IN_APP_PURCHASES_APPLE_KEY, IN_APP_PURCHASES_APPLE_KEY_ID,
    IN_APP_PURCHASES_APPLE_PASSWORD, IN_APP_PURCHASES_GOOGLE_KEY,
    IN_APP_PURCHASES_GOOGLE_SERVICE_ACCOUNT, SUBSCRIPTION_COLLECTION, USER_COLLECTION,
};
use appinsights::telemetry::SeverityLevel;
use chrono::Utc;
use cosmos_utils::{get, insert, query};
use in_app_purchases::Purchase as InAppPurchase;
use serde::Deserialize;
use warp::reject;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraRequest {
    pub platform: in_app_purchases::Platform,

    pub office_id: String,

    // Used by android.
    #[serde(default)]
    pub product_id: Option<String>,

    // Used by android.
    #[serde(default)]
    pub package_name: Option<String>,
}

pub async fn subscription_post(
    user_id: String,
    r: DataRequest<String, ExtraRequest>,
    _claims: Claims,
    _v: u8,
) -> Result<impl warp::Reply, warp::Rejection> {
    let receipt;
    if let Some(q) = r.data {
        receipt = q;
    } else {
        return Err(reject::custom(Fault::NoData));
    }

    let (platform, office_id, product_id, package_name) = match r.extra {
        Some(extra) => (
            extra.platform,
            extra.office_id,
            extra.product_id,
            extra.package_name,
        ),
        None => {
            return Err(reject::custom(Fault::NoExtra));
        }
    };

    let (user, _): (User, _) = get(USER_COLLECTION, [&user_id], &user_id).await?;

    let gateway = match in_app_purchases::Gateway::new(
        IN_APP_PURCHASES_APPLE_BUNDLE_ID.to_string(),
        IN_APP_PURCHASES_APPLE_KEY_ID.to_string(),
        IN_APP_PURCHASES_APPLE_KEY.to_string(),
        IN_APP_PURCHASES_APPLE_PASSWORD.to_string(),
        IN_APP_PURCHASES_APPLE_ISSUER.to_string(),
        IN_APP_PURCHASES_GOOGLE_SERVICE_ACCOUNT.to_string(),
        IN_APP_PURCHASES_GOOGLE_KEY.to_string(),
        None,
    )
    .await
    {
        Ok(client) => client,
        Err(err) => {
            APPLICATION_INSIGHTS_TELEMETRY_CLIENT.track_trace(
                format!(
                    "Error making in-app-purchases gateway ({}).",
                    err.to_string()
                ),
                SeverityLevel::Critical,
            );

            return Err(reject::custom(Fault::Unspecified(format!(
                "Error making in-app-purchases gateway ({}).",
                err.to_string()
            ))));
        }
    };

    // Verify receipt.
    let purchase = match gateway
        .get_purchase(
            receipt.clone(),
            product_id,
            package_name,
            in_app_purchases::ProductType::Subscription,
            user.test,
            platform,
        )
        .await
    {
        Ok(purchase) => purchase,
        Err(err) => match err {
            in_app_purchases::Error::AppleApiError(code, message) => {
                return Err(reject::custom(Fault::IllegalArgument(format!(
                    "Error verifying receipt (code: {:?}, message: {}).",
                    code, message
                ))));
            }
            in_app_purchases::Error::GoogleApiError(code, message) => {
                return Err(reject::custom(Fault::IllegalArgument(format!(
                    "Error verifying receipt (code: {:?}, message: {}).",
                    code, message
                ))));
            }
            in_app_purchases::Error::InvalidAppleReceipt(_) => {
                return Err(reject::custom(Fault::IllegalArgument(format!(
                    "Invalid receipt ({}).",
                    receipt
                ))));
            }
            _ => {
                return Err(reject::custom(Fault::Unspecified(format!(
                    "Error verifying receipt ({})",
                    err.to_string()
                ))));
            }
        },
    };

    // Make sure no active subscription is already present for this receipt.
    let q = format!(
        "SELECT * FROM {} o WHERE o.userId = \"{}\"",
        SUBSCRIPTION_COLLECTION, user_id
    );
    let subscriptions: Vec<Subscription> =
        query(SUBSCRIPTION_COLLECTION, [&user_id], q, -1).await?;

    let now = Utc::now();

    for subscription in subscriptions {
        let eligable = match subscription.end {
            Some(end) => end > now,
            None => true,
        };

        if eligable {
            for payment in subscription.payments {
                match (&payment, &purchase) {
                    (
                        Payment::AppleInAppSubscriptionPurchase {
                            original_transaction_id: ref payment_original_transaction_id,
                            ..
                        },
                        InAppPurchase::AppleSubscription {
                            original_transaction_id: ref purchase_original_transaction_id,
                            ..
                        },
                    ) => {
                        if payment_original_transaction_id == purchase_original_transaction_id {
                            return Err(reject::custom(Fault::Duplicate(format!(
                                "An active subscription with apple transaction id {} \
                                already exist.",
                                payment_original_transaction_id
                            ))));
                        }
                    }
                    (
                        Payment::GoogleInAppSubscriptionPurchase {
                            token: ref payment_token,
                            ..
                        },
                        InAppPurchase::GoogleSubscription {
                            token: ref purchase_token,
                            ..
                        },
                    ) => {
                        if payment_token == purchase_token {
                            return Err(reject::custom(Fault::Duplicate(format!(
                                "An active subscription with google token {} already exist.",
                                payment_token
                            ))));
                        }
                    }
                    _ => {}
                };
            }
        }
    }

    let payment = match purchase {
        InAppPurchase::AppleSubscription {
            product_id,
            original_transaction_id,
            ..
        } => Payment::AppleInAppSubscriptionPurchase {
            from: now,
            to: None,
            original_transaction_id,
            original_purchase_date: now,
            product_id,
            modified: now,
        },
        InAppPurchase::GoogleSubscription {
            token,
            package_name,
            product_id,
            ..
        } => Payment::GoogleInAppSubscriptionPurchase {
            from: now,
            to: None,
            token,
            package_name,
            original_purchase_date: now,
            product_id,
            modified: now,
        },
        _ => {
            return Err(reject::custom(Fault::IllegalState(format!(
                "Subscription purchase is not a apple or google subscription"
            ))));
        }
    };

    // Make subscription.
    let mut subscription = Subscription {
        id: uuid::Uuid::new_v4().to_string(),
        deleted: false,
        office_id,
        user_id: user_id.clone(),
        start: now,
        end: None,
        payments: vec![payment],
        created: now,
        modified: now,
    };

    insert(SUBSCRIPTION_COLLECTION, [&user_id], &subscription, None).await?;

    // FIXME(J): This is hiding the payments from the user, we need this temporary fix for launch,
    // but we should make the app code be able to handle getting the payment array as soon as
    // possible
    subscription.payments = vec![];

    Ok(warp::reply::json(&DataResponse {
        data: Some(&subscription),
        extra: None::<Empty>,
    }))
}
