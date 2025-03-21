use crate::{
    plugin::TerrainSettings,
    preprocess::MipPipelines,
    terrain_data::{AttachmentFormat, AttachmentLabel, TileAtlas, attachment::Attachment},
    util::GpuBuffer,
};
use bevy::{
    math::UVec3,
    prelude::default,
    render::{
        render_resource::{binding_types::*, *},
        renderer::RenderDevice,
    },
};
use itertools::Itertools;
use std::{iter, mem};

const COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;

fn align_byte_size(value: u32) -> u32 {
    // only works for non zero values
    value - 1 - (value - 1) % COPY_BYTES_PER_ROW_ALIGNMENT + COPY_BYTES_PER_ROW_ALIGNMENT
}

pub(crate) fn create_attachment_layout(device: &RenderDevice) -> BindGroupLayout {
    device.create_bind_group_layout(
        None,
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<u32>(false), // atlas_write_section
                texture_2d_array(TextureSampleType::Float { filterable: true }), // atlas
                sampler(SamplerBindingType::Filtering), // atlas sampler
                uniform_buffer::<AttachmentMeta>(false), // attachment meta
            ),
        ),
    )
}

#[derive(Clone, Debug, Default)]
pub struct AtlasTileAttachment {
    pub(crate) atlas_index: u32,
    pub(crate) label: AttachmentLabel,
}

#[derive(Default, ShaderType)]
pub(crate) struct AttachmentMeta {
    pub(crate) lod_count: u32,
    pub(crate) texture_size: u32,
    pub(crate) border_size: u32,
    pub(crate) center_size: u32,
    pub(crate) pixels_per_entry: u32,
    pub(crate) entries_per_side: u32,
    pub(crate) entries_per_tile: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AtlasBufferInfo {
    pub(crate) mask: bool,
    lod_count: u32,
    pub(crate) texture_size: u32,
    pub(crate) border_size: u32,
    pub(crate) center_size: u32,
    pub(crate) format: AttachmentFormat,
    pub(crate) mip_level_count: u32,

    pixels_per_entry: u32,

    entries_per_side: u32,
    entries_per_tile: u32,

    pub(crate) actual_side_size: u32,
    pub(crate) aligned_side_size: u32,
    pub(crate) actual_tile_size: u32,
    aligned_tile_size: u32,

    pub(crate) _workgroup_count: UVec3,
}

impl AtlasBufferInfo {
    fn new(attachment: &Attachment, lod_count: u32) -> Self {
        // Todo: adjust this code for pixel sizes larger than 4 byte
        // This approach is currently limited to 1, 2, and 4 byte sized pixels
        // Extending it to 8 and 16 sized pixels should be quite easy.
        // However 3, 6, 12 sized pixels do and will not work!
        // For them to work properly we will need to write into a texture instead of buffer.

        let format = attachment.format;
        let texture_size = attachment.texture_size;
        let border_size = attachment.border_size;
        let center_size = attachment.center_size;
        let mip_level_count = attachment.mip_level_count;

        let pixel_size = format.pixel_size();
        let entry_size = mem::size_of::<u32>() as u32;
        let pixels_per_entry = entry_size / pixel_size;

        let actual_side_size = texture_size * pixel_size;
        let aligned_side_size = align_byte_size(actual_side_size);
        let actual_tile_size = texture_size * actual_side_size;
        let aligned_tile_size = texture_size * aligned_side_size;

        let entries_per_side = aligned_side_size / entry_size;
        let entries_per_tile = texture_size * entries_per_side;

        let workgroup_count = UVec3::new(entries_per_side / 8, texture_size / 8, 1);

        Self {
            mask: attachment.mask,
            lod_count,
            border_size,
            center_size,
            texture_size,
            mip_level_count,
            pixels_per_entry,
            entries_per_side,
            entries_per_tile,
            actual_side_size,
            aligned_side_size,
            actual_tile_size,
            aligned_tile_size,
            format,
            _workgroup_count: workgroup_count,
        }
    }

