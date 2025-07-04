#define_import_path bevy_terrain::fragment

#import bevy_terrain::types::{Blend, Coordinate, WorldCoordinate, AtlasTile, TangentSpace}
#import bevy_terrain::bindings::{terrain, terrain_view, geometry_tiles, approximate_height}
#import bevy_terrain::functions::{compute_coordinate, compute_world_coordinate, compute_blend, compute_tangent_space, lookup_tile, apply_height, high_precision}
#import bevy_terrain::attachments::{sample_height_mask, sample_surface_gradient}
#import bevy_terrain::debug::{show_data_lod, show_geometry_lod, show_tile_tree, show_pixels}
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::pbr_types::{PbrInput, pbr_input_new}
#import bevy_pbr::pbr_functions::{calculate_view, apply_pbr_lighting}

struct FragmentInput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tile_uv: vec2<f32>,
    @location(1) tile_index: u32,
    @location(2) view_distance: f32,
    @location(3) height: f32,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>
}

struct FragmentInfo {
    clip_position: vec4<f32>,
    tile_index: u32,
    height: f32,
    coordinate: Coordinate,
    world_coordinate: WorldCoordinate,
    tangent_space: TangentSpace,
    blend: Blend,
}

fn fragment_info(input: FragmentInput) -> FragmentInfo{
    var info: FragmentInfo;
    info.clip_position    = input.clip_position;
    info.tile_index       = input.tile_index;
    info.height           = input.height;
    info.coordinate       = compute_coordinate(input.tile_index, input.tile_uv);
    info.world_coordinate = compute_world_coordinate(info.coordinate, input.height, input.view_distance);
    info.tangent_space    = compute_tangent_space(info.world_coordinate);
    info.blend            = compute_blend(info.world_coordinate.view_distance);
    return info;
}

fn fragment_output(info: ptr<function, FragmentInfo>, output: ptr<function, FragmentOutput>, color: vec4<f32>, surface_gradient: vec3<f32>) {
    let world_position = vec4<f32>(apply_height((*info).world_coordinate, (*info).height), 1.0);

#ifdef LIGHTING
    var pbr_input: PbrInput                 = pbr_input_new();
    pbr_input.material.base_color           = color;
    pbr_input.material.perceptual_roughness = 1.0;
    pbr_input.material.reflectance          = vec3<f32>(0.0);
    pbr_input.frag_coord                    = (*info).clip_position;
    pbr_input.world_position                = world_position;
    pbr_input.world_normal                  = (*info).world_coordinate.normal;
    pbr_input.N                             = normalize((*info).world_coordinate.normal - surface_gradient);
    pbr_input.V                             = calculate_view(world_position, pbr_input.is_orthographic);

    (*output).color = apply_pbr_lighting(pbr_input);
#else
    (*output).color = color;
#endif
}

fn fragment_debug(info: ptr<function, FragmentInfo>, output: ptr<function, FragmentOutput>, tile: AtlasTile, surface_gradient: vec3<f32>) {
    let normal = normalize((*info).world_coordinate.normal - surface_gradient);

#ifdef SHOW_DATA_LOD
    (*output).color = show_data_lod((*info).blend, tile);
#endif
#ifdef SHOW_GEOMETRY_LOD
    (*output).color = show_geometry_lod((*info).coordinate, (*info).tile_index);
#endif
#ifdef SHOW_TILE_TREE
    (*output).color = show_tile_tree((*info).coordinate, (*info).world_coordinate);
#endif
#ifdef SHOW_PIXELS
    (*output).color = mix((*output).color, show_pixels(tile), 0.5);
#endif
#ifdef SHOW_UV
    (*output).color = vec4<f32>(tile.coordinate.uv, 0.0, 1.0);
#endif
#ifdef SHOW_NORMALS
    (*output).color = vec4<f32>(normal, 1.0);
    // (*output).color = vec4<f32>(surface_gradient, 1.0);
#endif
#ifdef TEST3
    if (high_precision((*info).world_coordinate.view_distance)) {
        (*output).color = mix((*output).color, vec4<f32>(0.3), 0.5);
    }
#endif
}

@fragment
fn fragment(input: FragmentInput) -> FragmentOutput {
    var info = fragment_info(input);

    let tile             = lookup_tile(info.coordinate, info.blend);
    let mask             = sample_height_mask(tile);
    let color            = vec4<f32>(0.5);
    let surface_gradient = sample_surface_gradient(tile, info.tangent_space);

    if (mask) { discard; }

    var output: FragmentOutput;
    fragment_output(&info, &output, color, surface_gradient);
    fragment_debug(&info, &output, tile, surface_gradient);
    return FragmentOutput(vec4<f32>(output.color.xyz, 1.0));
}
