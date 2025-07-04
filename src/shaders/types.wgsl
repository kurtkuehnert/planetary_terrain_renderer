#define_import_path bevy_terrain::types

struct Terrain {
    lod_count: u32,
    scale: vec3<f32>,
    min_height: f32,
    max_height: f32,
    height_scale: f32,
    world_from_unit: mat3x4<f32>,
    unit_from_world_transpose_a: mat2x4<f32>,
    unit_from_world_transpose_b: f32,
}

struct TerrainView {
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
    coordinates: array<ViewCoordinate, 6>,
    height_scale: f32,
    world_position: vec3<f32>,
    half_spaces: array<vec4<f32>, 6>,
#ifdef HIGH_PRECISION
    surface_approximation: array<SurfaceApproximation, 6>, // must be last field of this struct
#endif
}

struct TileCoordinate {
    face: u32,
    lod: u32,
    xy: vec2<u32>,
}

struct GeometryTile {
    face: u32,
    lod: u32,
    xy: vec2<u32>,
    view_distances: vec4<f32>,
    morph_ratios: vec4<f32>,
}

struct Coordinate {
    face: u32,
    lod: u32,
    xy: vec2<u32>,
    uv: vec2<f32>,
#ifdef FRAGMENT
    uv_dx: vec2<f32>,
    uv_dy: vec2<f32>,
#endif
}

struct WorldCoordinate {
    position: vec3<f32>,
    normal: vec3<f32>,
    view_distance: f32,
}

struct ViewCoordinate {
    xy: vec2<u32>,
    uv: vec2<f32>,
}

struct PrepassState {
    tile_count: u32,
    counter: i32,
    child_index: atomic<i32>,
    final_index: atomic<i32>,
}

struct Blend {
    lod: u32,
    ratio: f32,
}

struct TileTreeEntry {
    atlas_index: u32,
    atlas_lod: u32,
}

// A tile inside the tile atlas, looked up based on the view of a tile tree.
struct AtlasTile {
    index: u32,
    coordinate: Coordinate,
    blend_ratio: f32,
}

#ifdef HIGH_PRECISION
struct SurfaceApproximation {
    p: vec3<f32>,
    p_u: vec3<f32>,
    p_v: vec3<f32>,
    p_uu: vec3<f32>,
    p_uv: vec3<f32>,
    p_vv: vec3<f32>,
}
#endif

struct BestLookup {
    tile: AtlasTile,
    tile_tree_uv: vec2<f32>,
}

struct AttachmentConfig {
    texture_size: f32,
    center_size: f32,
    scale: f32,
    offset: f32,
    mask: u32,
    paddinga: u32,
    paddingb: u32,
    paddingc: u32,
}


struct IndirectBuffer {
    workgroup_count: vec3<u32>,
}

struct TangentSpace {
    tangent_x: vec3<f32>,
    tangent_y: vec3<f32>,
    scale: f32,
}

struct SampleUV {
    uv: vec2<f32>,
#ifdef FRAGMENT
    dx: vec2<f32>,
    dy: vec2<f32>,
#endif
}
