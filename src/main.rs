use std::io;
use serde_json;
use matching_engine::orderbook::{Orderbook};
use matching_engine::parse::{parse_order, DeserializedOrder};
mod matching_engine;


fn main() {
    let mut orderbook = Orderbook::new();

    let mut buffer = String::new();
    loop {
        buffer.clear();
        match io::stdin().read_line(&mut buffer) {
            Ok(0) => break,
            Ok(_) => (),
            Err(error) => println!("error: {}", error),
        }

        match serde_json::from_str::<DeserializedOrder>(buffer.as_str()) {
            Err(error) => println!("{}", error),
            Ok(deserialized_order) => {
                let mut order = parse_order(deserialized_order);
                let events = orderbook.process_order(&mut order);
                let orders = orderbook.get_orders();

                println!("{}", serde_json::to_string(&orders).unwrap());
                for event in events {
                    println!("{}", serde_json::to_string(&event).unwrap());
                }
                println!("");
            }
        }
    };
}
