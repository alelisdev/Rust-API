#[derive(Debug, Clone)]
pub enum Purchase {
    AppleSubscription {
        product_id: String,
        transaction_id: String,
        original_transaction_id: String,
    },
    GoogleSubscription {
        product_id: String,
        token: String,
        package_name: String,
        order_id: String,
    },
    AppleConsumable {
        product_id: String,
        transaction_id: String,
    },
    GoogleConsumable {
        product_id: String,
        token: String,
        package_name: String,
        order_id: String,
    },
    AppleNonConsumable {
        product_id: String,
        transaction_id: String,
    },
    GoogleNonConsumable {
        product_id: String,
        token: String,
        package_name: String,
        order_id: String,
    },
}
