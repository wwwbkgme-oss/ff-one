//! # world
//!
//! **BKG-Layer:** `domain`
//!
//! **Einzige Verantwortung:** deterministischer Voxel-Weltsimulator.
//!
//! Enthält:
//! - [`generator`] — Terrain-Generierung via geschichteter Perlin-Noise  
//! - [`physics`] — Tick-basierte Physik (Gravitation, Fluide, Explosionen)  
//! - [`mesher`] — Greedy-Meshing für den Renderer  
//! - [`simulator`] — Implementierung des [`core::traits::WorldSimulator`]-Traits

pub mod generator;
pub mod mesher;
pub mod physics;
pub mod simulator;

pub use generator::WorldGenerator;
pub use mesher::{GreedyMesher, Mesh, Quad};
pub use physics::PhysicsEngine;
pub use simulator::VoxelSimulator;
