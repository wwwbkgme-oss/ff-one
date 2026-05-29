//! Deterministischer Terrain-Generator.
//!
//! Gleiches `seed` → identische Welt auf jedem Knoten (Replay-Safety).
//! Kein Wallclock-Zeit-Aufruf, keine globalen mutable States.

use noise::{NoiseFn, Perlin, Turbulence};
use types::{
    block::{Block, Material},
    position::ChunkPos,
    world::{BiomeType, Chunk, WorldState, CHUNK_SIZE},
};

/// Konfiguration der Welt-Generierung.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Amplitude der Terrain-Höhe in Blöcken.
    pub terrain_amplitude: f64,
    /// Meeresspiegeltiefe in Blöcken.
    pub sea_level: i64,
    /// Tiefe der Bedrock-Schicht.
    pub bedrock_depth: i64,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            terrain_amplitude: 40.0,
            sea_level: 32,
            bedrock_depth: 4,
        }
    }
}

/// Zustandsloser Terrain-Generator — deterministisch für gegebenes `seed`.
pub struct WorldGenerator {
    config: GeneratorConfig,
    height: Turbulence<Perlin, Perlin>,
    biome: Perlin,
    cave: Perlin,
}

impl WorldGenerator {
    pub fn new(seed: u64, config: GeneratorConfig) -> Self {
        let s = (seed & 0xffff_ffff) as u32;
        let base = Perlin::new(s);
        let height = Turbulence::<Perlin, Perlin>::new(base)
            .set_frequency(1.0)
            .set_power(0.5);
        Self {
            config,
            height,
            biome: Perlin::new(s.wrapping_add(1)),
            cave: Perlin::new(s.wrapping_add(2)),
        }
    }

    /// Generiert einen einzelnen Chunk deterministisch.
    pub fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        let biome = self.sample_biome(pos.cx, pos.cz);
        let mut chunk = Chunk::new(pos, biome);
        let cs = CHUNK_SIZE as usize;

        for lx in 0..cs {
            for lz in 0..cs {
                let wx = pos.cx * CHUNK_SIZE + lx as i64;
                let wz = pos.cz * CHUNK_SIZE + lz as i64;
                let surf = self.surface_height(wx, wz);
                for ly in 0..cs {
                    let wy = pos.cy * CHUNK_SIZE + ly as i64;
                    chunk.set(lx, ly, lz, self.block_at(wx, wy, wz, surf, biome), 0);
                }
            }
        }
        chunk
    }

    /// Befüllt ein `WorldState` mit Chunks innerhalb des Sichtradius.
    pub fn populate(&self, world: &mut WorldState, view: i64) {
        for cx in -view..=view {
            for cz in -view..=view {
                for cy in -2..=8 {
                    let pos = ChunkPos::new(cx, cy, cz);
                    if world.get_chunk(&pos).is_none() {
                        world.insert_chunk(self.generate_chunk(pos));
                    }
                }
            }
        }
    }

    // ── Private Hilfsmethoden ─────────────────────────────────────────────

    fn surface_height(&self, wx: i64, wz: i64) -> i64 {
        let raw = self.height.get([wx as f64 / 128.0, wz as f64 / 128.0]);
        self.config.sea_level + (raw * self.config.terrain_amplitude) as i64
    }

    fn is_cave(&self, wx: i64, wy: i64, wz: i64) -> bool {
        self.cave
            .get([wx as f64 / 32.0, wy as f64 / 16.0, wz as f64 / 32.0])
            > 0.6
    }

    fn sample_biome(&self, cx: i64, cz: i64) -> BiomeType {
        let v = self.biome.get([cx as f64 / 8.0, cz as f64 / 8.0]);
        if v < -0.6 {
            BiomeType::Ocean
        } else if v < -0.3 {
            BiomeType::Desert
        } else if v < 0.0 {
            BiomeType::Plains
        } else if v < 0.3 {
            BiomeType::Forest
        } else if v < 0.6 {
            BiomeType::Mountains
        } else {
            BiomeType::Tundra
        }
    }

    fn block_at(&self, wx: i64, wy: i64, wz: i64, surf: i64, biome: BiomeType) -> Block {
        if wy < self.config.bedrock_depth {
            return Block::new(Material::Obsidian);
        }
        if wy > surf {
            return if wy <= self.config.sea_level && biome == BiomeType::Ocean {
                Block::new(Material::Water)
            } else {
                Block::AIR
            };
        }
        if self.is_cave(wx, wy, wz) && wy < surf - 2 {
            return Block::AIR;
        }
        if wy == surf {
            return match biome {
                BiomeType::Desert => Block::new(Material::Sand),
                BiomeType::Ocean => Block::new(Material::Gravel),
                BiomeType::VolcanicWastes => Block::new(Material::Obsidian),
                _ => Block::new(Material::Grass),
            };
        }
        if wy >= surf - 3 {
            return if biome == BiomeType::Desert {
                Block::new(Material::Sand)
            } else {
                Block::new(Material::Dirt)
            };
        }
        // Erzadern
        let ore = self.cave.get([
            wx as f64 / 8.0 + 100.0,
            wy as f64 / 8.0,
            wz as f64 / 8.0 + 100.0,
        ]);
        if wy < 16 && ore > 0.70 {
            return Block::new(Material::Diamond);
        }
        if wy < 32 && ore > 0.65 {
            return Block::new(Material::Gold);
        }
        if wy < 48 && ore > 0.60 {
            return Block::new(Material::Iron);
        }
        Block::new(Material::Stone)
    }
}
