use crate::{
    plugin::TerrainSettings,
    preprocess::{MipPipelineKey, MipPipelines},
    terrain::TerrainComponents,
    terrain_data::{
        AttachmentData, AttachmentLabel, AttachmentTileWithData, GpuAttachment, TileAtlas,
    },
};
use bevy::{
    platform::collections::HashMap,
    prelude::*,
    render::{
        Extract, MainWorld,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
    },
    tasks::{AsyncComputeTaskPool, Task},
};
use std::{iter, mem};

/// Stores the GPU representation of the [`TileAtlas`] (array textures)
/// alongside the data to update it.
///
/// All attachments of newly loaded tiles are copied into their according atlas attachment.
#[derive(Component)]
pub struct GpuTileAtlas {
    /// Stores the atlas attachments of the terrain.
    pub(crate) attachments: HashMap<AttachmentLabel, GpuAttachment>,
    pub(crate) upload_tiles: Vec<AttachmentTileWithData>,
    pub(crate) download_tiles: Vec<Task<AttachmentTileWithData>>,
    pub(crate) is_spherical: bool,
}

impl GpuTileAtlas {
    pub(crate) fn generate_mip(&self, pass: &mut ComputePass, pipeline_cache: &PipelineCache) {
        for attachment in self.attachments.values() {
            let Some(pipeline) = pipeline_cache.get_compute_pipeline(attachment.mip_pipeline)
            else {
                dbg!("Skipped mipmap generation");
                return; // Todo: In case the pipeline has not been loaded yet, but a mip map should be created, we should not skip and clear the mip map generation list
            };

            pass.set_pipeline(pipeline);

            for bind_groups in &attachment.mip_bind_groups {
                for bind_group in bind_groups {
                    pass.set_bind_group(0, bind_group, &[]);
                    assert_eq!(
                        attachment.buffer_info.texture_size % 8,
                        0,
                        "Currently mipmap generation assumes the texture size to be a multiple of eight."
                    );
                    pass.dispatch_workgroups(
                        attachment.buffer_info.texture_size / 8,
                        attachment.buffer_info.texture_size / 8,
                        1,
                    );
                }
            }
        }
    }

    /// Creates a new gpu tile atlas and initializes its attachment textures.
    fn new(device: &RenderDevice, tile_atlas: &TileAtlas, settings: &TerrainSettings) -> Self {
        let attachments = tile_atlas
            .attachments
            .iter()
            .map(|(label, attachment)| {
                (
                    label.clone(),
                    GpuAttachment::new(device, label, attachment, tile_atlas, settings),
                )
            })
            .collect();

        Self {
            attachments,
            upload_tiles: default(),
            download_tiles: default(),
            is_spherical: tile_atlas.shape.is_spherical(),
        }
    }

    /// Initializes the [`GpuTileAtlas`] of newly created terrains.
    pub(crate) fn initialize(
        device: Res<RenderDevice>,
        mut gpu_tile_atlases: ResMut<TerrainComponents<GpuTileAtlas>>,
        mut tile_atlases: Extract<Query<(Entity, &TileAtlas), Added<TileAtlas>>>,
        settings: Extract<Res<TerrainSettings>>,
    ) {
        for (terrain, tile_atlas) in tile_atlases.iter_mut() {
            gpu_tile_atlases.insert(terrain, GpuTileAtlas::new(&device, tile_atlas, &settings));
        }
    }

    /// Extracts the tiles that have finished loading from all [`TileAtlas`]es into the
    /// corresponding [`GpuTileAtlas`]es.
    pub(crate) fn extract(
        mut main_world: ResMut<MainWorld>,
        mut gpu_tile_atlases: ResMut<TerrainComponents<GpuTileAtlas>>,
    ) {
        let mut tile_atlases = main_world.query::<(Entity, &mut TileAtlas)>();

        for (terrain, mut tile_atlas) in tile_atlases.iter_mut(&mut main_world) {
            let gpu_tile_atlas = gpu_tile_atlases.get_mut(&terrain).unwrap();

            mem::swap(
                &mut tile_atlas.uploading_tiles,
                &mut gpu_tile_atlas.upload_tiles,
            );

            for attachment in gpu_tile_atlas.attachments.values_mut() {
                attachment
                    .mips_to_generate
                    .iter_mut()
                    .for_each(|atlas_indices| atlas_indices.clear());
                attachment
                    .mip_bind_groups
                    .iter_mut()
                    .for_each(|bind_groups| bind_groups.clear());
            }

            for tile in &gpu_tile_atlas.upload_tiles {
                let attachment = gpu_tile_atlas.attachments.get_mut(&tile.label).unwrap();

                for mip_level in 1..attachment.buffer_info.mip_level_count {
                    attachment.mips_to_generate[mip_level as usize].push(tile.atlas_index);
                }
            }

            tile_atlas
                .downloading_tiles
                .extend(mem::take(&mut gpu_tile_atlas.download_tiles));
        }
    }

