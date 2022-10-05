use serde_repr::{Deserialize_repr, Serialize_repr};

// cf. https://developer.apple.com/documentation/appstoreserverapi/status
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum AppleSubscriptionStatus {
    Active = 1,
    Expired = 2,
    BillingRetry = 3,
    BillingGrace = 4,
    Revoked = 5,
}
