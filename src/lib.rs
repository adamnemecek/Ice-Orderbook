mod matching_engine;
pub use matching_engine::orderbook::Orderbook;
pub use matching_engine::order::{FillEvent, Order, OrderKey, OrderSide, IcebergOrder};

#[cfg(test)]
mod tests {
    use crate::*;

    const LIMIT_BUY_100_15: Order = Order {
        order_key: OrderKey {
            id: 1,
            timestamp: 0,
            price: 100,
            order_side: OrderSide::Buy,
        },
        quantity: 15,
        iceberg: None
    };

    const LIMIT_SELL_101_15: Order = Order {
        order_key: OrderKey {
            id: 2,
            timestamp: 0,
            price: 101,
            order_side: OrderSide::Sell,
        },
        quantity: 15,
        iceberg: None
    };

    const LIMIT_BUY_98_100: Order = Order {
        order_key: OrderKey {
            id: 3,
            timestamp: 0,
            price: 98,
            order_side: OrderSide::Buy,
        },
        quantity: 100,
        iceberg: None
    };

    #[test]
    fn no_fill() {
        let mut orderbook = Orderbook::new();

        let events = orderbook.process_order(&mut LIMIT_BUY_100_15.clone());
        assert_eq!(events, vec![]);

        let events = orderbook.process_order(&mut LIMIT_SELL_101_15.clone());
        assert_eq!(events, vec![]);

        assert_eq!(orderbook.get_buy_orders(), vec![Order {order_key: OrderKey {timestamp: 1, ..LIMIT_BUY_100_15.order_key}, ..LIMIT_BUY_100_15}]);
        assert_eq!(orderbook.get_sell_orders(), vec![Order {order_key: OrderKey {timestamp: 2, ..LIMIT_SELL_101_15.order_key}, ..LIMIT_SELL_101_15}]);
    }

    #[test]
    fn whole_fill() {
        let mut orderbook = Orderbook::new();

        let events = orderbook.process_order(&mut LIMIT_BUY_100_15.clone());
        assert_eq!(events, vec![]);

        let events = orderbook.process_order(&mut LIMIT_SELL_101_15.clone());
        assert_eq!(events, vec![]);

        let third_order = Order {
            order_key: OrderKey {
                id: 3,
                timestamp: 0,
                price: 100,
                order_side: OrderSide::Sell,
            },
            quantity: 5,
            iceberg: None
        };

        let events = orderbook.process_order(&mut third_order.clone());
        assert_eq!(events, vec![FillEvent {buy_order_id: 1, sell_order_id: 3, price: 100, quantity: 5}]);

        assert_eq!(orderbook.get_buy_orders(), vec![Order {quantity: 10, order_key: OrderKey {timestamp: 1, ..LIMIT_BUY_100_15.order_key}, ..LIMIT_BUY_100_15}]);
        assert_eq!(orderbook.get_sell_orders(), vec![Order {order_key: OrderKey {timestamp: 2, ..LIMIT_SELL_101_15.order_key}, ..LIMIT_SELL_101_15}]);
    }

    #[test]
    fn partial_fill() {
        let mut orderbook = Orderbook::new();

        let events = orderbook.process_order(&mut LIMIT_BUY_100_15.clone());
        assert_eq!(events, vec![]);

        let events = orderbook.process_order(&mut LIMIT_BUY_98_100.clone());
        assert_eq!(events, vec![]);

        let sell_order = Order {
            order_key: OrderKey {
                id: 4,
                timestamp: 0,
                price: 99,
                order_side: OrderSide::Sell,
            },
            quantity: 50,
            iceberg: None
        };

        let events = orderbook.process_order(&mut sell_order.clone());
        assert_eq!(events, vec![FillEvent {buy_order_id: 1, sell_order_id: 4, price: 100, quantity: 15}]);

        assert_eq!(orderbook.get_buy_orders(), vec![Order {order_key: OrderKey {timestamp: 2, ..LIMIT_BUY_98_100.order_key}, ..LIMIT_BUY_98_100}]);
        assert_eq!(orderbook.get_sell_orders(), vec![Order {quantity: 35, order_key: OrderKey {timestamp: 3, ..sell_order.order_key}, ..sell_order}]);
    }

    #[test]
    fn exact_fill() {
        let mut orderbook = Orderbook::new();

        let events = orderbook.process_order(&mut LIMIT_BUY_100_15.clone());
        assert_eq!(events, vec![]);

        let sell_order = Order {
            order_key: OrderKey {
                id: 2,
                timestamp: 0,
                price: 100,
                order_side: OrderSide::Sell,
            },
            quantity: 15,
            iceberg: None
        };
        let events = orderbook.process_order(&mut sell_order.clone());
        assert_eq!(events, vec![FillEvent {buy_order_id: 1, sell_order_id: 2, price: 100, quantity: 15}]);

        assert_eq!(orderbook.get_buy_orders(), vec![]);
        assert_eq!(orderbook.get_sell_orders(), vec![]);
    }


    const ICEBERG_BUY_100_100_500: Order = Order {
        order_key: OrderKey {
            id: 4,
            timestamp: 0,
            price: 100,
            order_side: OrderSide::Buy,
        },
        quantity: 100,
        iceberg: Some(IcebergOrder {
            peak_size: 100,
            hidden_quantity: 400
        })
    };

