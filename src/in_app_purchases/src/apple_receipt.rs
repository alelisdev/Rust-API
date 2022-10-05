use crate::util; //AppleReceiptType
use serde::{Deserialize, Serialize};

// cf. https://developer.apple.com/documentation/appstorereceipts/responsebody/receipt/in_app
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct AppleInAppReceipt {
    // The time the App Store refunded a transaction or revoked it from family sharing, in a date-
    // time format similar to the ISO 8601. This field is present only for refunded or revoked
    // transactions.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub cancellation_date: Option<String>,

    // The time the App Store refunded a transaction or revoked it from family sharing, in UNIX
    // epoch time format, in milliseconds. This field is present only for refunded or revoked
    // transactions. Use this time format for processing dates.
    // cf. https://developer.apple.com/documentation/appstorereceipts/cancellation_date_ms
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub cancellation_date_ms: Option<String>,

    // The time the App Store refunded a transaction or revoked it from family sharing, in the
    // Pacific Time zone. This field is present only for refunded or revoked transactions.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub cancellation_date_pst: Option<String>,

    // The reason for a refunded or revoked transaction. A value of “1” indicates that the customer
    // canceled their transaction due to an actual or perceived issue within your app. A value of
    // “0” indicates that the transaction was canceled for another reason; for example, if the
    // customer made the purchase accidentally.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub cancellation_reason: Option<String>,

    // The time a subscription expires or when it will renew, in a date-time format similar to the
    // ISO 8601.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub expires_date: Option<String>,

    // The time a subscription expires or when it will renew, in UNIX epoch time format, in
    // milliseconds. Use this time format for processing dates.
    // cf. https://developer.apple.com/documentation/appstorereceipts/expires_date_ms
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub expires_date_ms: Option<String>,

    // The time a subscription expires or when it will renew, in the Pacific Time zone.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub expires_date_pst: Option<String>,

    // An indicator of whether an auto-renewable subscription is in the introductory price period.
    // cf. https://developer.apple.com/documentation/appstorereceipts/is_in_intro_offer_period
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub is_in_intro_offer_period: Option<String>,

    // An indication of whether a subscription is in the free trial period.
    // cf. https://developer.apple.com/documentation/appstorereceipts/is_trial_period
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub is_trial_period: Option<String>,

    // The time of the original in-app purchase, in a date-time format similar to ISO 8601.
    pub original_purchase_date: String,

    // The time of the original in-app purchase, in UNIX epoch time format, in milliseconds. For an
    // auto-renewable subscription, this value indicates the date of the subscription's initial
    // purchase. The original purchase date applies to all product types and remains the same in
    // all transactions for the same product ID. This value corresponds to the original
    // transaction’s transactionDate property in StoreKit. Use this time format for processing
    // dates.
    pub original_purchase_date_ms: String,

    // The time of the original in-app purchase, in the Pacific Time zone.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub original_purchase_date_pst: Option<String>,

    // The transaction identifier of the original purchase.
    // cf. https://developer.apple.com/documentation/appstorereceipts/original_transaction_id
    pub original_transaction_id: String,

    // The unique identifier of the product purchased. You provide this value when creating the
    // product in App Store Connect, and it corresponds to the productIdentifier property of the
    // SKPayment object stored in the transaction's payment property.
    pub product_id: String,

    // The identifier of the subscription offer redeemed by the user.
    // cf. https://developer.apple.com/documentation/appstorereceipts/promotional_offer_id
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub promotional_offer_id: Option<String>,

    // The time the App Store charged the user's account for a purchased or restored product, or
    // the time the App Store charged the user’s account for a subscription purchase or renewal
    // after a lapse, in a date-time format similar to ISO 8601.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub purchase_date: Option<String>,

    // For consumable, non-consumable, and non-renewing subscription products, the time the App
    // Store charged the user's account for a purchased or restored product, in the UNIX epoch time
    // format, in milliseconds. For auto-renewable subscriptions, the time the App Store charged
    // the user’s account for a subscription purchase or renewal after a lapse, in the UNIX epoch
    // time format, in milliseconds. Use this time format for processing dates.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub purchase_date_ms: Option<String>,

    // The time the App Store charged the user's account for a purchased or restored product, or
    // the time the App Store charged the user’s account for a subscription purchase or renewal
    // after a lapse, in the Pacific Time zone.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub purchase_date_pst: Option<String>,

    // The number of consumable products purchased. This value corresponds to the quantity property
    // of the SKPayment object stored in the transaction's payment property. The value is usually
    // “1” unless modified with a mutable payment. The maximum value is 10.
    pub quantity: String,

    // A unique identifier for a transaction such as a purchase, restore, or renewal.
    // cf. https://developer.apple.com/documentation/appstorereceipts/transaction_id
    pub transaction_id: String,

    // A unique identifier for purchase events across devices, including subscription-renewal
    // events. This value is the primary key for identifying subscription purchases.
    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub web_order_line_item_id: Option<String>,
}

