use std::cmp::Ordering;
use rust_decimal::prelude::Decimal;
use chrono;

use uuid::Uuid;
use uuid::v1::{Context, Timestamp};
use serde::{Serialize, Deserialize};

pub fn timestamp() -> i64 {
    chrono::offset::Utc::now().timestamp_millis()
}

pub fn generate_uuid(ctx: &Context, timestamp: i64) -> Uuid {
    let ts = Timestamp::from_unix(ctx, timestamp as u64, 0);
    Uuid::new_v1(ts, &[0, 1, 2, 3, 4, 5]).expect("Failed to generate Uuid")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OrderDirection {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, Eq, Ord)]
pub struct LimitOrder {
    pub(crate) id: Uuid,
    pub(crate) parent: Option<Uuid>,
    pub(crate) owner: Uuid,
    pub(crate) price: Decimal,
    pub(crate) size: Decimal,
    pub(crate) direction: OrderDirection,
    pub(crate) timestamp: i64,
}

impl PartialEq<Self> for LimitOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }

    fn ne(&self, other: &Self) -> bool {
        self.id != other.id
    }
}

impl PartialOrd<Self> for LimitOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.price.partial_cmp(&other.price) {
            Some(Ordering::Less) => { return Some(Ordering::Less); }
            Some(Ordering::Greater) => { return Some(Ordering:: Greater); }
            _ => {}
        }

        match self.timestamp.partial_cmp(&other.timestamp) {
            Some(Ordering::Less) => { return Some(Ordering::Less); }
            Some(Ordering::Greater) => { return Some(Ordering:: Greater);}
            _ => {}
        }

        self.id.partial_cmp(&other.id)
    }

    fn lt(&self, other: &Self) -> bool {
        if self.price != other.price {
            return self.price < other.price;
        }

        self.timestamp < other.timestamp
    }

    fn le(&self, other: &Self) -> bool {
        if self.price != other.price {
            return self.price < other.price;
        }

        if self.timestamp != other.timestamp {
            return self.timestamp < other.timestamp;
        }

        self.id == other.id
    }

    fn gt(&self, other: &Self) -> bool {
        if self.price != other.price {
            return self.price > other.price;
        }

        self.timestamp > other.timestamp
    }

    fn ge(&self, other: &Self) -> bool {
        if self.price != other.price {
            return self.price > other.price;
        }

        if self.timestamp != other.timestamp {
            return self.timestamp > other.timestamp;
        }

        self.id == other.id
    }
}