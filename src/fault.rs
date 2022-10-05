#![allow(dead_code)]

use warp::reject;

#[derive(Debug)]
pub enum Fault {
    Unspecified(String),
    Set(Vec<Fault>),
    ApiLevelNoLongerSupported,
    Throttling,
    Duplicate(String),
    WrongPassword,
    NotFound(String),
    Unauthorized,
    Forbidden(String),
    IllegalArgument(String),
    IllegalState(String),
    Ineligible(String),
    NoData,
    NoExtra,
    Depleted,
}

impl reject::Reject for Fault {}

pub enum FaultCode {
    Unspecified = 0,
    Set = 1,
    ApiLevelNoLongerSupported = 2,
    Throttling = 3,
    Duplicate = 4,
    WrongPassword = 5,
    NotFound = 6,
    Unauthorized = 7,
    Forbidden = 8,
    IllegalArgument = 9,
    IllegalState = 10,
    Ineligible = 11,
    NoData = 12,
    NoExtra = 13,
    Depleted = 14,
}
