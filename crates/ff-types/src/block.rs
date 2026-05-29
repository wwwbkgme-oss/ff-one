//! Voxel material and block types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Physical material of a single voxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Material {
    Air,
    Stone,
    Dirt,
    Grass,
    Wood,
    Leaves,
    Sand,
    Gravel,
    Water,
    Lava,
    Iron,
    Gold,
    Diamond,
    Obsidian,
    Glass,
    Planks,
    Bricks,
    /// Plugin-registered material.
    Custom(u16),
}

impl Material {
    pub fn is_passable(self) -> bool {
        matches!(self, Self::Air | Self::Water)
    }

    pub fn hardness(self) -> f32 {
        match self {
            Self::Air => 0.0,
            Self::Grass | Self::Dirt | Self::Sand | Self::Gravel => 0.5,
            Self::Wood | Self::Leaves | Self::Planks => 2.0,
            Self::Stone | Self::Bricks => 3.0,
            Self::Iron => 5.0,
            Self::Gold => 3.0,
            Self::Diamond => 10.0,
            Self::Obsidian => 50.0,
            _ => 1.0,
        }
    }

    /// Economy value in gold per unit.
    pub fn resource_value(self) -> u64 {
        match self {
            Self::Air => 0,
            Self::Dirt | Self::Gravel => 1,
            Self::Grass | Self::Sand | Self::Leaves => 2,
            Self::Stone | Self::Wood | Self::Planks | Self::Glass => 5,
            Self::Bricks => 8,
            Self::Iron => 15,
            Self::Gold => 25,
            Self::Diamond => 100,
            Self::Obsidian => 40,
            _ => 3,
        }
    }
}

impl fmt::Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(id) => write!(f, "custom_{id}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

/// A single voxel — material plus optional metadata byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub material: Material,
    /// Orientation, growth stage, or other plugin metadata.
    pub meta: u8,
}

impl Block {
    pub const AIR: Self = Self { material: Material::Air, meta: 0 };

    #[inline]
    pub fn new(material: Material) -> Self {
        Self { material, meta: 0 }
    }

    #[inline]
    pub fn is_air(self) -> bool {
        self.material == Material::Air
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::AIR
    }
}
