//! This module contains the implementation of the Uniform Distance-Dependent Level of Detail (UDLOD).
//!
//! This algorithm is responsible for approximating the terrain geometry.
//! Therefore tiny mesh tiles are refined in a tile_tree-like manner in a compute shader prepass for
//! each view. Then they are drawn using a single draw indirect call and morphed together to form
//! one continuous surface.

mod terrain_bind_group;
mod terrain_material;
mod terrain_pass;
mod terrain_view_bind_group;
mod tiling_prepass;

pub use self::{
    terrain_bind_group::GpuTerrain,
    terrain_material::TerrainMaterialPlugin,
    terrain_view_bind_group::{GpuTerrainView, TerrainViewBindGroup},
};

pub(crate) use self::{
    terrain_bind_group::*, terrain_pass::*, terrain_view_bind_group::*, tiling_prepass::*,
};
