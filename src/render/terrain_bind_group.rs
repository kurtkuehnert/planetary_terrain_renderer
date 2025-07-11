use crate::{
    terrain::TerrainComponents,
    terrain_data::{GpuAttachment, GpuTileAtlas, TileAtlas},
    util::GpuBuffer,
};
use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{SystemParamItem, lifetimeless::SRes},
    },
    math::Affine3,
    prelude::*,
    render::{
        Extract,
        render_asset::RenderAssets,
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::*,
        renderer::RenderDevice,
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::FallbackImage,
    },
};
use std::array;

// Todo: use this once texture views can be used directly
#[derive(AsBindGroup)]
pub struct TerrainBindGroup {
    #[storage(0, visibility(all), read_only, buffer)]
    terrain: Buffer,
    #[uniform(1, visibility(all))]
    attachments: AttachmentUniform,
    #[sampler(2, visibility(all))]
    #[texture(3, visibility(all), dimension = "2d_array")]
    attachment0: Handle<Image>,
    #[texture(4, visibility(all), dimension = "2d_array")]
    attachment1: Handle<Image>,
    #[texture(5, visibility(all), dimension = "2d_array")]
    attachment2: Handle<Image>,
    #[texture(6, visibility(all), dimension = "2d_array")]
    attachment3: Handle<Image>,
    #[texture(7, visibility(all), dimension = "2d_array")]
    attachment4: Handle<Image>,
    #[texture(8, visibility(all), dimension = "2d_array")]
    attachment5: Handle<Image>,
    #[texture(9, visibility(all), dimension = "2d_array")]
    attachment6: Handle<Image>,
    #[texture(10, visibility(all), dimension = "2d_array")]
    attachment7: Handle<Image>,
}

#[derive(Default, ShaderType)]
struct AttachmentConfig {
    texture_size: f32,
    center_size: f32,
    scale: f32,
    offset: f32,
    mask: u32,
    padding1: u32,
    padding2: u32,
    padding3: u32,
}

impl AttachmentConfig {
    fn new(attachment: &GpuAttachment) -> Self {
        Self {
            center_size: attachment.buffer_info.center_size as f32,
            texture_size: attachment.buffer_info.texture_size as f32,
            scale: attachment.buffer_info.center_size as f32
                / attachment.buffer_info.texture_size as f32,
            offset: attachment.buffer_info.border_size as f32
                / attachment.buffer_info.texture_size as f32,
            mask: attachment.buffer_info.mask as u32,
            padding1: 0,
            padding2: 0,
            padding3: 0,
        }
    }
}

#[derive(Default, ShaderType)]
struct AttachmentUniform {
    attachments: [AttachmentConfig; 8],
}

impl AttachmentUniform {
    fn new(tile_atlas: &GpuTileAtlas) -> Self {
        Self {
            attachments: array::from_fn(|i| {
                tile_atlas
                    .attachments
                    .iter()
                    .find(|(_, attachment)| attachment.index == i)
                    .map_or(AttachmentConfig::default(), |(_, attachment)| {
                        AttachmentConfig::new(attachment)
                    })
            }),
        }
    }
}

/// The terrain config data that is available in shaders.
#[derive(Default, ShaderType)]
pub struct TerrainUniform {
    lod_count: u32,
    scale: Vec3,
    min_height: f32,
    max_height: f32,
    height_scale: f32,
    world_from_local: [Vec4; 3],
    local_from_world_transpose_a: [Vec4; 2],
    local_from_world_transpose_b: f32,
}

impl TerrainUniform {
    pub fn new(tile_atlas: &TileAtlas, global_transform: &GlobalTransform) -> Self {
        let transform = Affine3::from(&global_transform.affine());
        let world_from_local = transform.to_transpose();
        let (local_from_world_transpose_a, local_from_world_transpose_b) =
            transform.inverse_transpose_3x3();

        Self {
            lod_count: tile_atlas.lod_count,
            scale: tile_atlas.shape.scale().as_vec3(),
            min_height: tile_atlas.min_height * tile_atlas.height_scale,
            max_height: tile_atlas.max_height * tile_atlas.height_scale,
            height_scale: tile_atlas.height_scale,
            world_from_local,
            local_from_world_transpose_a,
            local_from_world_transpose_b,
        }
    }
}

