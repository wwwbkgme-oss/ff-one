//! 3-D integer coordinates and chunk addressing.

use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Add, Sub},
};

/// Integer voxel coordinate in world space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Position3D {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Position3D {
    pub const ORIGIN: Self = Self { x: 0, y: 0, z: 0 };

    #[inline]
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// Chunk address for this position given a chunk side length.
    #[inline]
    pub fn chunk_pos(&self, cs: i64) -> ChunkPos {
        ChunkPos::new(
            self.x.div_euclid(cs),
            self.y.div_euclid(cs),
            self.z.div_euclid(cs),
        )
    }

    /// Position within its chunk (local coordinates, always non-negative).
    #[inline]
    pub fn local_pos(&self, cs: i64) -> Self {
        Self::new(
            self.x.rem_euclid(cs),
            self.y.rem_euclid(cs),
            self.z.rem_euclid(cs),
        )
    }

    pub fn manhattan_distance(&self, o: &Self) -> u64 {
        self.x.abs_diff(o.x) + self.y.abs_diff(o.y) + self.z.abs_diff(o.z)
    }
}

impl fmt::Display for Position3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Add for Position3D {
    type Output = Self;
    fn add(self, r: Self) -> Self {
        Self::new(self.x + r.x, self.y + r.y, self.z + r.z)
    }
}

impl Sub for Position3D {
    type Output = Self;
    fn sub(self, r: Self) -> Self {
        Self::new(self.x - r.x, self.y - r.y, self.z - r.z)
    }
}

/// Chunk-grid address — one step coarser than [`Position3D`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ChunkPos {
    pub cx: i64,
    pub cy: i64,
    pub cz: i64,
}

impl ChunkPos {
    #[inline]
    pub fn new(cx: i64, cy: i64, cz: i64) -> Self {
        Self { cx, cy, cz }
    }

    /// World-space origin of this chunk.
    #[inline]
    pub fn world_origin(&self, cs: i64) -> Position3D {
        Position3D::new(self.cx * cs, self.cy * cs, self.cz * cs)
    }
}

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chunk({}, {}, {})", self.cx, self.cy, self.cz)
    }
}
