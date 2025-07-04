use crate::{
    debug::DebugTerrain,
    render::{
        DrawTerrainCommand, GpuTerrainView, SetTerrainBindGroup, SetTerrainViewBindGroup,
        TERRAIN_DEPTH_FORMAT, TerrainItem, TerrainTilingPrepassPipelines,
    },
    shaders::{DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER},
    spawn::{TerrainsToSpawn, spawn_terrains},
    terrain::TerrainComponents,
    terrain_data::GpuTileAtlas,
    terrain_view::TerrainViewComponents,
};
use bevy::{
    pbr::{MeshPipeline, MeshPipelineViewLayoutKey, SetMaterialBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItemExtraIndex, SetItemPipeline,
            ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::RetainedViewEntity,
    },
};
use std::{hash::Hash, marker::PhantomData};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct TerrainPipelineKey {
    pub flags: TerrainPipelineFlags,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct TerrainPipelineFlags: u32 {
        const NONE               = 0;
        const SPHERICAL          = 1 <<  0;
        const WIREFRAME          = 1 <<  1;
        const SHOW_DATA_LOD      = 1 <<  2;
        const SHOW_GEOMETRY_LOD  = 1 <<  3;
        const SHOW_TILE_TREE     = 1 <<  4;
        const SHOW_PIXELS        = 1 <<  5;
        const SHOW_UV            = 1 <<  6;
        const SHOW_NORMALS       = 1 <<  7;
        const MORPH              = 1 <<  8;
        const BLEND              = 1 <<  9;
        const TILE_TREE_LOD      = 1 << 10;
        const LIGHTING           = 1 << 11;
        const SAMPLE_GRAD        = 1 << 12;
        const HIGH_PRECISION     = 1 << 13;
        const TEST1              = 1 << 14;
        const TEST2              = 1 << 15;
        const TEST3              = 1 << 16;
        const MSAA_RESERVED_BITS = TerrainPipelineFlags::MSAA_MASK_BITS << TerrainPipelineFlags::MSAA_SHIFT_BITS;
    }
}

impl TerrainPipelineFlags {
    const MSAA_MASK_BITS: u32 = 0b111111;
    const MSAA_SHIFT_BITS: u32 = 32 - 6;

    pub fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits = ((msaa_samples - 1) & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS;
        TerrainPipelineFlags::from_bits(msaa_bits).unwrap()
    }

    pub fn from_debug(debug: &DebugTerrain) -> Self {
        let mut key = TerrainPipelineFlags::NONE;

        if debug.wireframe {
            key |= TerrainPipelineFlags::WIREFRAME;
        }
        if debug.show_data_lod {
            key |= TerrainPipelineFlags::SHOW_DATA_LOD;
        }
        if debug.show_geometry_lod {
            key |= TerrainPipelineFlags::SHOW_GEOMETRY_LOD;
        }
        if debug.show_tile_tree {
            key |= TerrainPipelineFlags::SHOW_TILE_TREE;
        }
        if debug.show_pixels {
            key |= TerrainPipelineFlags::SHOW_PIXELS;
        }
        if debug.show_uv {
            key |= TerrainPipelineFlags::SHOW_UV;
        }
        if debug.show_normals {
            key |= TerrainPipelineFlags::SHOW_NORMALS;
        }
        if debug.morph {
            key |= TerrainPipelineFlags::MORPH;
        }
        if debug.blend {
            key |= TerrainPipelineFlags::BLEND;
        }
        if debug.tile_tree_lod {
            key |= TerrainPipelineFlags::TILE_TREE_LOD;
        }
        if debug.lighting {
            key |= TerrainPipelineFlags::LIGHTING;
        }
        if debug.sample_grad {
            key |= TerrainPipelineFlags::SAMPLE_GRAD;
        }
        if debug.high_precision {
            key |= TerrainPipelineFlags::HIGH_PRECISION;
        }
        if debug.test1 {
            key |= TerrainPipelineFlags::TEST1;
        }
        if debug.test2 {
            key |= TerrainPipelineFlags::TEST2;
        }
        if debug.test3 {
            key |= TerrainPipelineFlags::TEST3;
        }

        key
    }

