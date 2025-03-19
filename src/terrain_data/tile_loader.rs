use crate::terrain_data::{AttachmentData, AttachmentFormat, AttachmentTile, TileAtlas};
use bevy::{
    asset::{AssetServer, Assets, Handle},
    image::Image,
    prelude::*,
};
use slab::Slab;

struct LoadingTile {
    handle: Handle<Image>,
    tile: AttachmentTile,
    format: AttachmentFormat,
}

#[derive(Component)]
pub struct DefaultLoader {
    loading_tiles: Slab<LoadingTile>,
}

impl Default for DefaultLoader {
    fn default() -> Self {
        Self {
            loading_tiles: Slab::with_capacity(32),
        }
    }
}

impl DefaultLoader {
    fn to_load_next(&self, tiles: &mut Vec<AttachmentTile>) -> Option<AttachmentTile> {
        // Todo: tile prioritization goes here
        tiles.pop()
    }

    fn finish_loading(
        &mut self,
        atlas: &mut TileAtlas,
        asset_server: &mut AssetServer,
        images: &mut Assets<Image>,
    ) {
        self.loading_tiles.retain(|_, tile| {
            if asset_server.is_loaded(tile.handle.id()) {
                let image = images.get(tile.handle.id()).unwrap();
                let data = AttachmentData::from_bytes(&image.data, tile.format);
                atlas.tile_loaded(tile.tile.clone(), data);

                false
            } else if asset_server.load_state(tile.handle.id()).is_failed() {
                return false;
            } else {
                true
            }
        });
    }

    fn start_loading(&mut self, atlas: &mut TileAtlas, asset_server: &mut AssetServer) {
        while self.loading_tiles.len() < self.loading_tiles.capacity() {
            if let Some(tile) = self.to_load_next(&mut atlas.to_load) {
                let attachment = &atlas.attachments[&tile.label];

                let path = tile
                    .coordinate
                    .path(&attachment.path.join(String::from(&tile.label)));

                self.loading_tiles.insert(LoadingTile {
                    handle: asset_server.load(path),
                    tile,
                    format: attachment.format,
                });
            } else {
                break;
            }
        }
    }
}

pub fn finish_loading(
    mut terrains: Query<(&mut TileAtlas, &mut DefaultLoader)>,
    mut asset_server: ResMut<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    for (mut tile_atlas, mut loader) in &mut terrains {
        loader.finish_loading(&mut tile_atlas, &mut asset_server, &mut images);
    }
}

pub fn start_loading(
    mut terrains: Query<(&mut TileAtlas, &mut DefaultLoader)>,
    mut asset_server: ResMut<AssetServer>,
) {
    for (mut tile_atlas, mut loader) in &mut terrains {
        loader.start_loading(&mut tile_atlas, &mut asset_server);
    }
}
