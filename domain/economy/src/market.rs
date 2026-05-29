use async_trait::async_trait;
use chrono::{Duration, Utc};
use contracts::{
    error::{FfError, Result},
    traits::EconomyEngine,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use types::economy::{AuctionItem, MarketListing, Resource, Wallet};
use uuid::Uuid;

/// In-Memory-Implementierung der Wirtschaft.
#[derive(Default)]
struct Inner {
    listings: HashMap<Uuid, MarketListing>,
    auctions: HashMap<Uuid, AuctionItem>,
    wallets: HashMap<Uuid, Wallet>,
}

pub struct EconomyStore {
    inner: Arc<RwLock<Inner>>,
}

impl EconomyStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::default())),
        }
    }
}

impl Default for EconomyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EconomyEngine for EconomyStore {
    async fn list_resource(
        &self,
        seller_id: Uuid,
        resource: Resource,
        price: u64,
    ) -> Result<MarketListing> {
        let l = MarketListing::new(seller_id, resource, price);
        self.inner.write().await.listings.insert(l.id, l.clone());
        Ok(l)
    }

    async fn purchase(&self, lid: Uuid, buyer_id: Uuid, qty: u64) -> Result<()> {
        let mut s = self.inner.write().await;
        let cost = s
            .listings
            .get(&lid)
            .ok_or_else(|| FfError::ListingNotFound(lid.to_string()))?
            .price_per_unit
            .saturating_mul(qty);
        let seller_id = s.listings.get(&lid).unwrap().seller_id;
        let bw = s.wallets.entry(buyer_id).or_default();
        if !bw.debit(cost) {
            return Err(FfError::InsufficientFunds {
                needed: cost,
                available: bw.balance,
            });
        }
        s.wallets.entry(seller_id).or_default().credit(cost);
        if let Some(l) = s.listings.get_mut(&lid) {
            l.active = false;
        }
        Ok(())
    }

    async fn get_listings(&self) -> Result<Vec<MarketListing>> {
        Ok(self
            .inner
            .read()
            .await
            .listings
            .values()
            .filter(|l| l.active)
            .cloned()
            .collect())
    }

    async fn create_auction(
        &self,
        seller_id: Uuid,
        resource: Resource,
        start_price: u64,
        dur: u64,
    ) -> Result<AuctionItem> {
        let a = AuctionItem::new(
            seller_id,
            resource,
            start_price,
            Utc::now() + Duration::seconds(dur as i64),
        );
        self.inner.write().await.auctions.insert(a.id, a.clone());
        Ok(a)
    }

    async fn place_bid(&self, aid: Uuid, bidder_id: Uuid, amount: u64) -> Result<()> {
        let mut s = self.inner.write().await;
        let a = s
            .auctions
            .get_mut(&aid)
            .ok_or_else(|| FfError::ListingNotFound(aid.to_string()))?;
        if a.closed {
            return Err(FfError::AuctionClosed(aid.to_string()));
        }
        if !a.place_bid(bidder_id, amount) {
            return Err(FfError::InsufficientFunds {
                needed: a.current_bid + 1,
                available: amount,
            });
        }
        Ok(())
    }

    async fn get_auctions(&self) -> Result<Vec<AuctionItem>> {
        Ok(self
            .inner
            .read()
            .await
            .auctions
            .values()
            .filter(|a| !a.closed)
            .cloned()
            .collect())
    }

    async fn get_wallet(&self, id: Uuid) -> Result<Wallet> {
        Ok(self
            .inner
            .read()
            .await
            .wallets
            .get(&id)
            .cloned()
            .unwrap_or_default())
    }

    async fn transfer(&self, from: Uuid, to: Uuid, amount: u64) -> Result<()> {
        let mut s = self.inner.write().await;
        let fw = s.wallets.entry(from).or_default();
        if !fw.debit(amount) {
            return Err(FfError::InsufficientFunds {
                needed: amount,
                available: fw.balance,
            });
        }
        s.wallets.entry(to).or_default().credit(amount);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wallet_transfer_ok() {
        let store = EconomyStore::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        store
            .inner
            .write()
            .await
            .wallets
            .insert(a, Wallet { balance: 100 });
        store.transfer(a, b, 40).await.unwrap();
        assert_eq!(store.get_wallet(a).await.unwrap().balance, 60);
        assert_eq!(store.get_wallet(b).await.unwrap().balance, 40);
    }

    #[tokio::test]
    async fn insufficient_funds_error() {
        let store = EconomyStore::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let err = store.transfer(a, b, 1).await.unwrap_err();
        assert!(matches!(err, FfError::InsufficientFunds { .. }));
    }
}
