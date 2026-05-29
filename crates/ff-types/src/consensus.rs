//! BLAKE3 consensus protocol types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// BLAKE3 digest of a world-state snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldHash(pub String);

impl WorldHash {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(blake3::hash(data).to_hex().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for WorldHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One agent's attestation of a world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessRecord {
    pub id:           Uuid,
    pub agent_id:     Uuid,
    pub tick:         u64,
    pub world_hash:   WorldHash,
    pub witnessed_at: DateTime<Utc>,
    pub signature:    Option<String>,
}

impl WitnessRecord {
    pub fn new(agent_id: Uuid, tick: u64, world_hash: WorldHash) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            tick,
            world_hash,
            witnessed_at: Utc::now(),
            signature: None,
        }
    }
}

/// Outcome of a consensus round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusResult {
    /// All witnesses agree.
    Unanimous(WorldHash),
    /// Supermajority (≥ ⅔) agrees.
    Majority { hash: WorldHash, votes: u32, total: u32 },
    /// No supermajority reached.
    Disputed,
}

/// State of an ongoing consensus round for one tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub tick:               u64,
    pub witnesses:          Vec<WitnessRecord>,
    pub required_witnesses: u32,
    pub result:             Option<ConsensusResult>,
    pub started_at:         DateTime<Utc>,
    pub finalised_at:       Option<DateTime<Utc>>,
}

impl ConsensusRound {
    pub fn new(tick: u64, required_witnesses: u32) -> Self {
        Self {
            tick,
            witnesses: Vec::new(),
            required_witnesses,
            result: None,
            started_at: Utc::now(),
            finalised_at: None,
        }
    }

    pub fn add_witness(&mut self, record: WitnessRecord) {
        self.witnesses.push(record);
        if self.witnesses.len() >= self.required_witnesses as usize {
            self.try_finalise();
        }
    }

    fn try_finalise(&mut self) {
        let mut counts: HashMap<&str, u32> = HashMap::new();
        for w in &self.witnesses {
            *counts.entry(w.world_hash.as_str()).or_insert(0) += 1;
        }
        let total = self.witnesses.len() as u32;
        if let Some((hash, &votes)) = counts.iter().max_by_key(|(_, &v)| v) {
            self.result = Some(if votes == total {
                ConsensusResult::Unanimous(WorldHash(hash.to_string()))
            } else if votes * 3 >= total * 2 {
                ConsensusResult::Majority {
                    hash: WorldHash(hash.to_string()),
                    votes,
                    total,
                }
            } else {
                ConsensusResult::Disputed
            });
            self.finalised_at = Some(Utc::now());
        }
    }

    pub fn is_finalised(&self) -> bool {
        self.result.is_some()
    }
}
