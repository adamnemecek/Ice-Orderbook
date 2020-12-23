use serde::{Deserialize};

use super::order::{Order, OrderKey, OrderSide, IcebergOrder};

#[derive(Debug, Deserialize)]
pub struct OrderCore {
    pub direction: OrderSide,
    pub id: u64,
    pub price: u64,
    pub quantity: u64
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "order")]
pub enum DeserializedOrder {
    Limit {
        #[serde(flatten)]
        order_core: OrderCore
    },
    Iceberg {
        #[serde(flatten)]
        order_core: OrderCore,
        peak: u64
    }
}


pub fn parse_order(deserialized_order: DeserializedOrder) -> Order {
    match deserialized_order {
        DeserializedOrder::Limit {order_core} => Order {
            order_key: OrderKey {
                id: order_core.id,
                price: order_core.price,
                timestamp: 0,
                order_side: order_core.direction
            },
            quantity: order_core.quantity,
            iceberg: None
        },
        DeserializedOrder::Iceberg {order_core, peak} => Order {
            order_key: OrderKey {
                id: order_core.id,
                price: order_core.price,
                timestamp: 0,
                order_side: order_core.direction
            },
            quantity: std::cmp::min(peak, order_core.quantity),
            iceberg: Some(IcebergOrder {
                peak_size: peak,
                hidden_quantity: std::cmp::max(order_core.quantity - peak, 0)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use crate::*;
    use matching_engine::order::{OrderKey, Order, IcebergOrder, OrderSide};

    #[test]
    pub fn parse_limit_order() {
        let serialized_iceberg_order = r#"{
            "type": "Limit",
            "order": {
                "direction": "Sell",
                "id": 99,
                "price": 100,
                "quantity": 500
            }
        }"#;

        match serde_json::from_str::<DeserializedOrder>(serialized_iceberg_order) {
            Err(error) => {
                println!("{}", error);
                assert!(false);
            },
            Ok(deserialized_order) => {
                let order = parse_order(deserialized_order);
                assert_eq!(order, Order {
                    order_key: OrderKey {
                        id: 99,
                        price: 100,
                        order_side: OrderSide::Sell,
                        timestamp: 0
                    },
                    quantity: 500,
                    iceberg: None
                })

            }
        }
    }

    #[test]
    pub fn parse_iceberg_order() {
        let serialized_iceberg_order = r#"{
            "type": "Iceberg",
            "order": {
                "direction": "Buy",
                "id": 4,
                "price": 100,
                "quantity": 500,
                "peak": 100
            }
        }"#;

        match serde_json::from_str::<DeserializedOrder>(serialized_iceberg_order) {
            Err(error) => {
                println!("{}", error);
                assert!(false);
            },
            Ok(deserialized_order) => {
                let order = parse_order(deserialized_order);
                assert_eq!(order, Order {
                    order_key: OrderKey {
                        id: 4,
                        price: 100,
                        order_side: OrderSide::Buy,
                        timestamp: 0
                    },
                    quantity: 100,
                    iceberg: Some(IcebergOrder {
                        peak_size: 100,
                        hidden_quantity: 400
                    })
                })

            }
        }
    }
}