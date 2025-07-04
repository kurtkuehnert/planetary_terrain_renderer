use crate::{
    math::{TileCoordinate, ViewCoordinate},
    render::TerrainTilingPrepassPipelines,
    terrain_data::{TileTree, TileTreeEntry},
    terrain_view::TerrainViewComponents,
};
use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{StaticSystemParam, SystemParamItem, lifetimeless::SRes},
    },
    prelude::*,
    render::{
        Extract,
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::*,
        renderer::RenderDevice,
        storage::ShaderStorageBuffer,
        sync_world::MainEntity,
    },
};

#[derive(AsBindGroup)]
pub struct IndirectBindGroup {
    #[storage(0, visibility(compute), buffer)]
    pub(crate) indirect: Buffer,
}

#[derive(AsBindGroup)]
pub struct PrepassViewBindGroup {
    #[storage(0, visibility(compute), read_only)]
    pub(crate) terrain_view: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(compute))]
    pub(crate) approximate_height: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(compute), read_only)]
    pub(crate) tile_tree: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(compute), buffer)]
    pub(crate) final_tiles: Buffer,
    #[storage(4, visibility(compute), buffer)]
    pub(crate) temporary_tiles: Buffer,
    #[storage(5, visibility(compute), buffer)]
    pub(crate) state: Buffer,
}

#[derive(AsBindGroup)]
pub struct TerrainViewBindGroup {
    // Todo: replace with updatable uniform buffer
    #[storage(0, visibility(vertex, fragment), read_only)]
    pub(crate) terrain_view: Handle<ShaderStorageBuffer>,
    #[storage(1, visibility(vertex, fragment), read_only)]
    pub(crate) approximate_height: Handle<ShaderStorageBuffer>,
    #[storage(2, visibility(vertex, fragment), read_only)]
    pub(crate) tile_tree: Handle<ShaderStorageBuffer>,
    #[storage(3, visibility(vertex, fragment), read_only, buffer)]
    pub(crate) geometry_tiles: Buffer,
}

#[derive(ShaderType)]
pub(crate) struct GeometryTile {
    face: u32,
    lod: u32,
    xy: UVec2,
    view_distances: Vec4,
    morph_ratios: Vec4,
}

#[derive(ShaderType)]
pub(crate) struct Indirect {
    x_or_vertex_count: u32,
    y_or_instance_count: u32,
    z_or_base_vertex: u32,
    base_instance: u32,
}

#[derive(ShaderType)]
pub(crate) struct PrepassState {
    tile_count: u32,
    counter: i32,
    child_index: i32,
    final_index: i32,
}

#[derive(Default, ShaderType)]
pub struct TileTreeUniform {
    #[size(runtime)]
    pub(crate) entries: Vec<TileTreeEntry>,
}

#[derive(ShaderType)]
pub(crate) struct TerrainViewUniform {
    tree_size: u32,
    geometry_tile_count: u32,
    grid_size: f32,
    vertices_per_row: u32,
    vertices_per_tile: u32,
    morph_distance: f32,
    blend_distance: f32,
    load_distance: f32,
    subdivision_distance: f32,
    morph_range: f32,
    blend_range: f32,
    precision_distance: f32,
    face: u32,
    lod: u32,
    coordinates: [ViewCoordinate; 6],
    world_position: Vec3,
    half_spaces: [Vec4; 6],
    surface_approximation: [crate::math::SurfaceApproximation; 6],
}

impl From<&TileTree> for TerrainViewUniform {
    fn from(tile_tree: &TileTree) -> Self {
        TerrainViewUniform {
            tree_size: tile_tree.tree_size,
            geometry_tile_count: tile_tree.geometry_tile_count,
            grid_size: tile_tree.grid_size as f32,
            vertices_per_row: 2 * (tile_tree.grid_size + 2),
            vertices_per_tile: 2 * tile_tree.grid_size * (tile_tree.grid_size + 2),
            morph_distance: tile_tree.morph_distance as f32,
            blend_distance: tile_tree.blend_distance as f32,
            load_distance: tile_tree.load_distance as f32,
            subdivision_distance: tile_tree.subdivision_distance as f32,
            precision_distance: tile_tree.precision_distance as f32,
            morph_range: tile_tree.morph_range,
            blend_range: tile_tree.blend_range,
            face: tile_tree.view_face,
            lod: tile_tree.view_lod,
            coordinates: tile_tree
                .view_coordinates
                .map(|view_coordinate| ViewCoordinate::new(view_coordinate, tile_tree.view_lod)),
            world_position: tile_tree.view_world_position,
            half_spaces: tile_tree.half_spaces,

            surface_approximation: tile_tree.surface_approximation.clone(),
        }
    }
}

pub struct GpuTerrainView {
    pub(crate) order: u32,
    pub(crate) refinement_count: u32,
    pub(crate) indirect_buffer: Buffer,
    pub(crate) indirect_bind_group: Option<BindGroup>,
    pub(crate) prepass_view_bind_group: Option<BindGroup>,
    pub(crate) terrain_view_bind_group: Option<BindGroup>,

    indirect: IndirectBindGroup,
    prepass_view: PrepassViewBindGroup,
    terrain_view: TerrainViewBindGroup,
}

