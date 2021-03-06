use std::collections::{BTreeMap, BTreeSet, btree_set};

use uuid::Uuid;
use rust_decimal::prelude::{Decimal, Zero};
use serde::{Serialize, Deserialize};

use crate::orderbook::order::{timestamp, generate_uuid, OrderDirection, LimitOrder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookRequest {
    Open(OpenEvent),
    Cancel(CancelEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookResult {
    Opened(OpenedEvent),
    Filled(FilledEvent),
    Canceled(CanceledEvent),
    Bounce(BounceEvent),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BounceReason {
    OrderNotFound,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OpenEvent {
    pub(crate) owner: Uuid,
    pub(crate) price: Decimal,
    pub(crate) size: Decimal,
    pub(crate) direction: OrderDirection,
    pub(crate) timestamp: i64,
    pub(crate) uuid: Option<Uuid>,
}

impl From<OpenEvent> for LimitOrder {
    fn from(open_event: OpenEvent) -> Self {
        Self {
            id: open_event.uuid.unwrap(),
            parent: None,
            owner: open_event.owner,
            price: open_event.price,
            size: open_event.size,
            direction: open_event.direction,
            timestamp: open_event.timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OpenedEvent { // exactly the same as LimitOrder, just a different name. Hacky!
    pub(crate) id: Uuid,
    pub(crate) parent: Option<Uuid>,
    pub(crate) owner: Uuid,
    pub(crate) price: Decimal,
    pub(crate) size: Decimal,
    pub(crate) direction: OrderDirection,
    pub(crate) timestamp: i64,
}

impl From<LimitOrder> for OpenedEvent {
    fn from(order: LimitOrder) -> Self {
        Self {
            id: order.id,
            parent: order.parent,
            owner: order.owner,
            price: order.price,
            size: order.size,
            direction: order.direction,
            timestamp: order.timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilledEvent {
    pub(crate) id: Uuid,
    pub(crate) owner: Uuid,
    pub(crate) parent: Option<Uuid>,
    pub(crate) price: Decimal,
    pub(crate) size: Decimal,
    pub(crate) timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CancelEvent {
    pub(crate) id: Uuid,
    pub(crate) owner: Uuid,
    pub(crate) timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CanceledEvent {
    pub(crate) id: Uuid,
    pub(crate) owner: Uuid,
    pub(crate) parent: Option<Uuid>,
    pub(crate) timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BounceEvent {
    pub(crate) id: Option<Uuid>,
    pub(crate) owner: Uuid,
    pub(crate) reason: BounceReason,
    pub(crate) timestamp: i64,
}

#[derive(Debug, Default)]
struct BookLevel {
    size: Decimal,
    orders: BTreeSet<LimitOrder>,
}

#[derive(Debug, Default)]
struct Book {
    price_books: BTreeMap<Decimal, BookLevel>,
    price_id_sets: BTreeMap<Decimal, BTreeSet<Uuid>>
}

#[derive(Debug)]
pub struct OrderBook {
    bid_book: Book,
    ask_book: Book,
    counter: u16,
}

impl BookLevel {
    pub fn new() -> Self { BookLevel::default() }

    pub fn open_order(&mut self, order: LimitOrder) -> BookResult {
        self.size += order.size;
        self.orders.insert(order);
        BookResult::Opened(OpenedEvent::from(order))
    }

    fn find_order_with_id(&self, id: &Uuid) -> Option<&LimitOrder> {
        for order in self.orders.iter() {
            if order.id == *id {
                return Some(order);
            }
        }
        None
    }

    pub fn remove_order(&mut self, id: &Uuid) -> Option<LimitOrder> {
        if let Some(order_to_remove) = self.find_order_with_id(id).cloned() {
            if self.orders.remove(&order_to_remove) {
                return Some(order_to_remove)
            } else {
                panic!("Could not remove order even though it was found!");
            }
        }
        None
    }

    pub fn iter(&self) -> btree_set::Iter<'_, LimitOrder> {
        self.orders.iter()
    }
}

impl Book {
    pub fn new() -> Self { Book::default() }

    fn open_order(&mut self, order: LimitOrder) -> BookResult {
        // if no data yet exists at this level
        if self.mut_price_level(&order.price).is_none() {
            self.price_books.insert(order.price, BookLevel::new()); // create price level
            self.price_id_sets.insert(order.price, BTreeSet::new()); // create id set for level
        }

        self.price_id_sets.get_mut(&order.price).unwrap().insert(order.id); // add current id
        self.price_books.get_mut(&order.price).unwrap().open_order(order) // push order into the orderbook
    }

    fn cancel_order(&mut self, cancel_event: CancelEvent) -> Option<LimitOrder> {
        // storing id's in the set by price means only a linear search over prices
        for (price, id_set) in self.price_id_sets.iter_mut() {
            // then a fast check if the id exists in that price's set of ids
            if id_set.contains(&cancel_event.id) {
                id_set.remove(&cancel_event.id);
                return self.price_books.get_mut(price).unwrap().remove_order(&cancel_event.id);
            }
        }

        // failed to cancel order, so return None
        None
    }

    fn mut_price_level(&mut self, price: &Decimal) -> Option<&mut BookLevel> {
        self.price_books.get_mut(price)
    }
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bid_book: Book::new(),
            ask_book: Book::new(),
            counter: 0,
        }
    }

    fn get_counter(&mut self) -> u16 {
        let c = self.counter;

        self.counter += 1;

        return c;
    }

    fn set_counter(&mut self, counter: u16) {
        self.counter = counter;
    }

    pub fn process_request(&mut self, book_msg: BookRequest) -> Vec<BookResult> {
        let ts = timestamp();
        match book_msg {
            BookRequest::Open(mut open_event) => {
                open_event.uuid = Some(generate_uuid(self.get_counter()));
                open_event.timestamp = ts;
                self.place_order(open_event)
            },
            BookRequest::Cancel(mut cancel_event) => {
                cancel_event.timestamp = ts;
                self.cancel_order(cancel_event) },
        }
    }

    fn place_order(&mut self, open_event: OpenEvent) -> Vec<BookResult> {
        match open_event.direction {
            OrderDirection::Bid => { self.fill_bid(LimitOrder::from(open_event)) },
            OrderDirection::Ask => { self.fill_ask(LimitOrder::from(open_event)) },
        }
    }

    fn fill_bid(&mut self, bid: LimitOrder) -> Vec<BookResult> {
        // keep track of filled orders and record the opening of the initial trade
        let mut filled_asks: BTreeMap<Decimal, Vec<Uuid>> = BTreeMap::new();
        let mut events: Vec<BookResult> = vec![self.bid_book.open_order(bid)];

        let (bid_replacement, ask_replacement) = self.book_walk( bid, &mut events, &mut filled_asks);

        // remove all filled asks
        filled_asks.iter().for_each(|(price_key, ids)| {
            ids.iter().for_each(|order_id| {
                OrderBook::remove_order(&mut self.ask_book, price_key, order_id);
            });
        });

        // if an ask is partially filled put the remainder back on the orderbook
        if let Some(ask_replacement) = ask_replacement {
            events.push(self.ask_book.open_order(ask_replacement));
        }

        // if the opened bid is at all filled remove the order from the orderbook, record the event,
        // and put the remainder of the order back on the orderbook if it exists
        if let Some(bid_replacement) = bid_replacement {
            OrderBook::remove_order(&mut self.bid_book, &bid.price, &bid.id);

            events.push(BookResult::Filled(FilledEvent{
                id: bid.id,
                owner: bid.owner,
                parent: bid.parent,
                price: bid.price,
                size: bid.size - bid_replacement.size,
                timestamp: bid.timestamp,
            }));

            if bid_replacement.size > Decimal::zero() {
                events.push(self.bid_book.open_order(bid_replacement));
            }
        }

        events
    }

    fn fill_ask(&mut self, ask: LimitOrder) ->  Vec<BookResult> {
        // keep track of filled orders and record the opening of the initial trade
        let mut filled_bids: BTreeMap<Decimal, Vec<Uuid>> = BTreeMap::new();
        let mut events: Vec<BookResult> = vec![self.ask_book.open_order(ask)];

        let (ask_replacement, bid_replacement) = self.book_walk(ask, &mut events, &mut filled_bids);

        // remove all filled bids
        filled_bids.iter().for_each(|(price_key, ids)| {
           ids.iter().for_each(|order_id| {
               OrderBook::remove_order(&mut self.bid_book, price_key, order_id);
           });
        });

        // if there is a partially filled bid then put it on the orderbook
        if let Some(bid_replacement) = bid_replacement {
            events.push(self.bid_book.open_order(bid_replacement));
        }

        // if the opened ask is (partially) filled, then remove it from the orderbook
        // and replace it with the remainder (if it exists)
        if let Some(ask_replacement) = ask_replacement {
            OrderBook::remove_order(&mut self.ask_book, &ask.price, &ask.id);

            events.push(BookResult::Filled(FilledEvent{
                id: ask.id,
                owner: ask.owner,
                parent: ask.parent,
                price: ask.price,
                size: ask.size - ask_replacement.size,
                timestamp: ask.timestamp,
            }));

            if ask_replacement.size > Decimal::zero() {
                events.push(self.ask_book.open_order(ask_replacement));
            }
        }

        events
    }

    fn cancel_order(&mut self, cancel_event: CancelEvent) -> Vec<BookResult> {
        let ts = timestamp();
        if let Some(canceled_order) = self.bid_book.cancel_order(cancel_event) {
            return vec![BookResult::Canceled(CanceledEvent{
                id: canceled_order.id,
                owner: canceled_order.owner,
                parent: canceled_order.parent,
                timestamp: ts,
            })];
        }

        if let Some(canceled_order) = self.ask_book.cancel_order(cancel_event) {
            return vec![BookResult::Canceled(CanceledEvent{
                id: canceled_order.id,
                owner: canceled_order.owner,
                parent: canceled_order.parent,
                timestamp: ts,
            })];
        }

        let bounce_event = BounceEvent{
            id: Some(cancel_event.id),
            owner: cancel_event.owner,
            reason: BounceReason::OrderNotFound,
            timestamp: ts,
        };

        vec![BookResult::Bounce(bounce_event)]
    }

    fn calculate_fill(order_match: &LimitOrder, remainder: &mut Decimal, all_events: &mut Vec<BookResult>, filled_ids: &mut Vec<Uuid>, ts: i64, counter: u16) -> Option<LimitOrder> {
        if order_match.size <= *remainder {
            // full fill of the order_match
            all_events.push(BookResult::Filled(FilledEvent{
                id: order_match.id,
                owner: order_match.owner,
                parent: order_match.parent,
                price: order_match.price,
                size: order_match.size,
                timestamp: ts,
            }));

            *remainder -= order_match.size;

            filled_ids.push(order_match.id);

            None

        } else {
            // partially fill the order_match
            all_events.push(BookResult::Filled(FilledEvent{
                id: order_match.id,
                price: order_match.price,
                owner: order_match.owner,
                parent: order_match.parent,
                size: *remainder,
                timestamp: ts,
            }));

            // return a new limit order to represent the remainder of the other order
            let replacement = LimitOrder{
                id: generate_uuid(counter),
                parent: Some(order_match.id),
                owner: order_match.owner,
                price: order_match.price,
                size: order_match.size - *remainder,
                direction: order_match.direction,
                timestamp: order_match.timestamp
            };

            *remainder = Decimal::zero();

            filled_ids.push(order_match.id);

            Some(replacement)
        }
    }

    fn book_walk(&mut self, order: LimitOrder, all_events: &mut Vec<BookResult>, filled_ids: &mut BTreeMap<Decimal, Vec<Uuid>>) -> (Option<LimitOrder>, Option<LimitOrder>) {
        let ts = timestamp();
        let mut remainder = order.size;
        let mut partial_order_fill: Option<LimitOrder> = None;
        let mut partial_match_fill: Option<LimitOrder> = None;

        let mut counter = self.get_counter();

        // Get the books that are compatible with trade in the correct price order
        // filter out any empty prices
        let level_iter: Vec<(&Decimal, &BookLevel)> = match order.direction {
            OrderDirection::Bid => {
                let book= &self.ask_book;
                // get the lowest priced offers first
                book.price_books.iter().filter(
                    |(p, lvl)| {
                        **p <= order.price && lvl.size > Decimal::zero()
                    }).collect()
            },
            OrderDirection::Ask => {
                // reverse this iterator since we are walking down the bid orderbook
                // so we want to get the highest bids first
                let book = &self.bid_book;
                book.price_books.iter().rev().filter(
                    |(p, lvl)| {
                        **p >= order.price && lvl.size > Decimal::zero()
                    }).collect() // IDK an easy way to get around the fact that .rev() messes with the return type enough I have to collect everything to a vec first >:(
            },
        };

        // iterate through the valid price levels in the correct order
        for (price, level) in level_iter {
            for order_match in level.iter() {
                let mut remove_ids = Vec::new();

                partial_match_fill = OrderBook::calculate_fill(order_match, &mut remainder, all_events, &mut remove_ids, ts, counter);
                counter += 1;

                filled_ids.insert(*price, remove_ids);

                // check if the matching order was partially filled (
                // or if the submitted order is completely filled
                if partial_match_fill.is_some() || remainder == Decimal::zero() {
                    break
                }
            }

            if partial_match_fill.is_some() || remainder == Decimal::zero() {
                break;
            }
        }

        self.set_counter(counter);

        // partially filled the submitting order
        // generate a replacement order
        if remainder < order.size {
            partial_order_fill = Some(LimitOrder {
                id: generate_uuid(self.get_counter()),
                parent: Some(order.id),
                owner: order.owner,
                price: order.price,
                size: remainder, // if this is 0 then we know that the order is completely filled
                direction: order.direction,
                timestamp: ts,
            });
        }

        (partial_order_fill, partial_match_fill)
    }

    fn remove_order(book: &mut Book, price: &Decimal, id: &Uuid) {
        if let Some(id_set) = book.price_id_sets.get_mut(price) {
            id_set.remove(id);
        }

        if let Some(level) = book.price_books.get_mut(price) {
            level.remove_order(id);
        }
    }
}