    /// Queues the attachments of the tiles that have finished loading to be copied into the
    /// corresponding atlas attachments.
    pub(crate) fn prepare(
        device: Res<RenderDevice>,
        queue: Res<RenderQueue>,
        mip_pipelines: Res<MipPipelines>,
        mut gpu_tile_atlases: ResMut<TerrainComponents<GpuTileAtlas>>,
    ) {
        for gpu_tile_atlas in gpu_tile_atlases.values_mut() {
            for attachment in gpu_tile_atlas.attachments.values_mut() {
                attachment.prepare_mip_bind_groups(&device, &mip_pipelines);
            }

            gpu_tile_atlas.upload_tiles(&queue);
        }
    }

    pub(crate) fn queue(
        pipeline_cache: Res<PipelineCache>,
        mip_pipelines: ResMut<MipPipelines>,
        mut pipelines: ResMut<SpecializedComputePipelines<MipPipelines>>,
        mut gpu_tile_atlases: ResMut<TerrainComponents<GpuTileAtlas>>,
    ) {
        for gpu_tile_atlas in gpu_tile_atlases.values_mut() {
            for attachment in gpu_tile_atlas.attachments.values_mut() {
                attachment.mip_pipeline = pipelines.specialize(
                    &pipeline_cache,
                    &mip_pipelines,
                    MipPipelineKey {
                        format: attachment.buffer_info.format,
                    },
                );
            }
        }
    }

    pub(crate) fn _cleanup(mut gpu_tile_atlases: ResMut<TerrainComponents<GpuTileAtlas>>) {
        for gpu_tile_atlas in gpu_tile_atlases.values_mut() {
            gpu_tile_atlas._start_downloading_tiles();
        }
    }

    fn upload_tiles(&mut self, queue: &RenderQueue) {
        for tile in self.upload_tiles.drain(..) {
            let attachment = &self.attachments[&tile.label];

            queue.write_texture(
                attachment.buffer_info.texture_copy_view(
                    &attachment.atlas_texture,
                    tile.atlas_index,
                    0,
                ),
                tile.data.bytes(),
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(attachment.buffer_info.actual_side_size),
                    rows_per_image: Some(attachment.buffer_info.texture_size),
                },
                attachment.buffer_info.extend_3d(0),
            );
        }
    }

    fn _start_downloading_tiles(&mut self) {
        for attachment in self.attachments.values_mut() {
            let buffer_info = attachment.buffer_info;
            let download_buffers = mem::take(&mut attachment._download_buffers);
            let atlas_write_slots = mem::take(&mut attachment._atlas_write_slots);

            self.download_tiles
                .extend(iter::zip(atlas_write_slots, download_buffers).map(
                    |(tile, download_buffer)| {
                        AsyncComputeTaskPool::get().spawn(async move {
                            let (tx, rx) = async_channel::bounded(1);

                            let buffer_slice = download_buffer.slice(..);

                            buffer_slice.map_async(MapMode::Read, move |_| {
                                tx.try_send(()).unwrap();
                            });

                            rx.recv().await.unwrap();

                            let mut data = buffer_slice.get_mapped_range().to_vec();

                            download_buffer.unmap();
                            drop(download_buffer);

                            if data.len() != buffer_info.actual_tile_size as usize {
                                let actual_side_size = buffer_info.actual_side_size as usize;
                                let aligned_side_size = buffer_info.aligned_side_size as usize;

                                let mut take_offset = aligned_side_size;
                                let mut place_offset = actual_side_size;

                                for _ in 1..buffer_info.texture_size {
                                    data.copy_within(
                                        take_offset..take_offset + aligned_side_size,
                                        place_offset,
                                    );
                                    take_offset += aligned_side_size;
                                    place_offset += actual_side_size;
                                }

                                data.truncate(buffer_info.actual_tile_size as usize);
                            }

                            AttachmentTileWithData {
                                atlas_index: tile.atlas_index,
                                label: tile.label,
                                data: AttachmentData::from_bytes(&data, buffer_info.format),
                            }
                        })
                    },
                ));
        }
    }
}
