use crate::util;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    pub iss: String,

    #[serde(with = "jwt_numeric_date")]
    pub iat: DateTime<Utc>,

    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub scope: Option<String>,

    pub aud: String,

    pub nonce: String,

    pub bid: String,

    #[serde(with = "jwt_numeric_date")]
    pub exp: DateTime<Utc>,
}

impl Claims {
    // If a token should always be equal to its representation after serializing and deserializing
    // again, this function must be used for construction. `DateTime` contains a microsecond field
    // but JWT timestamps are defined as UNIX timestamps (seconds). This function normalizes the
    // timestamps.
    pub fn new(
        iss: &str,
        iat: DateTime<Utc>,
        scope: Option<String>,
        aud: &str,
        nonce: &str,
        bid: &str,
        exp: DateTime<Utc>,
    ) -> Self {
        let iss = String::from(iss);
        let aud = String::from(aud);
        let nonce = String::from(nonce);
        let bid = String::from(bid);

        // normalize the timestamps by stripping of microseconds
        let exp = exp
            .date()
            .and_hms_milli(exp.hour(), exp.minute(), exp.second(), 0);
        Self {
            iss,
            iat,
            scope,
            aud,
            nonce,
            bid,
            exp,
        }
    }
}

mod jwt_numeric_date {
    // Custom serialization of DateTime<Utc> to conform with the JWT spec (RFC 7519 section 2,
    // "Numeric Date").
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    // Serializes a DateTime<Utc> to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T).
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.timestamp();
        serializer.serialize_i64(timestamp)
    }

    // Attempts to deserialize an i64 and use as a Unix timestamp.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Utc.timestamp_opt(i64::deserialize(deserializer)?, 0)
            .single() // If there are multiple or no valid DateTimes from timestamp, return None.
            .ok_or_else(|| serde::de::Error::custom("Invalid Unix timestamp value."))
    }
}
