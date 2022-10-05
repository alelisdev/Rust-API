use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProductType {
    Subscription,
    Consumable,
    NonConsumable,
}
