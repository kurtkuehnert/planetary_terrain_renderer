#import bevy_terrain::types::{AtlasTile}
#import bevy_terrain::bindings::{terrain, terrain_view, attachments, height_attachment, albedo_atlas, albedo_attachment, terrain_sampler}
#import bevy_terrain::attachments::{compute_sample_uv, sample_height, sample_height_mask, compute_slope, sample_surface_gradient, relief_shading}
#import bevy_terrain::fragment::{FragmentInput, FragmentOutput, fragment_info, fragment_output, fragment_debug}
#import bevy_terrain::functions::{lookup_tile, inverse_mix, high_precision}
#import bevy_pbr::pbr_types::{PbrInput, pbr_input_new}
#import bevy_pbr::pbr_functions::{calculate_view, apply_pbr_lighting}

struct GradientInfo {
    mode: u32,
}

@group(3) @binding(0)
var gradient: texture_2d<f32>;
@group(3) @binding(1)
var gradient_sampler: sampler;
@group(3) @binding(2)
var<uniform> gradient_info: GradientInfo;

fn sample_albedo(tile: AtlasTile) -> vec4<f32> {
    let uv = compute_sample_uv(tile, attachments.albedo);

#ifdef SAMPLE_GRAD
    return textureSampleGrad(albedo_attachment, terrain_sampler, uv.uv, tile.index, uv.dx, uv.dy);
#else
    return textureSampleLevel(albedo_attachment, terrain_sampler, uv.uv, tile.index, tile.blend_ratio);
#endif
}

fn color_earth(tile: AtlasTile) -> vec4<f32> {
   let height = sample_height(tile);

    if (height < 0.0) {  
        return textureSampleLevel(gradient, gradient_sampler, vec2<f32>(mix(0.0, 0.075, pow(height / terrain.min_height, 0.25)), 0.5), 0.0);
    } else {
        return sample_albedo(tile);
//        return textureSampleLevel(gradient, gradient_sampler, vec2<f32>(mix(0.09, 0.6, pow(height / terrain.max_height * 1.4, 1.0)), 0.5), 0.0);
    }
}

fn color_dataset(tile: AtlasTile) -> vec4<f32> {
    let height = sample_height(tile);

    return textureSampleLevel(gradient, gradient_sampler, vec2<f32>(inverse_mix(terrain.min_height, terrain.max_height, height), 0.5), 0.0);
}

fn sample_color(tile: AtlasTile) -> vec4<f32> {
    var color: vec4<f32>;
    switch (gradient_info.mode) {
        case 0u: { color = color_dataset(tile); }
        case 1u: { color = color_earth(tile);   }
        case 2u: { color = sample_albedo(tile); }
        case 3u: {
            color = sample_albedo(tile);
            if (color.a == 0) {
                color = vec4<f32>(0.5);
            }
        }
        case default: {}
    }

    return color;
}

fn slope_gradient(world_normal: vec3<f32>, surface_gradient: vec3<f32>) -> vec4<f32> {
    let slope = compute_slope(world_normal, surface_gradient);
    return textureSampleLevel(gradient, gradient_sampler, vec2<f32>(5 * slope + 0.1, 0.5), 0.0);
}

@fragment
fn fragment(input: FragmentInput) -> FragmentOutput {
    var info = fragment_info(input);

    let tile             = lookup_tile(info.coordinate, info.blend);
    let mask             = sample_height_mask(tile);
    var color            = sample_color(tile);
    var surface_gradient = sample_surface_gradient(tile, info.tangent_space);

//    let uv_res_per_pixel = max(length(tile.coordinate.uv_dx), length(tile.coordinate.uv_dy)) * attachments.height.center_size;
//
//    if (uv_res_per_pixel > 2.0) {
//        color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
//    } else {
//        color = vec4<f32>(2.0 - uv_res_per_pixel, 0.0, 0.0, 1.0);
//    }

//    color = textureSampleLevel(gradient, gradient_sampler, vec2<f32>(uv_res_per_pixel / 10.0 , 0.5), 0.0);
//    color = vec4<f32>(log2(uv_res_per_pixel), 0.0, 0.0, 1.0);

    if mask { discard; }

//    color = vec4(vec3(0.3), 1.0);
//    color = slope_gradient(info.world_coordinate.normal, surface_gradient);

//    if (distance(info.world_coordinate.position, bevy_terrain::bindings::view.world_position) > terrain.scale.y / 2.0 * 0.987) { color = vec4(1.0, 0.0, 0.0, 1.0); }

    var output: FragmentOutput;
#ifdef LIGHTING
    output.color = color * relief_shading(info.world_coordinate, surface_gradient);
#else
    output.color = color;
#endif

#ifdef TEST1
    fragment_output(&info, &output, color, surface_gradient);
#endif

    fragment_debug(&info, &output, tile, surface_gradient);
    return output;
}
