use crate::{
    formats::TiffLoader,
    preprocess::{MipPipelines, MipPrepass},
    render::{
        DepthCopyPipeline, GpuTerrain, GpuTerrainView, TerrainItem, TerrainPass,
        TerrainTilingPrepassPipelines, TilingPrepass, TilingPrepassItem, extract_terrain_phases,
        prepare_terrain_depth_textures, queue_tiling_prepass,
    },
    shaders::{InternalShaders, load_terrain_shaders},
    terrain::{TerrainComponents, TerrainConfig},
    terrain_data::{
        AttachmentLabel, GpuTileAtlas, TileAtlas, TileTree, finish_loading, start_loading,
    },
    terrain_view::TerrainViewComponents,
};
use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        graph::CameraDriverLabel,
        render_graph::{RenderGraph, RenderGraphApp, ViewNodeRunner},
        render_phase::{DrawFunctions, ViewSortedRenderPhases, sort_phase_system},
        render_resource::*,
    },
};
use bevy_common_assets::ron::RonAssetPlugin;
use big_space::prelude::*;

#[derive(Resource)]
pub struct TerrainSettings {
    pub attachments: Vec<AttachmentLabel>,
    pub atlas_size: u32,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            attachments: vec![AttachmentLabel::Height],
            atlas_size: 1028,
        }
    }
}

impl TerrainSettings {
    pub fn new(custom_attachments: Vec<&str>) -> Self {
        let mut attachments = vec![AttachmentLabel::Height];
        attachments.extend(
            custom_attachments
                .into_iter()
                .map(|name| AttachmentLabel::Custom(name.to_string())),
        );

        Self {
            attachments,
            atlas_size: 1028,
        }
    }
}

/// The plugin for the terrain renderer.
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigSpaceDefaultPlugins);

        app.add_plugins(RonAssetPlugin::<TerrainConfig>::new(&["tc.ron"]))
            .init_asset::<TerrainConfig>()
            .init_resource::<InternalShaders>()
            .init_resource::<TerrainViewComponents<TileTree>>()
            .init_resource::<TerrainSettings>()
            .init_asset_loader::<TiffLoader>()
            .add_systems(
                PostUpdate,
                (
                    // Todo: enable visibility checking again
                    // check_visibility::<With<TileAtlas>>.in_set(VisibilitySystems::CheckVisibility),
                    (
                        TileTree::compute_requests,
                        finish_loading,
                        TileAtlas::update,
                        start_loading,
                        TileTree::adjust_to_tile_atlas,
                        TileTree::generate_surface_approximation,
                        TileTree::update_terrain_view_buffer,
                        TileAtlas::update_terrain_buffer,
                    )
                        .chain()
                        .after(TransformSystem::TransformPropagate),
                ),
            );
        app.sub_app_mut(RenderApp)
            .init_resource::<SpecializedComputePipelines<MipPipelines>>()
            .init_resource::<SpecializedComputePipelines<TerrainTilingPrepassPipelines>>()
            .init_resource::<TerrainComponents<GpuTileAtlas>>()
            .init_resource::<TerrainComponents<GpuTerrain>>()
            .init_resource::<TerrainViewComponents<GpuTerrainView>>()
            .init_resource::<TerrainViewComponents<TilingPrepassItem>>()
            .init_resource::<DrawFunctions<TerrainItem>>()
            .init_resource::<ViewSortedRenderPhases<TerrainItem>>()
            .add_systems(
                ExtractSchedule,
                (
                    extract_terrain_phases,
                    GpuTileAtlas::initialize,
                    GpuTileAtlas::extract.after(GpuTileAtlas::initialize),
                    GpuTerrain::initialize.after(GpuTileAtlas::initialize),
                    GpuTerrainView::initialize,
                ),
            )
            .add_systems(
                Render,
                (
                    (
                        GpuTileAtlas::prepare,
                        GpuTerrain::prepare,
                        GpuTerrainView::prepare_terrain_view,
                        GpuTerrainView::prepare_indirect,
                        GpuTerrainView::prepare_refine_tiles,
                    )
                        .in_set(RenderSet::Prepare),
                    sort_phase_system::<TerrainItem>.in_set(RenderSet::PhaseSort),
                    prepare_terrain_depth_textures.in_set(RenderSet::PrepareResources),
                    (queue_tiling_prepass, GpuTileAtlas::queue).in_set(RenderSet::Queue),
                    GpuTileAtlas::_cleanup
                        .before(World::clear_entities)
                        .in_set(RenderSet::Cleanup),
                ),
            )
            .add_render_graph_node::<ViewNodeRunner<TerrainPass>>(Core3d, TerrainPass)
            .add_render_graph_edges(
                Core3d,
                (Node3d::StartMainPass, TerrainPass, Node3d::MainOpaquePass),
            );

        let mut render_graph = app
            .sub_app_mut(RenderApp)
            .world_mut()
            .resource_mut::<RenderGraph>();
        render_graph.add_node(MipPrepass, MipPrepass);
        render_graph.add_node(TilingPrepass, TilingPrepass);
        render_graph.add_node_edge(MipPrepass, TilingPrepass);
        render_graph.add_node_edge(TilingPrepass, CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let attachments = app
            .world()
            .resource::<TerrainSettings>()
            .attachments
            .clone();

        load_terrain_shaders(app, &attachments);

        app.sub_app_mut(RenderApp)
            .init_resource::<TerrainTilingPrepassPipelines>()
            .init_resource::<MipPipelines>()
            .init_resource::<DepthCopyPipeline>();
    }
}