    const ICEBERG_SELL_100_25_300: Order = Order {
        order_key: OrderKey {
            id: 5,
            timestamp: 0,
            price: 100,
            order_side: OrderSide::Sell,
        },
        quantity: 25,
        iceberg: Some(IcebergOrder {
            peak_size: 25,
            hidden_quantity: 275
        })
    };


    #[test]
    fn iceberg_agains_limit_orders() {
        let mut orderbook = Orderbook::new();

        (0..4).for_each(|i| {
            orderbook.process_order(&mut Order {
                order_key: OrderKey {
                    id: i,
                    timestamp: 0,
                    price: 100 + i,
                    order_side: OrderSide::Buy,
                },
                quantity: 30,
                iceberg: None
            });
        });


        let events = orderbook.process_order(&mut ICEBERG_SELL_100_25_300.clone());
        assert_eq!(events, vec![
            FillEvent {buy_order_id: 3, sell_order_id: 5, price: 103, quantity: 25},
            FillEvent {buy_order_id: 3, sell_order_id: 5, price: 103, quantity: 5},

            FillEvent {buy_order_id: 2, sell_order_id: 5, price: 102, quantity: 20},
            FillEvent {buy_order_id: 2, sell_order_id: 5, price: 102, quantity: 10},

            FillEvent {buy_order_id: 1, sell_order_id: 5, price: 101, quantity: 15},
            FillEvent {buy_order_id: 1, sell_order_id: 5, price: 101, quantity: 15},

            FillEvent {buy_order_id: 0, sell_order_id: 5, price: 100, quantity: 10},
            FillEvent {buy_order_id: 0, sell_order_id: 5, price: 100, quantity: 20},
        ]);

        assert_eq!(orderbook.get_buy_orders(), vec![]);
        assert_eq!(orderbook.get_sell_orders(), vec![
            Order {
                order_key: OrderKey {
                    timestamp: 5,
                    ..ICEBERG_SELL_100_25_300.order_key
                },
                quantity: 5,
                iceberg: Some(IcebergOrder {
                    hidden_quantity: 175,
                    ..ICEBERG_SELL_100_25_300.iceberg.unwrap()
                }),
                ..ICEBERG_SELL_100_25_300
            }
        ]);
    }

    #[test]
    fn limit_order_against_iceberg() {
        let mut orderbook = Orderbook::new();

        orderbook.process_order(&mut ICEBERG_SELL_100_25_300.clone());

        let limit_buy_order = Order {
            order_key: OrderKey {
                id: 1,
                timestamp: 0,
                price: 100,
                order_side: OrderSide::Buy,
            },
            quantity: 400,
            iceberg: None
        };
        let events = orderbook.process_order(&mut limit_buy_order.clone());


        assert_eq!(events, vec![FillEvent {buy_order_id: 1, sell_order_id: 5, price: 100, quantity: 25}; 12]);

        assert_eq!(orderbook.get_buy_orders(), vec![
            Order {
                quantity: 100,
                order_key: OrderKey {
                    timestamp: 2,
                    ..limit_buy_order.order_key
                },
                ..limit_buy_order
            }
        ]);
        assert_eq!(orderbook.get_sell_orders(), vec![]);
    }

    #[test]
    fn icebergs_preserving_order_test() {
        let mut orderbook = Orderbook::new();

        (0..3).for_each(|i| {
            orderbook.process_order(&mut Order {
                order_key: OrderKey {
                    id: i,
                    timestamp: 0,
                    price: 100,
                    order_side: OrderSide::Sell,
                },
                quantity: 100,
                iceberg: Some(IcebergOrder {
                    hidden_quantity: 100 * (i+1),
                    peak_size: 100
                })
            });
        });

        let events = orderbook.process_order(&mut ICEBERG_BUY_100_100_500.clone());


        assert_eq!(events, vec![
            FillEvent {buy_order_id: 4, sell_order_id: 0, price: 100, quantity: 100},
            FillEvent {buy_order_id: 4, sell_order_id: 1, price: 100, quantity: 100},
            FillEvent {buy_order_id: 4, sell_order_id: 2, price: 100, quantity: 100},
            FillEvent {buy_order_id: 4, sell_order_id: 0, price: 100, quantity: 100},
            FillEvent {buy_order_id: 4, sell_order_id: 1, price: 100, quantity: 100},
        ]);

        assert_eq!(orderbook.get_buy_orders(), vec![]);
        assert_eq!(orderbook.get_sell_orders(), vec![
            Order {
                quantity: 100,
                order_key: OrderKey {
                    timestamp: 7,
                    id: 2,
                    price: 100,
                    order_side: OrderSide::Sell
                },
                iceberg: Some(IcebergOrder {
                    peak_size: 100,
                    hidden_quantity: 200
                })
            },
            Order {
                quantity: 100,
                order_key: OrderKey {
                    timestamp: 8,
                    id: 1,
                    price: 100,
                    order_side: OrderSide::Sell
                },
                iceberg: Some(IcebergOrder {
                    peak_size: 100,
                    hidden_quantity: 0
                })
            }
        ]);
    }
}