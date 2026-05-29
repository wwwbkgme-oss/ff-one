//! World state, chunks, biomes, and epochs.

use crate::{block::Block, position::ChunkPos};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Side length of one chunk in voxels (16³ = 4 096 blocks).
pub const CHUNK_SIZE: i64 = 16;

/// Biome classification for a chunk region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Plains,
    Forest,
    Desert,
    Mountains,
    Tundra,
    Swamp,
    Ocean,
    VolcanicWastes,
    CrimsonForest,
    /// Plugin-registered biome.
    Custom(u16),
}

impl std::fmt::Display for BiomeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(id) => write!(f, "custom_biome_{id}"),
            Self::VolcanicWastes => write!(f, "Volcanic Wastes"),
            Self::CrimsonForest => write!(f, "Crimson Forest"),
            _ => write!(f, "{self:?}"),
        }
    }
}

/// A 16³ block of voxel data.
///
/// Blocks are stored in a flat array indexed by `x + 16*(y + 16*z)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub pos:                ChunkPos,
    pub biome:              BiomeType,
    pub blocks:             Vec<Block>,
    pub last_modified_tick: u64,
}

impl Chunk {
    pub fn new(pos: ChunkPos, biome: BiomeType) -> Self {
        let vol = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;
        Self { pos, biome, blocks: vec![Block::AIR; vol], last_modified_tick: 0 }
    }

    #[inline]
    fn idx(x: usize, y: usize, z: usize) -> usize {
        x + 16 * (y + 16 * z)
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[Self::idx(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block, tick: u64) {
        self.blocks[Self::idx(x, y, z)] = block;
        self.last_modified_tick = tick;
    }
}

/// Metadata for one simulation epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch {
    pub number:            u64,
    pub seed:              u64,
    pub started_at_tick:   u64,
    pub ended_at_tick:     Option<u64>,
    pub dominant_faction:  Option<String>,
}

/// Zone-control record (0–100% per faction).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneControl {
    pub zone_name: String,
    /// Faction name → ownership percentage.
    pub control: HashMap<String, u8>,
}

/// Complete snapshot of the voxel universe at one tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub id:           Uuid,
    pub seed:         u64,
    pub tick:         u64,
    pub epoch:        Epoch,
    pub chunks:       HashMap<String, Chunk>,
    pub zone_control: Vec<ZoneControl>,
    /// BLAKE3 hash of the serialised chunk data (set by the consensus layer).
    pub state_hash:   String,
}

impl WorldState {
    pub fn new(seed: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            seed,
            tick: 0,
            epoch: Epoch {
                number: 0,
                seed,
                started_at_tick: 0,
                ended_at_tick: None,
                dominant_faction: None,
            },
            chunks: HashMap::new(),
            zone_control: Vec::new(),
            state_hash: String::new(),
        }
    }

    pub fn chunk_key(pos: &ChunkPos) -> String {
        format!("{},{},{}", pos.cx, pos.cy, pos.cz)
    }

    pub fn get_chunk(&self, pos: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&Self::chunk_key(pos))
    }

    pub fn get_chunk_mut(&mut self, pos: &ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&Self::chunk_key(pos))
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(Self::chunk_key(&chunk.pos), chunk);
    }
}