    pub fn msaa_samples(&self) -> u32 {
        ((self.bits() >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS) + 1
    }

    pub fn polygon_mode(&self) -> PolygonMode {
        match self.contains(TerrainPipelineFlags::WIREFRAME) {
            true => PolygonMode::Line,
            false => PolygonMode::Fill,
        }
    }

    pub fn shader_defs(&self) -> Vec<ShaderDefVal> {
        let mut shader_defs = Vec::new();

        if self.contains(TerrainPipelineFlags::SPHERICAL) {
            shader_defs.push("SPHERICAL".into());
        }
        if self.contains(TerrainPipelineFlags::SHOW_DATA_LOD) {
            shader_defs.push("SHOW_DATA_LOD".into());
        }
        if self.contains(TerrainPipelineFlags::SHOW_GEOMETRY_LOD) {
            shader_defs.push("SHOW_GEOMETRY_LOD".into());
        }
        if self.contains(TerrainPipelineFlags::SHOW_TILE_TREE) {
            shader_defs.push("SHOW_TILE_TREE".into());
        }
        if self.contains(TerrainPipelineFlags::SHOW_PIXELS) {
            shader_defs.push("SHOW_PIXELS".into())
        }
        if self.contains(TerrainPipelineFlags::SHOW_UV) {
            shader_defs.push("SHOW_UV".into());
        }
        if self.contains(TerrainPipelineFlags::SHOW_NORMALS) {
            shader_defs.push("SHOW_NORMALS".into())
        }
        if self.contains(TerrainPipelineFlags::MORPH) {
            shader_defs.push("MORPH".into());
        }
        if self.contains(TerrainPipelineFlags::BLEND) {
            shader_defs.push("BLEND".into());
        }
        if self.contains(TerrainPipelineFlags::TILE_TREE_LOD) {
            shader_defs.push("TILE_TREE_LOD".into());
        }
        if self.contains(TerrainPipelineFlags::LIGHTING) {
            shader_defs.push("LIGHTING".into());
        }
        if self.contains(TerrainPipelineFlags::SAMPLE_GRAD) {
            shader_defs.push("SAMPLE_GRAD".into());
        }

        if self.contains(TerrainPipelineFlags::HIGH_PRECISION) {
            shader_defs.push("HIGH_PRECISION".into());
        }
        if self.contains(TerrainPipelineFlags::TEST1) {
            shader_defs.push("TEST1".into());
        }
        if self.contains(TerrainPipelineFlags::TEST2) {
            shader_defs.push("TEST2".into());
        }
        if self.contains(TerrainPipelineFlags::TEST3) {
            shader_defs.push("TEST3".into());
        }

        shader_defs
    }
}

/// The pipeline used to render the terrain entities.
#[derive(Resource)]
pub struct TerrainRenderPipeline<M: Material> {
    view_layout: BindGroupLayout,
    view_layout_multisampled: BindGroupLayout,
    terrain_layout: BindGroupLayout,
    terrain_view_layout: BindGroupLayout,
    material_layout: BindGroupLayout,
    vertex_shader: Handle<Shader>,
    fragment_shader: Handle<Shader>,
    marker: PhantomData<M>,
}

impl<M: Material> FromWorld for TerrainRenderPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let mesh_pipeline = world.resource::<MeshPipeline>();
        let prepass_pipelines = world.resource::<TerrainTilingPrepassPipelines>();

        let vertex_shader = match M::vertex_shader() {
            ShaderRef::Default => world.load_asset(DEFAULT_VERTEX_SHADER),
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => world.load_asset(path),
        };

        let fragment_shader = match M::fragment_shader() {
            ShaderRef::Default => world.load_asset(DEFAULT_FRAGMENT_SHADER),
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => world.load_asset(path),
        };

        Self {
            view_layout: mesh_pipeline
                .get_view_layout(MeshPipelineViewLayoutKey::empty())
                .clone(),
            view_layout_multisampled: mesh_pipeline
                .get_view_layout(MeshPipelineViewLayoutKey::MULTISAMPLED)
                .clone(),
            terrain_layout: prepass_pipelines.terrain_layout.clone(),
            terrain_view_layout: prepass_pipelines.terrain_view_layout.clone(),
            material_layout: M::bind_group_layout(device),
            vertex_shader,
            fragment_shader,
            marker: PhantomData,
        }
    }
}

impl<M: Material> SpecializedRenderPipeline for TerrainRenderPipeline<M> {
    type Key = TerrainPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = key.flags.shader_defs();

        let mut bind_group_layout = match key.flags.msaa_samples() {
            1 => vec![self.view_layout.clone()],
            _ => {
                shader_defs.push("MULTISAMPLED".into());
                vec![self.view_layout_multisampled.clone()]
            }
        };

        bind_group_layout.push(self.terrain_layout.clone());
        bind_group_layout.push(self.terrain_view_layout.clone());
        bind_group_layout.push(self.material_layout.clone());

