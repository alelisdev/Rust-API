use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum AppleApiErrorCode {
    Unknown = 0,

    // An error response that indicates the App Store account wasn’t found; to try again, resend
    // the same request.
    AccountNotFoundRetryableError = 4040002,

    // An error response that indicates the app wasn’t found; to try again, resend the same
    // request.
    AppNotFoundRetryableError = 4040004,

    // An error response that indicates an unknown error occurred; to try again, resend the same
    // request.
    GeneralInternalRetryableError = 5000001,

    // An error response that indicates the original transaction identifier wasn’t found; to try
    // again, resend the same request.
    OriginalTransactionIdNotFoundRetryableError = 4040006,

    // An error that indicates the App Store account wasn’t found.
    AccountNotFoundError = 4040001,

    // An error that indicates the app wasn’t found.
    AppNotFoundError = 4040003,

    // An error that indicates a general internal error.
    GeneralInternalError = 5000000,

    // An error that indicates an invalid request.
    GeneralBadRequestError = 4000000,

    // An error that indicates an invalid app identifier.
    InvalidAppIdentifierError = 4000002,

    // An error that indicates an invalid extend-by days value.
    InvalidExtendByDaysError = 4000009,

    // An error that indicates an invalid reason code.
    InvalidExtendReasonCodeError = 4000010,

    // An error that indicates an invalid original transaction identifier.
    InvalidOriginalTransactionIdError = 4000008,

    // An error that indicates an invalid request identifier.
    InvalidRequestIdentifierError = 4000011,

    // An error that indicates an invalid request revision.
    InvalidRequestRevisionError = 4000005,

    // An error that indicates an invalid original transaction identifier.
    OriginalTransactionIdNotFoundError = 4040005,

    // An error that indicates the subscription doesn’t qualify for a renewal-date extension due to
    // its subscription state.
    SubscriptionExtensionIneligibleError = 4030004,

    // An error that indicates the subscription doesn’t qualify for a renewal-date extension
    // because it has already received the maximum extensions.
    SubscriptionMaxExtensionError = 4030005,
}
