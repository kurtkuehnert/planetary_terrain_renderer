#import bevy_terrain::types::{TileCoordinate, GeometryTile, Coordinate, WorldCoordinate, Blend}
#import bevy_terrain::bindings::{terrain, terrain_view, final_tiles, approximate_height, temporary_tiles, state}
#import bevy_terrain::functions::{compute_subdivision_coordinate, compute_world_coordinate, compute_morph, compute_blend, lookup_tile, apply_height}
#import bevy_render::maths::affine3_to_square

fn child_index() -> i32 {
    return atomicAdd(&state.child_index, state.counter);
}

fn parent_index(id: u32) -> i32 {
    return i32(terrain_view.geometry_tile_count - 1u) * clamp(state.counter, 0, 1) - i32(id) * state.counter;
}

fn final_index() -> i32 {
    return atomicAdd(&state.final_index, 1);
}

fn should_be_divided(coordinate: Coordinate, world_coordinate: WorldCoordinate) -> bool {
    return world_coordinate.view_distance < terrain_view.subdivision_distance / exp2(f32(coordinate.lod + 1));
}

fn subdivide(tile: TileCoordinate) {
    for (var i: u32 = 0u; i < 4u; i = i + 1u) {
        let child_xy  = vec2<u32>((tile.xy.x << 1u) + (i & 1u), (tile.xy.y << 1u) + (i >> 1u & 1u));
        let child_lod = tile.lod + 1u;

        temporary_tiles[child_index()] = TileCoordinate(tile.face, child_lod, child_xy);
    }
}

fn frustum_cull_aabb(coordinate: Coordinate) -> bool {
    if (coordinate.lod == 0) { return false; }

    var aabb_min = vec3<f32>(3.40282e+38);
    var aabb_max = vec3<f32>(-3.40282e+38);

    for (var i = 0u; i < 4; i = i + 1) {
        let corner_uv               = vec2<f32>(f32(i & 1u), f32(i >> 1u & 1u));
        let corner_coordinate       = Coordinate(coordinate.face, coordinate.lod, coordinate.xy, corner_uv);
        let corner_world_coordinate = compute_world_coordinate(corner_coordinate, approximate_height);
        let corner_low              = apply_height(corner_world_coordinate, terrain.min_height);
        let corner_high             = apply_height(corner_world_coordinate, terrain.max_height);

        aabb_min = min(aabb_min, min(corner_low, corner_high));
        aabb_max = max(aabb_max, max(corner_low, corner_high));
    }

    for (var i = 0; i < 6; i = i + 1) {
        let half_space     = terrain_view.half_spaces[i];
        let closest_corner = vec4<f32>(select(aabb_min, aabb_max, half_space.xyz > vec3<f32>(0.0)), 1.0);

        if (dot(half_space, closest_corner) < 0.0) { return true; } // The closest corner is outside
    }

    return false;
}

fn frustum_cull_sphere(coordinate: Coordinate) -> bool {
    let center_coordinate = Coordinate(coordinate.face, coordinate.lod, coordinate.xy, vec2<f32>(0.5));
    let center_position   = compute_world_coordinate(center_coordinate, approximate_height).position;

    var radius = 0.0;

    for (var i = 0u; i < 4; i = i + 1) {
        let corner_uv               = vec2<f32>(f32(i & 1u), f32(i >> 1u & 1u));
        let corner_coordinate       = Coordinate(coordinate.face, coordinate.lod, coordinate.xy, corner_uv);
        let corner_world_coordinate = compute_world_coordinate(corner_coordinate, approximate_height);
        let corner_low              = apply_height(corner_world_coordinate, terrain.min_height);
        let corner_high             = apply_height(corner_world_coordinate, terrain.max_height);

        radius = max(radius, max(distance(center_position, corner_low), distance(center_position, corner_high)));
    }

    for (var i = 0; i < 6; i = i + 1) {
        let half_space = terrain_view.half_spaces[i];

        if (dot(half_space, vec4<f32>(center_position, 1.0)) + radius < 0.0) { return true; }
    }

     return false;
}