        let mut vertex_shader_defs = shader_defs.clone();
        vertex_shader_defs.push("VERTEX".into());
        let mut fragment_shader_defs = shader_defs.clone();
        fragment_shader_defs.push("FRAGMENT".into());

        RenderPipelineDescriptor {
            label: None,
            layout: bind_group_layout,
            push_constant_ranges: default(),
            vertex: VertexState {
                shader: self.vertex_shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: vertex_shader_defs,
                buffers: Vec::new(),
            },
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: key.flags.polygon_mode(),
                conservative: false,
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
            },
            fragment: Some(FragmentState {
                shader: self.fragment_shader.clone(),
                shader_defs: fragment_shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(DepthStencilState {
                format: TERRAIN_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState {
                        compare: CompareFunction::GreaterEqual,
                        fail_op: StencilOperation::Keep,
                        depth_fail_op: StencilOperation::Keep,
                        pass_op: StencilOperation::Replace,
                    },
                    back: StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: key.flags.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

/// The draw function of the terrain. It sets the pipeline and the bind groups and then issues the
/// draw call.
pub(crate) type DrawTerrain<M> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetTerrainBindGroup<1>,
    SetTerrainViewBindGroup<2>,
    SetMaterialBindGroup<M, 3>,
    DrawTerrainCommand,
);

/// Queses all terrain entities for rendering via the terrain pipeline.
#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_terrain<M: Material>(
    draw_functions: Res<DrawFunctions<TerrainItem>>,
    debug: Option<Res<DebugTerrain>>,
    pipeline_cache: Res<PipelineCache>,
    terrain_pipeline: Res<TerrainRenderPipeline<M>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<TerrainRenderPipeline<M>>>,
    mut terrain_phases: ResMut<ViewSortedRenderPhases<TerrainItem>>,
    gpu_tile_atlases: Res<TerrainComponents<GpuTileAtlas>>,
    gpu_terrain_views: Res<TerrainViewComponents<GpuTerrainView>>,
    mut views: Query<(MainEntity, &Msaa)>,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    let draw_function = draw_functions.read().get_id::<DrawTerrain<M>>().unwrap();

    for (view, msaa) in &mut views {
        let Some(terrain_phase) = terrain_phases.get_mut(&RetainedViewEntity {
            main_entity: view.into(),
            auxiliary_entity: Entity::PLACEHOLDER.into(),
            subview_index: 0,
        }) else {
            continue;
        };

        for (&terrain, gpu_tile_atlas) in gpu_tile_atlases.iter() {
            let Some(gpu_terrain_view) = gpu_terrain_views.get(&(terrain, view)) else {
                continue;
            };

            let mut flags = TerrainPipelineFlags::from_msaa_samples(msaa.samples());
            if gpu_tile_atlas.is_spherical {
                flags |= TerrainPipelineFlags::SPHERICAL;
            }

            if let Some(debug) = &debug {
                flags |= TerrainPipelineFlags::from_debug(debug);
            } else {
                flags |= TerrainPipelineFlags::LIGHTING
                    | TerrainPipelineFlags::MORPH
                    | TerrainPipelineFlags::BLEND
                    | TerrainPipelineFlags::SAMPLE_GRAD;
            }

            let key = TerrainPipelineKey { flags };

            let pipeline = pipelines.specialize(&pipeline_cache, &terrain_pipeline, key);

            terrain_phase.add(TerrainItem {
                representative_entity: (terrain, terrain.into()), // technically wrong
                draw_function,
                pipeline,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                order: gpu_terrain_view.order,
            })
        }
    }
}

/// This plugin adds a custom material for a terrain.
///
/// It can be used to render the terrain using a custom vertex and fragment shader.
pub struct TerrainMaterialPlugin<M: Material>(PhantomData<M>);

impl<M: Material> Default for TerrainMaterialPlugin<M> {
    fn default() -> Self {
        Self(default())
    }
}

impl<M: Material + Clone> Plugin for TerrainMaterialPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<M>::default())
            .insert_resource(TerrainsToSpawn::<M>(vec![]))
            .add_systems(PostUpdate, spawn_terrains::<M>);

        app.sub_app_mut(RenderApp)
            .add_render_command::<TerrainItem, DrawTerrain<M>>()
            .init_resource::<SpecializedRenderPipelines<TerrainRenderPipeline<M>>>()
            .add_systems(Render, queue_terrain::<M>.in_set(RenderSet::QueueMeshes));
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<TerrainRenderPipeline<M>>();
    }
}