impl GpuTerrainView {
    fn new(device: &RenderDevice, tile_tree: &TileTree) -> Self {
        // Todo: figure out a better way of limiting the tile buffer size

        let tiles = device.create_buffer(&BufferDescriptor {
            label: None,
            size: GeometryTile::min_size().get() * tile_tree.geometry_tile_count as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let temporary_tiles = device.create_buffer(&BufferDescriptor {
            label: None,
            size: TileCoordinate::min_size().get() * tile_tree.geometry_tile_count as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let state = device.create_buffer(&BufferDescriptor {
            label: None,
            size: PrepassState::min_size().get(),
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let indirect = device.create_buffer(&BufferDescriptor {
            label: None,
            size: Indirect::min_size().get(),
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        let prepare_prepass = IndirectBindGroup {
            indirect: indirect.clone(),
        };
        let refine_tiles = PrepassViewBindGroup {
            terrain_view: tile_tree.terrain_view_buffer.clone(),
            approximate_height: tile_tree.approximate_height_buffer.clone(),
            tile_tree: tile_tree.tile_tree_buffer.clone(),
            final_tiles: tiles.clone(),
            temporary_tiles,
            state,
        };
        let terrain_view = TerrainViewBindGroup {
            terrain_view: tile_tree.terrain_view_buffer.clone(),
            approximate_height: tile_tree.approximate_height_buffer.clone(),
            tile_tree: tile_tree.tile_tree_buffer.clone(),
            geometry_tiles: tiles,
        };

        Self {
            order: tile_tree.order,
            refinement_count: tile_tree.refinement_count,
            indirect_buffer: indirect,
            indirect: prepare_prepass,
            prepass_view: refine_tiles,
            terrain_view,
            indirect_bind_group: None,
            prepass_view_bind_group: None,
            terrain_view_bind_group: None,
        }
    }

    pub(crate) fn initialize(
        device: Res<RenderDevice>,
        mut gpu_terrain_views: ResMut<TerrainViewComponents<GpuTerrainView>>,
        tile_trees: Extract<Res<TerrainViewComponents<TileTree>>>,
    ) {
        for (&(terrain, view), tile_tree) in tile_trees.iter() {
            if gpu_terrain_views.contains_key(&(terrain, view)) {
                continue;
            }

            gpu_terrain_views.insert((terrain, view), GpuTerrainView::new(&device, tile_tree));
        }
    }

    pub(crate) fn prepare_terrain_view(
        device: Res<RenderDevice>,
        prepass_pipeline: Res<TerrainTilingPrepassPipelines>,
        mut gpu_terrain_views: ResMut<TerrainViewComponents<GpuTerrainView>>,
        mut param: StaticSystemParam<<TerrainViewBindGroup as AsBindGroup>::Param>,
    ) {
        for gpu_terrain_view in &mut gpu_terrain_views.values_mut() {
            // Todo: be smarter about bind group recreation
            let bind_group = gpu_terrain_view.terrain_view.as_bind_group(
                &prepass_pipeline.terrain_view_layout,
                &device,
                &mut param,
            );
            gpu_terrain_view.terrain_view_bind_group = bind_group.ok().map(|b| b.bind_group);
        }
    }

    pub(crate) fn prepare_indirect(
        device: Res<RenderDevice>,
        prepass_pipeline: Res<TerrainTilingPrepassPipelines>,
        mut gpu_terrain_views: ResMut<TerrainViewComponents<GpuTerrainView>>,
        mut param: StaticSystemParam<<IndirectBindGroup as AsBindGroup>::Param>,
    ) {
        for gpu_terrain_view in &mut gpu_terrain_views.values_mut() {
            let bind_group = &mut gpu_terrain_view.indirect_bind_group;

            if bind_group.is_none() {
                *bind_group = gpu_terrain_view
                    .indirect
                    .as_bind_group(&prepass_pipeline.indirect_layout, &device, &mut param)
                    .ok()
                    .map(|b| b.bind_group);
            }
        }
    }

    pub(crate) fn prepare_refine_tiles(
        device: Res<RenderDevice>,
        prepass_pipeline: Res<TerrainTilingPrepassPipelines>,
        mut gpu_terrain_views: ResMut<TerrainViewComponents<GpuTerrainView>>,
        mut param: StaticSystemParam<<PrepassViewBindGroup as AsBindGroup>::Param>,
    ) {
        for gpu_terrain_view in gpu_terrain_views.values_mut() {
            // Todo: be smarter about bind group recreation
            let bind_group = gpu_terrain_view.prepass_view.as_bind_group(
                &prepass_pipeline.prepass_view_layout,
                &device,
                &mut param,
            );
            gpu_terrain_view.prepass_view_bind_group = bind_group.ok().map(|b| b.bind_group);
        }
    }
}

pub struct SetTerrainViewBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetTerrainViewBindGroup<I> {
    type Param = SRes<TerrainViewComponents<GpuTerrainView>>;
    type ViewQuery = MainEntity;
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        view: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        gpu_terrain_views: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_terrain_view = &gpu_terrain_views.into_inner()[&(item.main_entity().id(), view)];

        if let Some(bind_group) = &gpu_terrain_view.terrain_view_bind_group {
            pass.set_bind_group(I, bind_group, &[]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Skip
        }
    }
}

pub(crate) struct DrawTerrainCommand;

impl<P: PhaseItem> RenderCommand<P> for DrawTerrainCommand {
    type Param = SRes<TerrainViewComponents<GpuTerrainView>>;
    type ViewQuery = MainEntity;
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        view: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        gpu_terrain_views: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_terrain_view = &gpu_terrain_views.into_inner()[&(item.main_entity().id(), view)];

        pass.set_stencil_reference(gpu_terrain_view.order);
        pass.draw_indirect(&gpu_terrain_view.indirect_buffer, 0);

        RenderCommandResult::Success
    }
}
