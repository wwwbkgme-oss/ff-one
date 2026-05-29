//! Greedy Meshing — reduziert Chunk-Flächen auf minimale Quads.
//!
//! Deterministisch — gleicher Chunk → gleiche Mesh.

use types::block::Material;
use types::world::{Chunk, CHUNK_SIZE};

/// Ein einzelnes rechteckiges Flächenelement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quad {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub w: usize,
    pub h: usize,
    pub material: Material,
    /// Flächennormale: 0=+X 1=−X 2=+Y 3=−Y 4=+Z 5=−Z.
    pub face: u8,
}

/// Vollständig gebautes Mesh eines Chunks.
#[derive(Debug, Default)]
pub struct Mesh {
    pub quads: Vec<Quad>,
}

impl Mesh {
    pub fn quad_count(&self) -> usize {
        self.quads.len()
    }
}

/// Zustandsloser Greedy-Mesher.
pub struct GreedyMesher;

impl GreedyMesher {
    pub fn new() -> Self {
        Self
    }

    pub fn mesh(&self, chunk: &Chunk) -> Mesh {
        let mut quads = Vec::new();
        for face in 0u8..6 {
            quads.extend(self.mesh_axis(chunk, face));
        }
        Mesh { quads }
    }

    fn mesh_axis(&self, chunk: &Chunk, face: u8) -> Vec<Quad> {
        let cs = CHUNK_SIZE as usize;
        let (da, ua, va) = match face / 2 {
            0 => (0, 1, 2),
            1 => (1, 0, 2),
            _ => (2, 0, 1),
        };
        let positive = face % 2 == 0;
        let mut quads = Vec::new();

        for d in 0..cs {
            let mut mask = vec![None::<Material>; cs * cs];
            for v in 0..cs {
                for u in 0..cs {
                    let mut c = [0usize; 3];
                    c[da] = d;
                    c[ua] = u;
                    c[va] = v;
                    let b = chunk.get(c[0], c[1], c[2]);
                    if b.is_air() {
                        continue;
                    }
                    let vis = if positive {
                        if d + 1 < cs {
                            let mut a = c;
                            a[da] = d + 1;
                            chunk.get(a[0], a[1], a[2]).is_air()
                        } else {
                            true
                        }
                    } else if d > 0 {
                        let mut a = c;
                        a[da] = d - 1;
                        chunk.get(a[0], a[1], a[2]).is_air()
                    } else {
                        true
                    };
                    if vis {
                        mask[v * cs + u] = Some(b.material);
                    }
                }
            }
            quads.extend(self.merge(&mask, cs, d, da, ua, va, face));
        }
        quads
    }

    fn merge(
        &self,
        mask: &[Option<Material>],
        cs: usize,
        d: usize,
        da: usize,
        ua: usize,
        va: usize,
        face: u8,
    ) -> Vec<Quad> {
        let mut quads = Vec::new();
        let mut done = vec![false; cs * cs];

        for v in 0..cs {
            for u in 0..cs {
                let i = v * cs + u;
                if done[i] || mask[i].is_none() {
                    continue;
                }
                let mat = mask[i].unwrap();
                let mut w = 1;
                while u + w < cs && !done[v * cs + u + w] && mask[v * cs + u + w] == Some(mat) {
                    w += 1;
                }
                let mut h = 1;
                'outer: while v + h < cs {
                    for k in 0..w {
                        let ti = (v + h) * cs + u + k;
                        if done[ti] || mask[ti] != Some(mat) {
                            break 'outer;
                        }
                    }
                    h += 1;
                }
                for dv in 0..h {
                    for du in 0..w {
                        done[(v + dv) * cs + u + du] = true;
                    }
                }
                let mut c = [0usize; 3];
                c[da] = d;
                c[ua] = u;
                c[va] = v;
                let (qw, qh) = if ua < va { (w, h) } else { (h, w) };
                quads.push(Quad {
                    x: c[0],
                    y: c[1],
                    z: c[2],
                    w: qw,
                    h: qh,
                    material: mat,
                    face,
                });
            }
        }
        quads
    }
}

impl Default for GreedyMesher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{
        block::{Block, Material},
        position::ChunkPos,
        world::{BiomeType, Chunk, CHUNK_SIZE},
    };

    #[test]
    fn solid_chunk_max_6_quads() {
        let cs = CHUNK_SIZE as usize;
        let mut c = Chunk::new(ChunkPos::new(0, 0, 0), BiomeType::Plains);
        for x in 0..cs {
            for y in 0..cs {
                for z in 0..cs {
                    c.set(x, y, z, Block::new(Material::Stone), 0);
                }
            }
        }
        assert!(GreedyMesher::new().mesh(&c).quad_count() <= 6);
    }

    #[test]
    fn empty_chunk_no_quads() {
        let c = Chunk::new(ChunkPos::new(0, 0, 0), BiomeType::Plains);
        assert_eq!(GreedyMesher::new().mesh(&c).quad_count(), 0);
    }
}