    pub(crate) fn texture_copy_view<'a>(
        &'a self,
        texture: &'a Texture,
        index: u32,
        mip_level: u32,
    ) -> TexelCopyTextureInfo<'a> {
        TexelCopyTextureInfo {
            texture,
            mip_level,
            origin: Origin3d {
                z: index,
                ..default()
            },
            aspect: TextureAspect::All,
        }
    }

    fn _buffer_copy_view<'a>(&'a self, buffer: &'a Buffer, index: u32) -> TexelCopyBufferInfo<'a> {
        TexelCopyBufferInfo {
            buffer,
            layout: TexelCopyBufferLayout {
                bytes_per_row: Some(self.aligned_side_size),
                rows_per_image: Some(self.texture_size),
                offset: self.buffer_size(index) as BufferAddress,
            },
        }
    }

    pub(crate) fn extend_3d(&self, mip_level: u32) -> Extent3d {
        Extent3d {
            width: self.texture_size >> mip_level,
            height: self.texture_size >> mip_level,
            depth_or_array_layers: 1,
        }
    }

    fn buffer_size(&self, slots: u32) -> u32 {
        slots * self.aligned_tile_size
    }

    fn attachment_meta(&self) -> AttachmentMeta {
        AttachmentMeta {
            lod_count: self.lod_count,
            texture_size: self.texture_size,
            border_size: self.border_size,
            center_size: self.center_size,
            pixels_per_entry: self.pixels_per_entry,
            entries_per_side: self.entries_per_side,
            entries_per_tile: self.entries_per_tile,
        }
    }
}

pub(crate) struct GpuAttachment {
    pub(crate) index: usize,

    pub(crate) buffer_info: AtlasBufferInfo,

    pub(crate) atlas_texture: Texture,

    pub(crate) mip_pipeline: CachedComputePipelineId,
    pub(crate) mip_views: Vec<TextureView>,
    pub(crate) mips_to_generate: Vec<Vec<u32>>,
    pub(crate) mip_bind_groups: Vec<Vec<BindGroup>>,

    pub(crate) _atlas_write_section: GpuBuffer<()>,
    pub(crate) _download_buffers: Vec<GpuBuffer<()>>,
    pub(crate) _bind_group: BindGroup,
    pub(crate) _max_atlas_write_slots: u32,
    pub(crate) _atlas_write_slots: Vec<AtlasTileAttachment>,
}

