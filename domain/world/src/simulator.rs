//! [`WorldSimulator`]-Implementierung — verbindet Generator und Physik-Engine.

use crate::{generator::WorldGenerator, physics::PhysicsEngine};
use async_trait::async_trait;
use contracts::{
    error::{FfError, Result},
    events::WorldEvent,
    traits::WorldSimulator,
};
use types::{
    block::Block,
    consensus::WorldHash,
    position::{ChunkPos, Position3D},
    world::{Chunk, WorldState, CHUNK_SIZE},
};
use uuid::Uuid;

/// Konkrete Implementierung des deterministischen Welt-Simulators.
///
/// Kein I/O, kein HTTP — reine Domänenlogik.
pub struct VoxelSimulator {
    gen:  WorldGenerator,
    phys: PhysicsEngine,
}

impl VoxelSimulator {
    pub fn new(seed: u64) -> Self {
        Self {
            gen:  WorldGenerator::new(seed, Default::default()),
            phys: PhysicsEngine::new(),
        }
    }
}

#[async_trait]
impl WorldSimulator for VoxelSimulator {
    async fn tick(&mut self, world: &mut WorldState) -> Result<Vec<WorldEvent>> {
        world.tick += 1;
        Ok(self.phys.tick(world))
    }

    async fn load_chunk(&mut self, world: &mut WorldState, pos: ChunkPos) -> Result<Chunk> {
        if world.get_chunk(&pos).is_none() {
            world.insert_chunk(self.gen.generate_chunk(pos));
        }
        world.get_chunk(&pos)
            .cloned()
            .ok_or_else(|| FfError::ChunkNotFound(WorldState::chunk_key(&pos)))
    }

    async fn place_block(
        &mut self,
        world: &mut WorldState,
        agent_id: Uuid,
        pos: Position3D,
        block: Block,
    ) -> Result<WorldEvent> {
        let cp   = pos.chunk_pos(CHUNK_SIZE);
        let lp   = pos.local_pos(CHUNK_SIZE);
        let tick = world.tick;

        if world.get_chunk(&cp).is_none() {
            world.insert_chunk(self.gen.generate_chunk(cp));
        }
        world.get_chunk_mut(&cp)
            .ok_or_else(|| FfError::ChunkNotFound(WorldState::chunk_key(&cp)))
            .map(|c| {
                c.set(lp.x as usize, lp.y as usize, lp.z as usize, block, tick);
                WorldEvent::BlockPlaced { agent_id, position: pos, block, tick }
            })
    }

    async fn mine_block(
        &mut self,
        world: &mut WorldState,
        agent_id: Uuid,
        pos: Position3D,
    ) -> Result<WorldEvent> {
        let cp   = pos.chunk_pos(CHUNK_SIZE);
        let lp   = pos.local_pos(CHUNK_SIZE);
        let tick = world.tick;

        let chunk = world.get_chunk_mut(&cp)
            .ok_or_else(|| FfError::ChunkNotFound(WorldState::chunk_key(&cp)))?;
        let was = *chunk.get(lp.x as usize, lp.y as usize, lp.z as usize);
        chunk.set(lp.x as usize, lp.y as usize, lp.z as usize, Block::AIR, tick);
        Ok(WorldEvent::BlockMined { agent_id, position: pos, was, tick })
    }

    fn compute_hash(&self, world: &WorldState) -> WorldHash {
        let mut keys: Vec<&String> = world.chunks.keys().collect();
        keys.sort();

        let mut h = blake3::Hasher::new();
        h.update(&world.tick.to_le_bytes());
        h.update(&world.seed.to_le_bytes());

        for key in keys {
            if let Some(chunk) = world.chunks.get(key) {
                h.update(key.as_bytes());
                for b in &chunk.blocks {
                    let id: u16 = match b.material {
                        types::block::Material::Air      => 0,
                        types::block::Material::Stone    => 1,
                        types::block::Material::Dirt     => 2,
                        types::block::Material::Grass    => 3,
                        types::block::Material::Sand     => 6,
                        types::block::Material::Water    => 8,
                        types::block::Material::Lava     => 9,
                        types::block::Material::Iron     => 10,
                        types::block::Material::Gold     => 11,
                        types::block::Material::Diamond  => 12,
                        types::block::Material::Obsidian => 13,
                        types::block::Material::Custom(x) => x,
                        _                                => 99,
                    };
                    h.update(&id.to_le_bytes());
                    h.update(&[b.meta]);
                }
            }
        }
        WorldHash(h.finalize().to_hex().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn same_seed_same_hash() {
        let seed = 42u64;
        let cp = ChunkPos::new(0, 0, 0);

        let mut s1 = VoxelSimulator::new(seed);
        let mut w1 = WorldState::new(seed);
        s1.load_chunk(&mut w1, cp).await.unwrap();

        let mut s2 = VoxelSimulator::new(seed);
        let mut w2 = WorldState::new(seed);
        s2.load_chunk(&mut w2, cp).await.unwrap();

        assert_eq!(
            s1.compute_hash(&w1),
            s2.compute_hash(&w2),
            "Gleiches Seed muss denselben World-Hash ergeben"
        );
    }
}