pub struct GpuTerrain {
    pub(crate) terrain_bind_group: Option<BindGroup>,

    terrain_buffer: Handle<ShaderStorageBuffer>,
    atlas_sampler: Sampler,
    attachment_textures: [TextureView; 8],
    attachment_buffer: GpuBuffer<AttachmentUniform>,
}

impl GpuTerrain {
    fn new(
        device: &RenderDevice,
        fallback_image: &FallbackImage,
        tile_atlas: &TileAtlas,
        gpu_tile_atlas: &GpuTileAtlas,
    ) -> Self {
        let attachment_buffer = GpuBuffer::create(
            device,
            &AttachmentUniform::new(gpu_tile_atlas),
            BufferUsages::UNIFORM,
        );

        let attachment_textures = array::from_fn(|i| {
            gpu_tile_atlas
                .attachments
                .iter()
                .find(|(_, attachment)| attachment.index == i)
                .map_or(
                    fallback_image.d2_array.texture_view.clone(),
                    |(_, attachment)| {
                        attachment
                            .atlas_texture
                            .create_view(&TextureViewDescriptor {
                                format: Some(attachment.buffer_info.format.render_format()),
                                usage: Some(TextureUsages::TEXTURE_BINDING),
                                ..default()
                            })
                    },
                )
        });

        let atlas_sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            anisotropy_clamp: 16, // Todo: make this customisable
            ..default()
        });

        Self {
            terrain_buffer: tile_atlas.terrain_buffer.clone(),
            attachment_buffer,
            atlas_sampler,
            attachment_textures,
            terrain_bind_group: None,
        }
    }

    pub(crate) fn initialize(
        device: Res<RenderDevice>,
        fallback_image: Res<FallbackImage>,
        mut gpu_terrains: ResMut<TerrainComponents<GpuTerrain>>,
        gpu_tile_atlases: Res<TerrainComponents<GpuTileAtlas>>,
        tile_atlases: Extract<Query<(Entity, &TileAtlas), Added<TileAtlas>>>,
    ) {
        for (terrain, tile_atlas) in &tile_atlases {
            let gpu_tile_atlas = &gpu_tile_atlases[&terrain];

            gpu_terrains.insert(
                terrain,
                GpuTerrain::new(&device, &fallback_image, tile_atlas, gpu_tile_atlas),
            );
        }
    }

    pub(crate) fn prepare(
        device: Res<RenderDevice>,
        buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
        mut gpu_terrains: ResMut<TerrainComponents<GpuTerrain>>,
    ) {
        for gpu_terrain in &mut gpu_terrains.values_mut() {
            let terrain_buffer = buffers.get(&gpu_terrain.terrain_buffer).unwrap();

            // Todo: be smarter about bind group recreation
            gpu_terrain.terrain_bind_group = Some(device.create_bind_group(
                "terrain_bind_group",
                &TerrainBindGroup::bind_group_layout(&device),
                &BindGroupEntries::sequential((
                    terrain_buffer.buffer.as_entire_binding(),
                    &gpu_terrain.attachment_buffer,
                    &gpu_terrain.atlas_sampler,
                    &gpu_terrain.attachment_textures[0],
                    &gpu_terrain.attachment_textures[1],
                    &gpu_terrain.attachment_textures[2],
                    &gpu_terrain.attachment_textures[3],
                    &gpu_terrain.attachment_textures[4],
                    &gpu_terrain.attachment_textures[5],
                    &gpu_terrain.attachment_textures[6],
                    &gpu_terrain.attachment_textures[7],
                )),
            ));
        }
    }
}

pub struct SetTerrainBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetTerrainBindGroup<I> {
    type Param = SRes<TerrainComponents<GpuTerrain>>;
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        gpu_terrains: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_terrain = &gpu_terrains.into_inner()[&item.main_entity()];

        if let Some(bind_group) = &gpu_terrain.terrain_bind_group {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Skip
        }
    }
}