impl GpuAttachment {
    pub(crate) fn new(
        device: &RenderDevice,
        label: &AttachmentLabel,
        attachment: &Attachment,
        tile_atlas: &TileAtlas,
        settings: &TerrainSettings,
    ) -> Self {
        let index = settings
            .attachments
            .iter()
            .position(|l| l == label)
            .unwrap();

        let name = String::from(label);
        let max_atlas_write_slots = 4;
        let atlas_write_slots = Vec::with_capacity(max_atlas_write_slots as usize);

        let buffer_info = AtlasBufferInfo::new(attachment, tile_atlas.lod_count);

        let atlas_texture = device.create_texture(&TextureDescriptor {
            label: Some(&format!("{name}_attachment")),
            size: Extent3d {
                width: buffer_info.texture_size,
                height: buffer_info.texture_size,
                depth_or_array_layers: settings.atlas_size,
            },
            mip_level_count: attachment.mip_level_count,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: buffer_info.format.render_format(),
            usage: TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[buffer_info.format.processing_format()],
        });

        let atlas_view = atlas_texture.create_view(&TextureViewDescriptor {
            format: Some(buffer_info.format.processing_format()),
            ..default()
        });

        let atlas_sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..default()
        });

        let atlas_write_section = GpuBuffer::empty_sized_labeled(
            format!("{name}_atlas_write_section").as_str(),
            device,
            buffer_info.buffer_size(max_atlas_write_slots) as BufferAddress,
            BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        );

        let attachment_meta_buffer = GpuBuffer::create_labeled(
            format!("{name}_attachment_meta").as_str(),
            device,
            &buffer_info.attachment_meta(),
            BufferUsages::UNIFORM,
        );

        let bind_group = device.create_bind_group(
            format!("{name}_attachment_bind_group").as_str(),
            &create_attachment_layout(device),
            &BindGroupEntries::sequential((
                &atlas_write_section,
                &atlas_view,
                &atlas_sampler,
                &attachment_meta_buffer,
            )),
        );

        let mip_views = (0..buffer_info.mip_level_count)
            .map(|mip_level| {
                atlas_texture.create_view(&TextureViewDescriptor {
                    format: Some(buffer_info.format.processing_format()),
                    base_mip_level: mip_level,
                    mip_level_count: Some(1),
                    ..default()
                })
            })
            .collect_vec();

        Self {
            index,
            buffer_info,
            atlas_texture,
            mip_pipeline: CachedComputePipelineId::INVALID,
            mip_views,
            mips_to_generate: vec![default(); buffer_info.mip_level_count as usize],
            mip_bind_groups: vec![default(); buffer_info.mip_level_count as usize],

            _atlas_write_section: atlas_write_section,
            _download_buffers: default(),
            _bind_group: bind_group,
            _max_atlas_write_slots: max_atlas_write_slots,
            _atlas_write_slots: atlas_write_slots,
        }
    }

    pub(crate) fn prepare_mip_bind_groups(
        &mut self,
        device: &RenderDevice,
        mip_pipelines: &MipPipelines,
    ) {
        for (mip_level, atlas_indices) in self.mips_to_generate.iter().enumerate() {
            for atlas_index in atlas_indices {
                self.mip_bind_groups[mip_level].push(device.create_bind_group(
                    None,
                    &mip_pipelines.mip_layouts[&self.buffer_info.format],
                    &BindGroupEntries::sequential((
                        &GpuBuffer::create(device, atlas_index, BufferUsages::UNIFORM),
                        &self.mip_views[mip_level - 1],
                        &self.mip_views[mip_level],
                    )),
                ));
            }
        }
    }

    pub(crate) fn _reserve_write_slot(&mut self, tile: AtlasTileAttachment) -> Option<u32> {
        if self._atlas_write_slots.len() < self._max_atlas_write_slots as usize {
            self._atlas_write_slots.push(tile);
            Some(self._atlas_write_slots.len() as u32 - 1)
        } else {
            None
        }
    }

    pub(crate) fn _copy_tiles_to_write_section(&self, command_encoder: &mut CommandEncoder) {
        for (section_index, tile) in self._atlas_write_slots.iter().enumerate() {
            command_encoder.copy_texture_to_buffer(
                self.buffer_info
                    .texture_copy_view(&self.atlas_texture, tile.atlas_index, 0),
                self.buffer_info
                    ._buffer_copy_view(&self._atlas_write_section, section_index as u32),
                self.buffer_info.extend_3d(0),
            );
        }
    }

    pub(crate) fn _copy_tiles_from_write_section(&self, command_encoder: &mut CommandEncoder) {
        for (section_index, tile) in self._atlas_write_slots.iter().enumerate() {
            command_encoder.copy_buffer_to_texture(
                self.buffer_info
                    ._buffer_copy_view(&self._atlas_write_section, section_index as u32),
                self.buffer_info
                    .texture_copy_view(&self.atlas_texture, tile.atlas_index, 0),
                self.buffer_info.extend_3d(0),
            );
        }
    }

    pub(crate) fn _download_tiles(&self, command_encoder: &mut CommandEncoder) {
        for (tile, download_buffer) in iter::zip(&self._atlas_write_slots, &self._download_buffers)
        {
            command_encoder.copy_texture_to_buffer(
                self.buffer_info
                    .texture_copy_view(&self.atlas_texture, tile.atlas_index, 0),
                self.buffer_info._buffer_copy_view(download_buffer, 0),
                self.buffer_info.extend_3d(0),
            );
        }
    }

    pub(crate) fn _create_download_buffers(&mut self, device: &RenderDevice) {
        self._download_buffers = (0..self._atlas_write_slots.len())
            .map(|_| {
                GpuBuffer::empty_sized_labeled(
                    None,
                    device,
                    self.buffer_info.aligned_tile_size as BufferAddress,
                    BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                )
            })
            .collect_vec();
    }
}
