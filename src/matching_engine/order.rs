use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FillEvent {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub price: u64,
    pub quantity: u64
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct OrderKey {
    pub id: u64,
    pub price: u64,
    #[serde(skip_serializing)]
    pub timestamp: u64,
    #[serde(skip_serializing)]
    pub order_side: OrderSide,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub struct Order {
    #[serde(flatten)]
    pub order_key: OrderKey,
    pub quantity: u64,
    #[serde(skip_serializing)]
    pub iceberg: Option<IcebergOrder>
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub struct IcebergOrder {
    pub hidden_quantity: u64,
    pub peak_size: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell
}

impl Order {
    pub fn reload_iceberg_order(&mut self) {
        if self.quantity == 0 {
            if let Some(ref mut iceberg) = self.iceberg {
                let reload_quantity = std::cmp::min(iceberg.peak_size, iceberg.hidden_quantity);
                self.quantity = reload_quantity;
                iceberg.hidden_quantity -= reload_quantity;
            }
        }
    }

    pub fn is_iceberg(&self) -> bool {
        self.iceberg.is_some()
    }

    pub fn empty(&self) -> bool {
        match self.iceberg {
            None => self.quantity == 0,
            Some(ref iceberg) => self.quantity == 0 && iceberg.hidden_quantity == 0
        }
    }

    pub fn get_fill_event(&self, maker_order: &Self) -> FillEvent {
        let fill_quantity = std::cmp::min(self.quantity, maker_order.quantity);
        let price = maker_order.order_key.price;
        let (buy_order_id, sell_order_id) = match self.order_key.order_side {
            OrderSide::Buy => (self.order_key.id, maker_order.order_key.id),
            OrderSide::Sell => (maker_order.order_key.id, self.order_key.id)
        };
        FillEvent {buy_order_id, sell_order_id, price, quantity: fill_quantity}
    }
}

impl Ord for OrderKey {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price < other.price {
            return match self.order_side {
                OrderSide::Buy => Ordering::Less,
                OrderSide::Sell => Ordering::Greater
            }
        }
        else if self.price > other.price {
            match self.order_side {
                OrderSide::Buy => return Ordering::Greater,
                OrderSide::Sell => return Ordering::Less
            }
        } else {
            match self.timestamp < other.timestamp {
                true => Ordering::Greater,
                false => Ordering::Less
            }
        }
    }
}

impl PartialOrd for OrderKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderKey {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.timestamp == other.timestamp
    }
}

impl Eq for OrderKey { }
