pub mod book;
pub mod order;

#[macro_export]
macro_rules! bid {
    ($owner:expr, [$(($price:expr, $size:expr)),+]) => {
        vec![
            $(
                OpenEvent {
                    owner: $owner,
                    price: $price.into(),
                    size: $size.into(),
                    direction: OrderDirection::Bid,
                    timestamp: 0,
                    uuid: None
                }
            ),+
        ]
    };
}

#[macro_export]
macro_rules! ask {
    ($owner:expr, [$(($price:expr, $size:expr)),+]) => {
        vec![
            $(
                OpenEvent {
                    owner: $owner,
                    price: $price.into(),
                    size: $size.into(),
                    direction: OrderDirection::Ask,
                    timestamp: 0,
                    uuid: None
                }
            ),+
        ]
    };
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use uuid::v1::Context;
    use crate::orderbook::book::*;
    use crate::orderbook::order::*;
    use rust_decimal::prelude::Decimal;

    fn trader() -> Uuid {
        generate_uuid(&Context::new(rand::random()), timestamp())
    }

    #[test]
    fn open_order() {
        let mut orderbook = OrderBook::new();

        let trader_id = trader();

        let bid = bid!(trader_id, [(10, 1)])[0];

        let events = orderbook.process_request(BookRequest::Open(bid));

        assert_eq!(events.len(), 1);

        if let BookResult::Opened(opened_event) = events[0] {
            assert_eq!(opened_event.owner, trader_id);
            assert_eq!(opened_event.size, Decimal::from(1));
            assert_eq!(opened_event.price, Decimal::from(10));
            assert_eq!(opened_event.direction, OrderDirection::Bid);
        } else {
            panic!("Expected BookResult::Opened");
        }
    }

    #[test]
    fn cancel_open_order() {
        let mut orderbook = OrderBook::new();

        let trader_id = trader();

        let bid = bid!(trader_id, [(10, 1)])[0];

        // place the bid order
        let id = match orderbook.process_request(BookRequest::Open(bid))[0] {
            BookResult::Opened(opened_event) => opened_event.id,
            _ => panic!("Expected BookResult::Opened"),
        };

        // cancel the bid order
        let events = orderbook.process_request(BookRequest::Cancel(CancelEvent{
            id,
            owner: trader_id,
            timestamp: 0
        }));

        assert_eq!(events.len(), 1);

        if let BookResult::Canceled(canceled_event) = events[0] {
            assert_eq!(canceled_event.owner, trader_id);
            assert_eq!(canceled_event.id, id);
        } else {
            panic!("Expected canceled event");
        }
    }

    #[test]
    fn cancel_na_order() {
        let mut orderbook = OrderBook::new();

        let id = trader();

        let events = orderbook.process_request(BookRequest::Cancel(CancelEvent{
            id, // bogus id's to cancel
            owner: id,
            timestamp: 0
        }));

        assert_eq!(events.len(), 1);

        if let BookResult::Bounce(bounce_event) = events[0] {
            match bounce_event.reason {
                BounceReason::OrderNotFound => (),
                _ => panic!("Expected BounceReason to be OrderNotFound"),
            }
            assert_eq!(bounce_event.id.unwrap(), id);
            assert_eq!(bounce_event.owner, id);
        } else {
            panic!("Expected bounce");
        }
    }

    #[test]
    fn double_cancel() {
        let mut orderbook = OrderBook::new();

        let trader_id = trader();

        let bid = bid!(trader_id, [(10, 1)])[0];

        // place the bid order
        let id = match orderbook.process_request(BookRequest::Open(bid))[0] {
            BookResult::Opened(opened_event) => opened_event.id,
            _ => panic!("Expected BookResult::Opened"),
        };

        // cancel the bid order
        let events = orderbook.process_request(BookRequest::Cancel(CancelEvent{
            id,
            owner: trader_id,
            timestamp: 0
        }));

        assert_eq!(events.len(), 1);

        if let BookResult::Canceled(canceled_event) = events[0] {
            assert_eq!(canceled_event.owner, trader_id);
            assert_eq!(canceled_event.id, id);
        } else {
            panic!("Expected canceled event");
        }

        // attempt to cancel the same order after it has already been removed
        let events = orderbook.process_request(BookRequest::Cancel(CancelEvent{
            id, // same id and trader id
            owner: trader_id,
            timestamp: 0
        }));

        assert_eq!(events.len(), 1);

        if let BookResult::Bounce(bounce_event) = events[0] {
            match bounce_event.reason {
                BounceReason::OrderNotFound => (),
                _ => panic!("Expected BounceReason to be OrderNotFound"),
            }
            assert_eq!(bounce_event.id.unwrap(), id);
            assert_eq!(bounce_event.owner, trader_id);
        } else {
            panic!("Expected bounce");
        }
    }

    #[test]
    fn fill_single() {
        let mut orderbook = OrderBook::new();

        let trader_a = trader();
        let trader_b = trader();

        let bid = bid!(trader_a, [(10, 1)])[0];
        let ask = ask!(trader_b, [(10, 1)])[0];

        let bid_id = match orderbook.process_request(BookRequest::Open(bid))[0] {
            BookResult::Opened(opened_event) => opened_event.id,
            _ => panic!("Expected opened event"),
        };

        let events = orderbook.process_request(BookRequest::Open(ask));

        // should get 1 order opened and 2 order filled events
        // should be in the order of
        // 1) OPEN - ASK
        // 2) FILLED - BID
        // 3) FILLED - ASK
        assert_eq!(events.len(), 3);

        // check open ask is first & correct
        let ask_id = match events[0].clone() {
            BookResult::Opened(opened_event) => { opened_event.id },
            _ => panic!("Expected first result to be OpenedEvent for ask"),
        };

        match events[1].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_id);
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for bid"),
        };

        match events[2].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, ask_id);
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for ask"),
        };
    }

    #[test]
    fn fill_partial_single() {
        let mut orderbook = OrderBook::new();

        let trader_a = trader();
        let trader_b = trader();

        let bid = bid!(trader_a, [(10, 1)])[0];
        let ask = ask!(trader_b, [(10, 2)])[0];

        let bid_id = match orderbook.process_request(BookRequest::Open(bid))[0] {
            BookResult::Opened(opened_event) => opened_event.id,
            _ => panic!("Expected opened event"),
        };

        let events = orderbook.process_request(BookRequest::Open(ask));

        // should get 1 order opened and 2 order filled events
        // should be in the order of
        // 1) OPEN - ASK
        // 2) FILLED - BID
        // 3) PARTIAL FILLED - ASK
        // 4) OPEN - ASK
        assert_eq!(events.len(), 4);

        // check open ask is first & correct
        let ask_id = match events[0].clone() {
            BookResult::Opened(opened_event) => { opened_event.id },
            _ => panic!("Expected first result to be OpenedEvent for ask"),
        };

        match events[1].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_id);
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for bid"),
        };

        match events[2].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, ask_id);
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for ask"),
        };

        match events[3].clone() {
            BookResult::Opened(opened_event) => {
                assert_eq!(opened_event.parent.unwrap(), ask_id);
                assert_eq!(opened_event.price, Decimal::from(10));
                assert_eq!(opened_event.size, Decimal::from(1));
                assert_eq!(opened_event.owner, trader_b);
                assert_eq!(opened_event.direction, OrderDirection::Ask)
            },
            _ => panic!("Expected second result to be FilledEvent for ask"),
        };
    }

    #[test]
    fn fill_many() {
        let mut orderbook = OrderBook::new();

        let trader_a = trader();
        let trader_b = trader();

        let bid = bid!(trader_a, [(10, 1), (10, 1)]);
        let ask = ask!(trader_b, [(10, 2)])[0];

        let bid_ids: Vec<Uuid> = bid.iter().map(|b| {
           match orderbook.process_request(BookRequest::Open(*b))[0] {
               BookResult::Opened(opened_event) => opened_event.id,
               _ => panic!("Expected Opened BookResult"),
           }
        }).collect();

        let events = orderbook.process_request(BookRequest::Open(ask));

        // should get 1 order opened and 2 order filled events
        // should be in the order of
        // 1) OPEN - ASK
        // 2) FILLED - BID_1
        // 3) FILLED - BID_2
        // 4) FILLED - ASK
        assert_eq!(events.len(), 4);

        // check open ask is first & correct
        let ask_id = match events[0].clone() {
            BookResult::Opened(opened_event) => { opened_event.id },
            _ => panic!("Expected first result to be OpenedEvent for ask"),
        };

        match events[1].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_ids[0].clone());
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for bid 1"),
        };

        match events[2].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_ids[1].clone());
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for bid 2"),
        };

        match events[3].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, ask_id);
                assert_eq!(filled_event.size, Decimal::from(2));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for ask"),
        };
    }

    #[test]
    fn fill_partial_many() {
        let mut orderbook = OrderBook::new();

        let trader_a = trader();
        let trader_b = trader();

        let bid = bid!(trader_a, [(10, 1), (10, 1)]);
        let ask = ask!(trader_b, [(10, 3)])[0];

        let bid_ids: Vec<Uuid> = bid.iter().map(|b| {
            match orderbook.process_request(BookRequest::Open(*b))[0] {
                BookResult::Opened(opened_event) => opened_event.id,
                _ => panic!("Expected Opened BookResult"),
            }
        }).collect();

        let events = orderbook.process_request(BookRequest::Open(ask));

        // should get 1 order opened and 2 order filled events
        // should be in the order of
        // 1) OPEN - ASK
        // 2) FILLED - BID_1
        // 3) FILLED - BID_2
        // 4) FILLED - ASK
        // 5) OPEN - ASK
        assert_eq!(events.len(), 5);

        // check open ask is first & correct
        let ask_id = match events[0].clone() {
            BookResult::Opened(opened_event) => { opened_event.id },
            _ => panic!("Expected first result to be OpenedEvent for ask"),
        };

        match events[1].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_ids[0].clone());
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for bid 1"),
        };

        match events[2].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_ids[1].clone());
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for bid 2"),
        };

        match events[3].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, ask_id);
                assert_eq!(filled_event.size, Decimal::from(2));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for ask"),
        };

        match events[4].clone() {
            BookResult::Opened(opened_event) => {
                assert_eq!(opened_event.parent.unwrap(), ask_id);
                assert_eq!(opened_event.size, Decimal::from(1));
                assert_eq!(opened_event.price, Decimal::from(10));
                assert_eq!(opened_event.direction, OrderDirection::Ask);
            },
            _ => panic!("Expected second result to be OpenedEvent for ask"),
        };
    }

    #[test]
    fn fill_cross_levels_ask() {
        let mut orderbook = OrderBook::new();

        let trader_a = trader();
        let trader_b = trader();

        let bid = bid!(trader_a, [(10, 3), (11, 1)]);
        let ask = ask!(trader_b, [(10, 2)])[0];

        let bid_ids: Vec<Uuid> = bid.iter().map(|b| {
            match orderbook.process_request(BookRequest::Open(*b))[0] {
                BookResult::Opened(opened_event) => opened_event.id,
                _ => panic!("Expected Opened BookResult"),
            }
        }).collect();

        let events = orderbook.process_request(BookRequest::Open(ask));

        // should get 1 order opened and 2 order filled events
        // should be in the order of
        // 1) OPEN - ASK
        // 2) FILLED - BID_2_0
        // 3) FILLED - BID_1_0
        // 4) OPEN - BID_1_1
        // 5) FILLED - ASK
        assert_eq!(events.len(), 5);

        // check open ask is first & correct
        let ask_id = match events[0].clone() {
            BookResult::Opened(opened_event) => { opened_event.id },
            _ => panic!("Expected first result to be OpenedEvent for ask"),
        };

        match events[1].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_ids[1].clone());
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(11));
            },
            _ => panic!("Expected second result to be FilledEvent for bid 1"),
        };

        match events[2].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_ids[0].clone());
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for bid 2"),
        };

        match events[3].clone() {
            BookResult::Opened(opened_event) => {
                assert_eq!(opened_event.parent.unwrap(), bid_ids[0].clone());
                assert_eq!(opened_event.size, Decimal::from(2));
                assert_eq!(opened_event.price, Decimal::from(10));
                assert_eq!(opened_event.direction, OrderDirection::Bid);
                assert_eq!(opened_event.owner, trader_a);
            },
            _ => panic!("Expected 4th result to be OpenedEvent for bid"),
        };

        match events[4].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, ask_id);
                assert_eq!(filled_event.size, Decimal::from(2));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected 5th result to be FilledEvent for ask"),
        };
    }

    #[test]
    fn fill_cross_levels_bid() {
        let mut orderbook = OrderBook::new();

        let trader_a = trader();
        let trader_b = trader();

        let bid = bid!(trader_a, [(11, 5)])[0];
        let ask = ask!(trader_b, [(11, 2), (10, 4)]);

        let ask_ids: Vec<Uuid> = ask.iter().map(|a| {
            match orderbook.process_request(BookRequest::Open(*a))[0] {
                BookResult::Opened(opened_event) => opened_event.id,
                _ => panic!("Expected Opened BookResult"),
            }
        }).collect();

        let events = orderbook.process_request(BookRequest::Open(bid));

        // 1) OPEN - BID
        // 2) FILLED - ASK_2_0
        // 3) FILLED - ASK_1_0
        // 4) OPEN - ASK_1_1
        // 5) FILLED - BID
        assert_eq!(events.len(), 5);

        // check open ask is first & correct
        let bid_id = match events[0].clone() {
            BookResult::Opened(opened_event) => { opened_event.id },
            _ => panic!("Expected first result to be OpenedEvent for bid"),
        };

        match events[1].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, ask_ids[1].clone());
                assert_eq!(filled_event.size, Decimal::from(4));
                assert_eq!(filled_event.price, Decimal::from(10));
            },
            _ => panic!("Expected second result to be FilledEvent for ask 2"),
        };

        match events[2].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, ask_ids[0].clone());
                assert_eq!(filled_event.size, Decimal::from(1));
                assert_eq!(filled_event.price, Decimal::from(11));
            },
            _ => panic!("Expected second result to be FilledEvent for ask 1"),
        };

        match events[3].clone() {
            BookResult::Opened(opened_event) => {
                assert_eq!(opened_event.parent.unwrap(), ask_ids[0].clone());
                assert_eq!(opened_event.size, Decimal::from(1));
                assert_eq!(opened_event.price, Decimal::from(11));
                assert_eq!(opened_event.direction, OrderDirection::Ask);
                assert_eq!(opened_event.owner, trader_b);
            },
            _ => panic!("Expected 4th result to be OpenedEvent for ask"),
        };

        match events[4].clone() {
            BookResult::Filled(filled_event) => {
                assert_eq!(filled_event.id, bid_id);
                assert_eq!(filled_event.size, Decimal::from(5));
                assert_eq!(filled_event.price, Decimal::from(11));
                // TODO:
                //  I probably want to change the algorithm so that (partially) filling at a better price
                //  is made clear by the results of the FilledEvent, but it works the way it is, I'm running out of time,
                //  and people will never put their $$$ anywhere near this thing so who cares?
            },
            _ => panic!("Expected 5th result to be FilledEvent for bid"),
        };
    }
}