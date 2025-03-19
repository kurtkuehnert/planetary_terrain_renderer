#define_import_path bevy_terrain::vertex

#import bevy_terrain::types::{Blend, Coordinate, WorldCoordinate}
#import bevy_terrain::bindings::terrain_view
#import bevy_terrain::functions::{compute_coordinate, compute_world_coordinate, compute_blend, lookup_tile, apply_height}
#import bevy_terrain::attachments::sample_height
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::view_transformations::position_world_to_clip

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tile_uv: vec2<f32>,
    @location(1) tile_index: u32,
    @location(2) view_distance: f32,
    @location(3) height: f32,
}

struct VertexInfo {
    tile_index: u32,
    coordinate: Coordinate,
    world_coordinate: WorldCoordinate,
    blend: Blend,
}

fn vertex_info(input: VertexInput) -> VertexInfo {
    var info: VertexInfo;
    info.tile_index       = input.vertex_index / terrain_view.vertices_per_tile;
    info.coordinate       = compute_coordinate(input.vertex_index);
    info.world_coordinate = compute_world_coordinate(info.coordinate, info.tile_index, info.coordinate.uv);
    info.blend            = compute_blend(info.world_coordinate.view_distance);
    return info;
}

fn vertex_output(info: ptr<function, VertexInfo>, height: f32) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = position_world_to_clip(apply_height((*info).world_coordinate, height));
    output.tile_uv       = (*info).coordinate.uv;
    output.tile_index    = (*info).tile_index;
    output.view_distance = (*info).world_coordinate.view_distance;
    output.height        = height;
    return output;
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var info   = vertex_info(input);

    let tile   = lookup_tile(info.coordinate, info.blend);
    var height = sample_height(tile);

    return vertex_output(&info, height);
}
