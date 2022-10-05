use crate::{
    fault::Fault,
    models::{Payment, Subscription},
    util::{DataResponse, Empty},
    APPLICATION_INSIGHTS_TELEMETRY_CLIENT, PRODUCTION_ENVIRONMENT, SUBSCRIPTION_COLLECTION,
};
use appinsights::telemetry::SeverityLevel;
use cosmos_utils::{query_crosspartition, upsert};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use warp::reject;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub signed_payload: String,
}

pub async fn webhook_subscription_apple(
    webhook: Webhook,
) -> Result<impl warp::Reply, warp::Rejection> {
    // cf. https://developer.apple.com/documentation/appstoreservernotifications/notificationtype
    #[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum NotificationType {
        ConsumptionRequest,
        DidChangeRenewalPref,
        DidChangeRenewalStatus,
        DidFailToRenew,
        DidRenew,
        Expired,
        GracePeriodExpired,
        OfferRedeemed,
        PriceIncrease,
        Refund,
        RefundDeclined,
        RenewalExtended,
        Revoke,
        Subscribed,
    }

    // cf. https://developer.apple.com/documentation/appstoreservernotifications/environment
    #[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
    enum Environment {
        Sandbox,
        Production,
    }

    let parts = webhook.signed_payload.split('.').collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(reject::custom(Fault::IllegalArgument(format!(
            "Wrong length of parts ({}).",
            parts.len()
        ))));
    }

    let payload = match base64::decode(parts[1]) {
        Ok(b) => match String::from_utf8(b.to_vec()) {
            Ok(g) => g,
            Err(err) => {
                return Err(reject::custom(Fault::IllegalArgument(format!(
                    "Could not decode webhook payload, getting string from decoded base 64 {} \
                    ({}).",
                    parts[1],
                    err.to_string()
                ))));
            }
        },
        Err(err) => {
            return Err(reject::custom(Fault::IllegalArgument(format!(
                "Could not decode webhook payload from base 64 {} ({}).",
                parts[1],
                err.to_string()
            ))));
        }
    };

    // cf. https://developer.apple.com/documentation/appstoreservernotifications/data
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PayloadData {
        // pub bundle_id: String,
        // pub bundle_version: String,
        pub environment: Environment,
        pub signed_transaction_info: String,
        // #[serde(default)]
        // pub signed_renewal_info: Option<String>,
    }

    // cf. https://developer.apple.com/documentation/appstoreservernotifications/responsebodyv2decodedpayload
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
        pub notification_type: NotificationType,
        // #[serde(rename = "notificationUUID")]
        // pub notification_uuid: String,
        pub data: PayloadData,
        // pub version: String,
    }

    // Parse.
    let payload: Payload = match serde_json::from_str(&payload) {
        Ok(r) => r,
        Err(err) => {
            return Err(reject::custom(Fault::IllegalArgument(format!(
                "Could not deserialize payload ({}) from {}.",
                err.to_string(),
                &payload,
            ))));
        }
    };

    // Is this from the sandbox system? Redirect to play.
    if *PRODUCTION_ENVIRONMENT && payload.data.environment == Environment::Sandbox {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.primecrime.v1+json"),
        );

        let client = match reqwest::ClientBuilder::new()
            .default_headers(headers)
            .https_only(true)
            .timeout(std::time::Duration::new(60, 0))
            .build()
        {
            Ok(r) => r,
            Err(err) => {
                return Err(reject::custom(Fault::IllegalState(format!(
                    "Could not create reqwest client ({}).",
                    err.to_string()
                ))));
            }
        };

        let res = match client
            .post("https://primecrime-api-play.azurewebsites.net/webhooks/subscriptions/apple")
            .json(&webhook)
            .send()
            .await
        {
            Ok(r) => r,
            Err(err) => {
                return Err(reject::custom(Fault::IllegalState(format!(
                    "Could not send request ({}).",
                    err.to_string()
                ))));
            }
        };

        let status = res.status();
        let text = res
            .text()
            .await
            .unwrap_or_else(|_| String::from("Could not retrieve body text."));

        if status != 200 {
            return Err(reject::custom(Fault::Unspecified(format!(
                "Got error {} calling play apple subscriptions webhook ({}).",
                status, &text,
            ))));
        }

        return Ok(warp::reply::json(&DataResponse {
            data: None::<Empty>,
            extra: None::<Empty>,
        }));
    }

    let parts = payload
        .data
        .signed_transaction_info
        .split('.')
        .collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(reject::custom(Fault::IllegalArgument(format!(
            "Wrong length of parts inside payload data signed transaction info ({}).",
            parts.len()
        ))));
    }

    let transaction_data = match base64::decode(parts[1]) {
        Ok(b) => match String::from_utf8(b.to_vec()) {
            Ok(g) => g,
            Err(err) => {
                return Err(reject::custom(Fault::IllegalArgument(format!(
                    "Could not decode webhook payload data, getting string from decoded base 64 \
                    {} ({}).",
                    parts[1],
                    err.to_string()
                ))));
            }
        },
        Err(err) => {
            return Err(reject::custom(Fault::IllegalArgument(format!(
                "Could not decode webhook payload data from base 64 {} ({}).",
                parts[1],
                err.to_string()
            ))));
        }
    };

    // cf. https://developer.apple.com/documentation/appstoreservernotifications/jwstransactiondecodedpayload
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct TransactionData {
        // pub transaction_id: String,
        pub original_transaction_id: String,
        // pub web_order_line_item_id: String,
        // pub bundle_id: String,
        // pub product_id: String,
        // pub subscription_group_identifier: String,
        // pub purchase_date: i64,
        // pub original_purchase_date: i64,
        // pub expires_date: i64,
        // pub quantity: i64,
        // pub r#type: String,
        // pub in_app_ownership_type: String,
        // pub signed_date: i64,
    }

    // Parse.
    let transaction_data: TransactionData = match serde_json::from_str(&transaction_data) {
        Ok(r) => r,
        Err(err) => {
            return Err(reject::custom(Fault::IllegalArgument(format!(
                "Could not deserialize response ({}) from {}.",
                err.to_string(),
                &transaction_data,
            ))));
        }
    };

    if payload.notification_type == NotificationType::Expired {
        // Find subscription.
        let q = format!(
            "SELECT VALUE m FROM {} m JOIN p IN m.payments WHERE p.type = \
            \"AppleInAppSubscriptionPurchase\" AND p.originalTransactionId = \"{}\"",
            SUBSCRIPTION_COLLECTION, &transaction_data.original_transaction_id,
        );
        let subscriptions: Vec<Subscription> =
            query_crosspartition(SUBSCRIPTION_COLLECTION, [()], q, -1, true).await?;

        for subscription in &subscriptions {
            let mut subscription = subscription.clone();

            // Mark subscription as terminated.
            let update = if let None = subscription.end {
                let now = chrono::Utc::now();

                subscription.end = Some(now);
                subscription.modified = now;

                // Set payment to.
                let last = subscription.payments.len() - 1;
                match subscription.payments[last] {
                    Payment::AppleInAppSubscriptionPurchase { ref mut to, .. } => {
                        *to = Some(now);
                    }
                    Payment::GoogleInAppSubscriptionPurchase { .. } => {
                        return Err(reject::custom(Fault::IllegalState(format!(
                            "Found an incompatible payment type for apple expired webhook \
                            in subscription {} for user {}.",
                            subscription.id, subscription.user_id,
                        ))));
                    }
                };

                true
            } else {
                false
            };

            if update {
                upsert(
                    SUBSCRIPTION_COLLECTION,
                    [&subscription.user_id],
                    &subscription,
                    None,
                )
                .await?;
            }
        }

        if subscriptions.len() == 0 {
            // Log.
            APPLICATION_INSIGHTS_TELEMETRY_CLIENT.track_trace(
                format!(
                    "Did not find a matching subscription for apple expired webhook with an \
                    original transaction id of {}.",
                    &transaction_data.original_transaction_id
                ),
                SeverityLevel::Critical,
            );
        } else if subscriptions.len() > 1 {
            // Log.
            APPLICATION_INSIGHTS_TELEMETRY_CLIENT.track_trace(
                format!(
                    "Found several matching subscriptions for apple expired webhook with an \
                    original transaction id of {}.",
                    &transaction_data.original_transaction_id
                ),
                SeverityLevel::Critical,
            );
        }
    }

    Ok(warp::reply::json(&DataResponse {
        data: None::<Empty>,
        extra: None::<Empty>,
    }))
}
