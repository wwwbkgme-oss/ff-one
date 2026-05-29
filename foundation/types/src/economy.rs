//! Economy domain types — resources, market listings, auctions, wallets.
//!
//! Quest types live in `ff-types::quest` (separate concept).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Category of a tradeable resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    RawMaterial(String),
    CraftedItem(String),
    QuestItem(String),
    Currency,
}

/// A stackable tradeable resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub quantity: u64,
    pub unit_value: u64,
}

impl Resource {
    pub fn new(resource_type: ResourceType, quantity: u64, unit_value: u64) -> Self {
        Self {
            resource_type,
            quantity,
            unit_value,
        }
    }

    pub fn total_value(&self) -> u64 {
        self.unit_value.saturating_mul(self.quantity)
    }
}

/// Fixed-price market listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketListing {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub resource: Resource,
    pub price_per_unit: u64,
    pub listed_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

impl MarketListing {
    pub fn new(seller_id: Uuid, resource: Resource, price_per_unit: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            seller_id,
            resource,
            price_per_unit,
            listed_at: Utc::now(),
            expires_at: None,
            active: true,
        }
    }
}

/// Competitive-bid auction item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionItem {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub resource: Resource,
    pub starting_price: u64,
    pub current_bid: u64,
    pub highest_bidder: Option<Uuid>,
    pub ends_at: DateTime<Utc>,
    pub closed: bool,
}

impl AuctionItem {
    pub fn new(
        seller_id: Uuid,
        resource: Resource,
        starting_price: u64,
        ends_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            seller_id,
            resource,
            starting_price,
            current_bid: starting_price,
            highest_bidder: None,
            ends_at,
            closed: false,
        }
    }

    /// Returns `true` if the bid was accepted.
    pub fn place_bid(&mut self, bidder: Uuid, amount: u64) -> bool {
        if amount > self.current_bid && !self.closed {
            self.current_bid = amount;
            self.highest_bidder = Some(bidder);
            true
        } else {
            false
        }
    }
}

/// Gold wallet for an agent or zone treasury.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wallet {
    pub balance: u64,
}

impl Wallet {
    pub fn credit(&mut self, amount: u64) {
        self.balance = self.balance.saturating_add(amount);
    }

    /// Returns `true` on success, `false` if insufficient funds.
    pub fn debit(&mut self, amount: u64) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            true
        } else {
            false
        }
    }
}
