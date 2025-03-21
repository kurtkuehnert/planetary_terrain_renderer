//! Types for configuring terrains.
//!

use crate::{
    math::{TerrainShape, TileCoordinate},
    terrain_data::{AttachmentConfig, AttachmentLabel},
};
use bevy::{
    ecs::entity::hash_map::EntityHashMap, platform_support::collections::HashMap, prelude::*,
};
use serde::{Deserialize, Serialize};

/// Resource that stores components that are associated to a terrain entity.
/// This is used to persist components in the render world.
#[derive(Deref, DerefMut, Resource)]
pub struct TerrainComponents<C>(EntityHashMap<C>);

impl<C> Default for TerrainComponents<C> {
    fn default() -> Self {
        Self(default())
    }
}

/// The configuration of a terrain.
///
/// Here you can define all fundamental parameters of the terrain.
#[derive(Serialize, Deserialize, Asset, TypePath, Debug, Clone)]
pub struct TerrainConfig {
    /// The path to the terrain folder inside the assets directory.
    pub path: String,
    pub shape: TerrainShape,
    /// The count of level of detail layers.
    pub lod_count: u32,
    pub min_height: f32,
    pub max_height: f32,
    /// The attachments of the terrain.
    pub attachments: HashMap<AttachmentLabel, AttachmentConfig>,
    /// The tiles of the terrain.
    pub tiles: Vec<TileCoordinate>,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            shape: TerrainShape::Plane { side_length: 1.0 },
            lod_count: 1,
            min_height: 0.0,
            max_height: 1.0,
            path: default(),
            tiles: default(),
            attachments: default(),
        }
    }
}
