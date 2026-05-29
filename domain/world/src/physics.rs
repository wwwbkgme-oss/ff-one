//! Tick-basierte Voxel-Physik.
//!
//! **BKG:** reine Domänenlogik — deterministisch, kein I/O.
//! Gravitation (Sand/Kies), Fluidausbreitung (Wasser/Lava), Explosionen.

use contracts::events::WorldEvent;
use types::{
    block::{Block, Material},
    position::Position3D,
    world::{WorldState, CHUNK_SIZE},
};
use contracts::error::FfError;

/// Zustandsloser Physik-Prozessor.
pub struct PhysicsEngine;

impl PhysicsEngine {
    pub fn new() -> Self {
        Self
    }

    /// Führt einen Physik-Tick aus — mutiert `WorldState`, gibt Events zurück.
    pub fn tick(&self, world: &mut WorldState) -> Vec<WorldEvent> {
        let keys: Vec<String> = world.chunks.keys().cloned().collect();
        let cs = CHUNK_SIZE as usize;
        for key in &keys {
            let chunk = match world.chunks.get(key) {
                Some(c) => c.clone(),
                None    => continue,
            };
            for ly in (0..cs).rev() {
                for lx in 0..cs {
                    for lz in 0..cs {
                        match chunk.get(lx, ly, lz).material {
                            Material::Sand | Material::Gravel => self.gravity(world, &chunk, lx, ly, lz),
                            Material::Water | Material::Lava  => self.fluid(world, &chunk, lx, ly, lz),
                            _ => {}
                        }
                    }
                }
            }
        }
        vec![]
    }

    /// Explosion: zerstört Blöcke innerhalb des Radius (außer Obsidian).
    pub fn explode(&self, world: &mut WorldState, center: Position3D, radius: u32, tick: u64) -> WorldEvent {
        let r = radius as i64;
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if (dx * dx + dy * dy + dz * dz) as f64 <= (r * r) as f64 {
                        let pos = center + Position3D::new(dx, dy, dz);
                        if let Ok(b) = self.get_block(world, &pos) {
                            if b.material != Material::Obsidian && !b.is_air() {
                                let _ = self.set_block(world, &pos, Block::AIR, tick);
                            }
                        }
                    }
                }
            }
        }
        WorldEvent::ExplosionAt { center, radius, tick }
    }

    // ── Private Helfer ────────────────────────────────────────────────────

    fn gravity(&self, w: &mut WorldState, c: &types::world::Chunk, lx: usize, ly: usize, lz: usize) {
        if ly == 0 || !c.get(lx, ly - 1, lz).is_air() {
            return;
        }
        let block = *c.get(lx, ly, lz);
        let o = c.pos.world_origin(CHUNK_SIZE);
        let pos   = Position3D::new(o.x + lx as i64, o.y + ly as i64, o.z + lz as i64);
        let below = Position3D::new(pos.x, pos.y - 1, pos.z);
        let _ = self.set_block(w, &pos, Block::AIR, 0);
        let _ = self.set_block(w, &below, block, 0);
    }

    fn fluid(&self, w: &mut WorldState, c: &types::world::Chunk, lx: usize, ly: usize, lz: usize) {
        if ly == 0 { return; }
        let fluid = *c.get(lx, ly, lz);
        let o = c.pos.world_origin(CHUNK_SIZE);
        let pos = Position3D::new(o.x + lx as i64, o.y + ly as i64, o.z + lz as i64);
        let below = Position3D::new(pos.x, pos.y - 1, pos.z);
        if let Ok(b) = self.get_block(w, &below) {
            if b.is_air() {
                let _ = self.set_block(w, &below, fluid, 0);
                return;
            }
        }
        for dir in [
            Position3D::new(1, 0, 0), Position3D::new(-1, 0, 0),
            Position3D::new(0, 0, 1), Position3D::new(0, 0, -1),
        ] {
            let n = pos + dir;
            if let Ok(b) = self.get_block(w, &n) {
                if b.is_air() {
                    let _ = self.set_block(w, &n, fluid, 0);
                    break;
                }
            }
        }
    }

    fn get_block(&self, w: &WorldState, pos: &Position3D) -> Result<Block, FfError> {
        let cp = pos.chunk_pos(CHUNK_SIZE);
        let lp = pos.local_pos(CHUNK_SIZE);
        w.get_chunk(&cp)
            .ok_or_else(|| FfError::ChunkNotFound(WorldState::chunk_key(&cp)))
            .map(|c| *c.get(lp.x as usize, lp.y as usize, lp.z as usize))
    }

    fn set_block(&self, w: &mut WorldState, pos: &Position3D, block: Block, tick: u64) -> Result<(), FfError> {
        let cp = pos.chunk_pos(CHUNK_SIZE);
        let lp = pos.local_pos(CHUNK_SIZE);
        w.get_chunk_mut(&cp)
            .ok_or_else(|| FfError::ChunkNotFound(WorldState::chunk_key(&cp)))
            .map(|c| c.set(lp.x as usize, lp.y as usize, lp.z as usize, block, tick))
    }
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}
