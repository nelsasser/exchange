use std::cmp::Ordering;
use rust_decimal::prelude::Decimal;
use chrono;

use uuid::Uuid;
use uuid::v1::{Context, Timestamp};
use serde::{Serialize, Deserialize};

pub fn timestamp() -> i64 {
    chrono::offset::Utc::now().timestamp()
}

pub fn timestamp_nanos() -> i64 {
    chrono::offset::Utc::now().timestamp_nanos()
}

pub fn generate_uuid(counter: u16) -> Uuid {
   let ts = Timestamp::from_rfc4122((timestamp_nanos() / 100) as u64, counter);
    Uuid::new_v1(ts, &[0, 1, 2, 3, 4, 5]).expect("Failed to generate Uuid")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OrderDirection {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct LimitOrder {
    pub(crate) id: Uuid,
    pub(crate) parent: Option<Uuid>,
    pub(crate) owner: Uuid,
    pub(crate) price: Decimal,
    pub(crate) size: Decimal,
    pub(crate) direction: OrderDirection,
    pub(crate) timestamp: i64,
}

impl Ord for LimitOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Less => { return Ordering::Less }
            Ordering::Greater => { return Ordering:: Greater }
            _ => {}
        }

        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Less => { return Ordering::Less }
            Ordering::Greater => { return Ordering:: Greater}
            _ => {}
        }

        self.id.cmp(&other.id)
    }

    fn min(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Less => self,
            _ => other
        }
    }

    fn max(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Greater => self,
            _ => other
        }
    }
}

impl PartialEq<Self> for LimitOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd<Self> for LimitOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Less
    }

    fn le(&self, other: &Self) -> bool {
        let c = self.cmp(other);

        c == Ordering::Less || c == Ordering::Equal
    }

    fn gt(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Greater
    }

    fn ge(&self, other: &Self) -> bool {
        let c = self.cmp(other);

        c == Ordering::Greater || c == Ordering::Equal
    }
}