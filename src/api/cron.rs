use crate::{
    fault::Fault,
    models::{Payment, Subscription},
    util::{log_critical, DataRequest, DataResponse, Empty},
    APPLICATION_INSIGHTS_TELEMETRY_CLIENT, CRON_SECRET, IN_APP_PURCHASES_APPLE_BUNDLE_ID,
    IN_APP_PURCHASES_APPLE_ISSUER, IN_APP_PURCHASES_APPLE_KEY, IN_APP_PURCHASES_APPLE_KEY_ID,
    IN_APP_PURCHASES_APPLE_PASSWORD, IN_APP_PURCHASES_GOOGLE_KEY,
    IN_APP_PURCHASES_GOOGLE_SERVICE_ACCOUNT, PRODUCTION_ENVIRONMENT, SUBSCRIPTION_COLLECTION,
};
use appinsights::telemetry::SeverityLevel;
use chrono::{TimeZone, Utc};
use cosmos_utils::{query_crosspartition_etag, upsert};
use warp::reject;

pub async fn cron(
    r: DataRequest<Empty, String>,
    _v: u8,
) -> Result<impl warp::Reply, warp::Rejection> {
    match r.extra {
        Some(extra) => {
            if &extra != &*CRON_SECRET {
                return Err(reject::custom(Fault::Unauthorized));
            }
        }
        None => {
            return Err(reject::custom(Fault::NoExtra));
        }
    };

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

    let now = Utc::now();

    // Get all subscriptions that have not expired
    let q = format!(
        "SELECT * FROM {} s WHERE s['end'] >= \"{}\" OR NOT IS_DEFINED(s['end'])",
        SUBSCRIPTION_COLLECTION,
        now.to_rfc3339(),
    );
    let active_subscriptions: Vec<(Subscription, _)> =
        query_crosspartition_etag(SUBSCRIPTION_COLLECTION, [&()], q, -1, true).await?;

    for (mut sub, etag) in active_subscriptions {
        let payment = match sub.payments.last_mut() {
            Some(payment) => payment,
            None => {
                log_critical(format!(
                    "Cron found a subscription that does not contain a payment ({}).",
                    sub.id
                ));
                continue;
            }
        };
        match payment {
            Payment::AppleInAppSubscriptionPurchase {
                // ref mut to,
                // ref original_transaction_id,
                // ref mut modified,
                ..
            } => {
                continue;

                // let subscription_status = match gateway
                //     .get_apple_subscription_status(&original_transaction_id, !*PRODUCTION_ENVIRONMENT)
                //     .await
                // {
                //     Ok(res) => res,
                //     Err(err) => {
                //         log_critical(format!(
                //             "Error getting subscription status for apple original transaction id \
                //             \"{}\" in cron ({}).",
                //             original_transaction_id,
                //             err.to_string()
                //         ));
                //         continue;
                //     }
                // };
                // match subscription_status {
                //     in_app_purchases::AppleSubscriptionStatus::Expired => {
                //         *to = Some(now);
                //         *modified = now;
                //         sub.end = Some(now);
                //         sub.modified = now;
                //         match upsert(SUBSCRIPTION_COLLECTION, [&sub.user_id], &sub, Some(&etag))
                //             .await
                //         {
                //             Ok(_) => (),
                //             Err(err) => {
                //                 log_critical(format!(
                //                     "Error upserting expired apple subscription in cron ({}).",
                //                     err.to_string()
                //                 ));
                //                 continue;
                //             }
                //         }
                //     }
                //     _ => {
                //         continue;
                //     }
                // };
            }
            Payment::GoogleInAppSubscriptionPurchase {
                ref mut to,
                token,
                package_name,
                product_id,
                ref mut modified,
                ..
            } => {
                // Verify receipt.
                // Subscription ID is the same as product ID
                // https://stackoverflow.com/questions/58530628/google-play-developer-api-get-
                // subscription-what-is-subscriptionid
                let sub_purchase = match gateway
                    .get_google_subscription(
                        &token,
                        product_id,
                        package_name,
                        !*PRODUCTION_ENVIRONMENT,
                    )
                    .await
                {
                    Ok(purchase) => purchase,
                    Err(err) => match err {
                        in_app_purchases::Error::GoogleApiError(code, message) => {
                            log_critical(format!(
                                "Error verifying google receipt in cron (code: {:?}, message: {}).",
                                code, message
                            ));
                            continue;
                        }
                        _ => {
                            log_critical(format!(
                                "Error verifying google receipt in cron ({}).",
                                err.to_string()
                            ));
                            continue;
                        }
                    },
                };
                match sub_purchase.expiry_time_millis {
                    Some(expiry) => {
                        let milis = match expiry.parse::<i64>() {
                            Ok(milis) => milis,
                            Err(err) => {
                                log_critical(format!(
                                    "Error parsing expiry time in cron ({}).",
                                    err.to_string()
                                ));
                                continue;
                            }
                        };
                        let expiry = Utc.timestamp_millis(milis);
                        if expiry < now {
                            *to = Some(expiry);
                            *modified = now;
                            sub.end = Some(expiry);
                            sub.modified = now;
                            match upsert(SUBSCRIPTION_COLLECTION, [&sub.user_id], &sub, Some(&etag))
                                .await
                            {
                                Ok(_) => (),
                                Err(err) => {
                                    log_critical(format!(
                                        "Error upserting expired google subscription in cron ({}).",
                                        err.to_string()
                                    ));
                                    continue;
                                }
                            }
                        }
                    }
                    None => {
                        continue;
                    }
                };
            }
        }
    }

    Ok(warp::reply::json(&DataResponse {
        data: None::<Empty>,
        extra: None::<Empty>,
    }))
}
