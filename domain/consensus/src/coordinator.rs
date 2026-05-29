
use async_trait::async_trait;
use contracts::{error::{FfError, Result}, traits::ConsensusCoordinator};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use types::consensus::{ConsensusResult, ConsensusRound, WitnessRecord, WorldHash};
use uuid::Uuid;

/// In-Memory-Konsens-Koordinator.
///
/// Deterministisch: gleiche Witnesses → gleiche ConsensusResult.
pub struct ConsensusStore {
    rounds: Arc<RwLock<HashMap<u64, ConsensusRound>>>,
}

impl ConsensusStore {
    pub fn new() -> Self { Self { rounds: Arc::new(RwLock::new(HashMap::new())) } }
}

impl Default for ConsensusStore {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl ConsensusCoordinator for ConsensusStore {
    async fn start_round(&self, tick: u64, required: u32) -> Result<ConsensusRound> {
        let r = ConsensusRound::new(tick, required);
        self.rounds.write().await.insert(tick, r.clone());
        Ok(r)
    }

    async fn submit_witness(&self, tick: u64, agent_id: Uuid, hash: WorldHash) -> Result<()> {
        let mut rs = self.rounds.write().await;
        let r = rs.get_mut(&tick).ok_or(FfError::ConsensusDisputed(tick))?;
        r.add_witness(WitnessRecord::new(agent_id, tick, hash));
        Ok(())
    }

    async fn get_round(&self, tick: u64) -> Result<ConsensusRound> {
        self.rounds.read().await.get(&tick).cloned().ok_or(FfError::ConsensusDisputed(tick))
    }

    async fn await_consensus(&self, tick: u64, timeout_ms: u64) -> Result<ConsensusRound> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            let r = self.get_round(tick).await?;
            if r.is_finalised() { return Ok(r); }
            if tokio::time::Instant::now() >= deadline {
                return Err(FfError::ConsensusDisputed(tick));
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unanimous_two_witnesses() {
        let cc = ConsensusStore::new();
        cc.start_round(1, 2).await.unwrap();
        let h = WorldHash("abc123".into());
        cc.submit_witness(1, Uuid::new_v4(), h.clone()).await.unwrap();
        cc.submit_witness(1, Uuid::new_v4(), h).await.unwrap();
        let r = cc.get_round(1).await.unwrap();
        assert!(matches!(r.result, Some(ConsensusResult::Unanimous(_))));
    }

    #[tokio::test]
    async fn majority_two_of_three() {
        let cc = ConsensusStore::new();
        cc.start_round(2, 3).await.unwrap();
        let h1 = WorldHash("aaa".into());
        let h2 = WorldHash("bbb".into());
        cc.submit_witness(2, Uuid::new_v4(), h1.clone()).await.unwrap();
        cc.submit_witness(2, Uuid::new_v4(), h1.clone()).await.unwrap();
        cc.submit_witness(2, Uuid::new_v4(), h2).await.unwrap();
        let r = cc.get_round(2).await.unwrap();
        assert!(matches!(r.result, Some(ConsensusResult::Majority { .. })));
    }
}
