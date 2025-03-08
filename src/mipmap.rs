use crate::{
    shaders::MIP_SHADER,
    terrain::TerrainComponents,
    terrain_data::{AttachmentFormat, GpuTileAtlas},
};
use bevy::{
    asset::{AssetServer, Handle},
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::{binding_types::*, *},
        renderer::{RenderContext, RenderDevice},
    },
    utils::HashMap,
};
use strum::IntoEnumIterator;

pub(crate) fn create_mip_layout(
    device: &RenderDevice,
    format: AttachmentFormat,
) -> BindGroupLayout {
    device.create_bind_group_layout(
        None,
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                uniform_buffer::<u32>(false), // atlas_index
                texture_2d_array(TextureSampleType::Float { filterable: true }), // parent
                texture_storage_2d_array(
                    format.processing_format(),
                    StorageTextureAccess::WriteOnly,
                ), // child
            ),
        ),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MipPipelineKey {
    pub(crate) format: AttachmentFormat,
}

impl MipPipelineKey {
    pub fn shader_defs(&self) -> Vec<ShaderDefVal> {
        let mut shader_defs = Vec::new();

        let format = match self.format {
            AttachmentFormat::Rgb8U => "RGB8U",
            AttachmentFormat::Rgba8U => "RGBA8U",
            AttachmentFormat::R16U => "R16U",
            AttachmentFormat::R16I => "R16I",
            AttachmentFormat::Rg16U => "RG16U",
            AttachmentFormat::R32F => "R32F",
        };

        shader_defs.push(format.into());

        shader_defs
    }
}

#[derive(Resource)]
pub struct MipPipelines {
    pub(crate) mip_layouts: HashMap<AttachmentFormat, BindGroupLayout>,
    mip_shader: Handle<Shader>,
}

impl FromWorld for MipPipelines {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();

        let mip_layouts = AttachmentFormat::iter()
            .map(|format| (format, create_mip_layout(device, format)))
            .collect();
        let mip_shader = asset_server.load(MIP_SHADER);

        Self {
            mip_layouts,
            mip_shader,
        }
    }
}

impl SpecializedComputePipeline for MipPipelines {
    type Key = MipPipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("mip_pipeline".into()),
            layout: vec![self.mip_layouts[&key.format].clone()],
            push_constant_ranges: default(),
            shader: self.mip_shader.clone(),
            shader_defs: key.shader_defs(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MipNode;

impl render_graph::Node for MipNode {
    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let gpu_tile_atlases = world.resource::<TerrainComponents<GpuTileAtlas>>();

        context.add_command_buffer_generation_task(move |device| {
            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

            for gpu_tile_atlas in gpu_tile_atlases.values() {
                gpu_tile_atlas.generate_mip(&mut pass, pipeline_cache);
            }

            drop(pass);

            encoder.finish()
        });

        Ok(())
    }
}