// // cf. https://developer.apple.com/documentation/appstorereceipts/responsebody/receipt
// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(rename_all = "snake_case")]
// pub struct AppleReceipt {
//     // See app_item_id.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub adam_id: Option<i64>,

//     // Generated by App Store Connect and used by the App Store to uniquely identify the app
//     // purchased. Apps are assigned this identifier only in production. Treat this value as a 64-
//     // bit long integer.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub app_item_id: Option<i64>,

//     // The app’s version number. The app's version number corresponds to the value of
//     // CFBundleVersion (in iOS) or CFBundleShortVersionString (in macOS) in the Info.plist. In
//     // production, this value is the current version of the app on the device based on the
//     // receipt_creation_date_ms. In the sandbox, the value is always "1.0".
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub application_version: Option<String>,

//     // The bundle identifier for the app to which the receipt belongs. You provide this string on
//     // App Store Connect. This corresponds to the value of CFBundleIdentifier in the Info.plist
//     // file of the app.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub bundle_id: Option<String>,

//     // A unique identifier for the app download transaction.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub download_id: Option<i32>,

//     // The time the receipt expires for apps purchased through the Volume Purchase Program, in a
//     // date-time format similar to the ISO 8601.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub expiration_date: Option<String>,

//     // The time the receipt expires for apps purchased through the Volume Purchase Program, in UNIX
//     // epoch time format, in milliseconds. If this key is not present for apps purchased through
//     // the Volume Purchase Program, the receipt does not expire. Use this time format for
//     // processing dates.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub expiration_date_ms: Option<String>,

//     // The time the receipt expires for apps purchased through the Volume Purchase Program, in the
//     // Pacific Time zone.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub expiration_date_pst: Option<String>,

//     #[serde(skip_serializing_if = "util::is_empty")]
//     #[serde(default)]
//     pub in_app: Vec<AppleInAppReceipt>,

//     // The version of the app that the user originally purchased. This value does not change, and
//     // corresponds to the value of CFBundleVersion (in iOS) or CFBundleShortVersionString (in
//     // macOS) in the Info.plist file of the original purchase. In the sandbox environment, the
//     // value is always "1.0".
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub original_application_version: Option<String>,

//     // The time of the original app purchase, in a date-time format similar to ISO 8601.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub original_purchase_date: Option<String>,

//     // The time of the original app purchase, in UNIX epoch time format, in milliseconds. Use this
//     // time format for processing dates.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub original_purchase_date_ms: Option<String>,

//     // The time of the original app purchase, in the Pacific Time zone.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub original_purchase_date_pst: Option<String>,

//     // The time the user ordered the app available for pre-order, in a date-time format similar to
//     // ISO 8601.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub preorder_date: Option<String>,

//     // The time the user ordered the app available for pre-order, in UNIX epoch time format, in
//     // milliseconds. This field is only present if the user pre-orders the app. Use this time
//     // format for processing dates.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub preorder_date_ms: Option<String>,

//     // The time the user ordered the app available for pre-order, in the Pacific Time zone.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub preorder_date_pst: Option<String>,

//     // The time the App Store generated the receipt, in a date-time format similar to ISO 8601.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub receipt_creation_date: Option<String>,

//     // The time the App Store generated the receipt, in UNIX epoch time format, in milliseconds.
//     // Use this time format for processing dates. This value does not change.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub receipt_creation_date_ms: Option<String>,

//     // The time the App Store generated the receipt, in the Pacific Time zone.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub receipt_creation_date_pst: Option<String>,

//     // The type of receipt generated. The value corresponds to the environment in which the app or
//     // VPP purchase was made.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub receipt_type: Option<AppleReceiptType>,

//     // The time the request to the verifyReceipt endpoint was processed and the response was
//     // generated, in a date-time format similar to ISO 8601.`
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub request_date: Option<String>,

//     // The time the request to the verifyReceipt endpoint was processed and the response was
//     // generated, in UNIX epoch time format, in milliseconds. Use this time format for processing
//     // dates.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub request_date_ms: Option<String>,

//     // The time the request to the verifyReceipt endpoint was processed and the response was
//     // generated, in the Pacific Time zone.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub request_date_pst: Option<String>,

//     // An arbitrary number that identifies a revision of your app. In the sandbox, this key's value
//     // is “0”.
//     #[serde(skip_serializing_if = "util::is_none")]
//     #[serde(default)]
//     pub version_external_identifier: Option<i32>,
// }
