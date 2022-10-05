use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum AppleReceiptStatus {
    Valid = 0,

    // The request to the App Store was not made using the HTTP POST request method.
    Error21000 = 21000,

    // This status code is no longer sent by the App Store.
    Error21001 = 21001,

    // The data in the receipt-data property was malformed or the service experienced a temporary
    // issue. Try again.
    Error21002 = 21002,

    // The receipt could not be authenticated.
    Error21003 = 21003,

    // The shared secret you provided does not match the shared secret on file for your account.
    Error21004 = 21004,

    // The receipt server was temporarily unable to provide the receipt. Try again.
    Error21005 = 21005,

    // This receipt is valid but the subscription has expired. When this status code is returned to
    // your server, the receipt data is also decoded and returned as part of the response. Only
    // returned for iOS 6-style transaction receipts for auto-renewable subscriptions.
    Error21006 = 21006,

    // This receipt is from the test environment, but it was sent to the production environment for
    // verification.
    Error21007 = 21007,

    // This receipt is from the production environment, but it was sent to the test environment for
    // verification.
    Error21008 = 21008,

    // Internal data access error. Try again later.
    Error21009 = 21009,

    // The user account cannot be found or has been deleted.
    Error21010 = 21010,
}
