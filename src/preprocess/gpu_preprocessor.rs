// use crate::terrain_data::tile_atlas::AtlasTile;
// use crate::{
//     preprocess::{PreprocessTask, PreprocessTaskType, Preprocessor, TerrainPreprocessItem},
//     terrain::TerrainComponents,
//     terrain_data::{GpuTileAtlas, TileAtlas},
//     util::GpuBuffer,
// };
// use bevy::{
//     prelude::*,
//     render::{
//         render_asset::RenderAssets,
//         render_resource::{binding_types::*, *},
//         renderer::RenderDevice,
//         texture::GpuImage,
//         Extract,
//     },
// };
// use std::collections::VecDeque;
//
// pub(crate) struct ProcessingTask {
//     pub(crate) task: PreprocessTask,
//     pub(crate) bind_group: Option<BindGroup>,
// }
//
// #[derive(Clone, Debug, ShaderType)]
// pub(crate) struct SplitData {
//     tile: AtlasTile,
//     top_left: Vec2,
//     bottom_right: Vec2,
//     tile_index: u32,
// }
//
// #[derive(Clone, Debug, ShaderType)]
// struct StitchData {
//     tile: AtlasTile,
//     neighbour_tiles: [AtlasTile; 8],
//     tile_index: u32,
// }
//
// #[derive(Clone, Debug, ShaderType)]
// struct DownsampleData {
//     tile: AtlasTile,
//     child_tiles: [AtlasTile; 4],
//     tile_index: u32,
// }
//
// pub(crate) fn create_split_layout(device: &RenderDevice) -> BindGroupLayout {
//     device.create_bind_group_layout(
//         None,
//         &BindGroupLayoutEntries::sequential(
//             ShaderStages::COMPUTE,
//             (
//                 uniform_buffer::<SplitData>(false), // split_tile_data
//                 texture_2d(TextureSampleType::Float { filterable: true }), // tile
//                 sampler(SamplerBindingType::Filtering), // tile_sampler
//             ),
//         ),
//     )
// }
//
// pub(crate) fn create_stitch_layout(device: &RenderDevice) -> BindGroupLayout {
//     device.create_bind_group_layout(
//         None,
//         &BindGroupLayoutEntries::single(ShaderStages::COMPUTE, uniform_buffer::<StitchData>(false)),
//     )
// }
//
// pub(crate) fn create_downsample_layout(device: &RenderDevice) -> BindGroupLayout {
//     device.create_bind_group_layout(
//         None,
//         &BindGroupLayoutEntries::single(
//             ShaderStages::COMPUTE,
//             uniform_buffer::<DownsampleData>(false),
//         ),
//     )
// }
//
// pub(crate) struct GpuPreprocessor {
//     pub(crate) ready_tasks: VecDeque<PreprocessTask>,
//     pub(crate) processing_tasks: Vec<ProcessingTask>,
// }
//
// impl GpuPreprocessor {
//     pub(crate) fn new() -> Self {
//         Self {
//             ready_tasks: default(),
//             processing_tasks: vec![],
//         }
//     }
//
//     pub(crate) fn initialize(
//         mut gpu_preprocessors: ResMut<TerrainComponents<GpuPreprocessor>>,
//         terrains: Extract<Query<Entity, Added<TileAtlas>>>,
//     ) {
//         for terrain in terrains.iter() {
//             gpu_preprocessors.insert(terrain, GpuPreprocessor::new());
//         }
//     }
//
//     pub(crate) fn extract(
//         mut gpu_preprocessors: ResMut<TerrainComponents<GpuPreprocessor>>,
//         preprocessors: Extract<Query<(Entity, &Preprocessor)>>,
//     ) {
//         for (terrain, preprocessor) in preprocessors.iter() {
//             let gpu_preprocessor = gpu_preprocessors.get_mut(&terrain).unwrap();
//
//             // Todo: mem take using &mut world?
//             gpu_preprocessor
//                 .ready_tasks
//                 .extend(preprocessor.ready_tasks.clone().into_iter());
//         }
//     }
//
//     #[allow(clippy::too_many_arguments)]
//     pub(crate) fn prepare(
//         device: Res<RenderDevice>,
//         images: Res<RenderAssets<GpuImage>>,
//         preprocess_items: Res<TerrainComponents<TerrainPreprocessItem>>,
//         mut gpu_preprocessors: ResMut<TerrainComponents<GpuPreprocessor>>,
//         mut gpu_tile_atlases: ResMut<TerrainComponents<GpuTileAtlas>>,
//         pipeline_cache: Res<PipelineCache>,
//     ) {
//         for (&terrain, item) in preprocess_items.iter() {
//             if !item.is_loaded(&pipeline_cache) {
//                 continue;
//             }
//
//             let gpu_preprocessor = gpu_preprocessors.get_mut(&terrain).unwrap();
//             let gpu_tile_atlas = gpu_tile_atlases.get_mut(&terrain).unwrap();
//
//             gpu_preprocessor.processing_tasks.clear();
//
//             while !gpu_preprocessor.ready_tasks.is_empty() {
//                 let task = gpu_preprocessor.ready_tasks.back().unwrap();
//                 let attachment = &mut gpu_tile_atlas.attachments
//                     [task.tile.attachment_index as usize]
//                     .as_mut()
//                     .unwrap();
//
//                 if let Some(section_index) = attachment.reserve_write_slot(task.tile) {
//                     let task = gpu_preprocessor.ready_tasks.pop_back().unwrap();
//
//                     let bind_group = match &task.task_type {
//                         PreprocessTaskType::Split {
//                             tile_data,
//                             top_left,
//                             bottom_right,
//                         } => {
//                             let tile_data = images.get(tile_data).unwrap();
//
//                             let split_buffer = GpuBuffer::create_labeled(
//                                 None,
//                                 &device,
//                                 &SplitData {
//                                     tile: task.tile.into(),
//                                     top_left: *top_left,
//                                     bottom_right: *bottom_right,
//                                     tile_index: section_index,
//                                 },
//                                 BufferUsages::UNIFORM,
//                             );
//
//                             Some(device.create_bind_group(
//                                 None,
//                                 &create_split_layout(&device),
//                                 &BindGroupEntries::sequential((
//                                     &split_buffer,
//                                     &tile_data.texture_view,
//                                     &tile_data.sampler,
//                                 )),
//                             ))
//                         }
//                         PreprocessTaskType::Stitch { neighbour_tiles } => {
//                             let stitch_buffer = GpuBuffer::create_labeled(
//                                 None,
//                                 &device,
//                                 &StitchData {
//                                     tile: task.tile.into(),
//                                     neighbour_tiles: *neighbour_tiles,
//                                     tile_index: section_index,
//                                 },
//                                 BufferUsages::UNIFORM,
//                             );
//
//                             Some(device.create_bind_group(
//                                 None,
//                                 &create_stitch_layout(&device),
//                                 &BindGroupEntries::single(&stitch_buffer),
//                             ))
//                         }
//                         PreprocessTaskType::Downsample { child_tiles } => {
//                             let downsample_buffer = GpuBuffer::create_labeled(
//                                 None,
//                                 &device,
//                                 &DownsampleData {
//                                     tile: task.tile.into(),
//                                     child_tiles: *child_tiles,
//                                     tile_index: section_index,
//                                 },
//                                 BufferUsages::UNIFORM,
//                             );
//
//                             Some(device.create_bind_group(
//                                 None,
//                                 &create_downsample_layout(&device),
//                                 &BindGroupEntries::single(&downsample_buffer),
//                             ))
//                         }
//                         _ => break,
//                     };
//
//                     gpu_preprocessor
//                         .processing_tasks
//                         .push(ProcessingTask { task, bind_group });
//                 } else {
//                     break;
//                 }
//             }
//         }
//     }
// }