fn horizon_cull(coordinate: Coordinate, world_coordinate: WorldCoordinate) -> bool {
    // Todo: implement high precision supprot for culling
    if (coordinate.lod < 3) { return false; }
    // up to LOD 3, the closest point estimation is not reliable when projecting to adjacent sides
    // to prevent issues with cut of corners, horizon culling is skipped for those cases
    // this still leads to adeqate culling when close to the surface


    // position on the edge of the tile closest to the viewer with maximum height applied
    // serves as a conservative ocluder proxy
    // if this point is not visible, no other point of the tile should be visible

    // transform from world to unit coordinates centered on the world origin, this eliminates the oblatness of the ellipsoid
    let ellipsoid_to_sphere = 1.0 / terrain.scale;

    // radius of the culling sphere, to be conservative we use the minimal height scaled by the minor axis
    let radius = 1.0 + terrain.min_height / terrain.scale.y;

    let view_position   = ellipsoid_to_sphere * terrain_view.world_position;
    let tile_position   = ellipsoid_to_sphere * apply_height(world_coordinate, terrain.max_height);
    let origin_position = ellipsoid_to_sphere * (affine3_to_square(terrain.world_from_unit) * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let view_tile       = tile_position - view_position;
    let view_origin     = origin_position - view_position;

    let vh_vh = dot(view_origin, view_origin) - radius * radius; // distance square from view to horizon
    let vo_vt = dot(view_origin, view_tile);                     // distance square from view to tile projected onto the radius
    let vt_vt = dot(view_tile, view_tile);                       // distance square from view to tile

    // cull tile, if it is behind the horizon plane and it is inside the horizon cone
    return (vo_vt > vh_vh) && (vo_vt * vo_vt > vh_vh * vt_vt);
}

fn no_data_cull(coordinate: Coordinate, world_coordinate: WorldCoordinate) -> bool {
    var blend      = compute_blend(world_coordinate.view_distance);
    blend.lod      = min(coordinate.lod, blend.lod);
    let atlas_tile = lookup_tile(coordinate, blend);

    return atlas_tile.index == 4294967295;
}

fn cull(coordinate: Coordinate, world_coordinate: WorldCoordinate) -> bool {
//    if (frustum_cull_aabb(coordinate)) { return true; }
    if (frustum_cull_sphere(coordinate)) { return true; }
    if (horizon_cull(coordinate, world_coordinate)) { return true; }
    if (no_data_cull(coordinate, world_coordinate)) { return true; }

    return false;
}

fn prepare_tile(tile: TileCoordinate) -> GeometryTile {
    var morph_ratios: array<f32, 4>;

    for (var i = 0u; i < 4; i = i + 1) {
        let corner_uv               = vec2<f32>(f32(i & 1u), f32(i >> 1u & 1u));
        let corner_coordinate       = Coordinate(tile.face, tile.lod, tile.xy, corner_uv);
        let corner_world_coordinate = compute_world_coordinate(corner_coordinate, approximate_height);

        let view_distance = corner_world_coordinate.view_distance;
        let target_lod  = log2(terrain_view.morph_distance / view_distance);
        let lod         = corner_coordinate.lod;
        morph_ratios[i] = select(saturate(1.0 - (target_lod - f32(lod)) / terrain_view.morph_range), 0.0, lod == 0);
        morph_ratios[i] = compute_morph(corner_coordinate.lod, corner_world_coordinate.view_distance);
    }

    let t = vec4<f32>(morph_ratios[0], morph_ratios[1], morph_ratios[2], morph_ratios[3]);

    return GeometryTile(tile.face, tile.lod, tile.xy, t);
}

@compute @workgroup_size(64, 1, 1)
fn refine_tiles(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    if (invocation_id.x >= state.tile_count) { return; }

    let tile             = temporary_tiles[parent_index(invocation_id.x)];
    let coordinate       = compute_subdivision_coordinate(tile);
    let world_coordinate = compute_world_coordinate(coordinate, approximate_height);

    if cull(coordinate, world_coordinate) { return; }

    if (should_be_divided(coordinate, world_coordinate)) {
        subdivide(tile);
    } else {
        final_tiles[final_index()] = prepare_tile(tile);
    }
}
