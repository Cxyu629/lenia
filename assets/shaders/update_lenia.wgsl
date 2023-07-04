@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

struct LeniaGPUParams {
    random_float: f32,
    kernel_area: f32,
    kernel_resolution: f32,
    delta_time: f32,
    dt: f32,
    growth_resolution: u32,
}


@group(0) @binding(1)
var <uniform> params: LeniaGPUParams;

@group(0) @binding(2)
var kernel_texture: texture_storage_2d<rgba32float, read>;

@group(0) @binding(3)
var <storage, read> growth_array: array<f32>;


fn wrap(coords: vec2<i32>) -> vec2<i32> {
    let dimensions: vec2<i32> = textureDimensions(texture);
    return vec2<i32>(fract(vec2<f32>(coords) / vec2<f32>(dimensions)) * vec2<f32>(dimensions));
}

fn calculate_with_texture(location: vec2<i32>, the_texture: texture_storage_2d<rgba32float, read>, the_area: vec4<f32>, radius: f32) -> vec4<f32> {
    var sum: vec4<f32> = vec4(0.0);
    for (var dx: f32 = -radius; dx <= radius; dx += 1.0) {
        for (var dy: f32 = -radius; dy <= radius; dy += 1.0) {
            let weight = textureLoad(the_texture, wrap(vec2<i32>(i32(radius)) + vec2<i32>(i32(dx), i32(dy))));
            let value = textureLoad(texture, wrap(location + vec2<i32>(i32(dx), i32(dy))));
            sum += value * weight;
        }
    }

    return sum / the_area;
}

fn calculate_growth(value: f32, resolution: u32) -> f32 {
    let float_index = value * (f32(resolution) - 1.0);
    if float_index == 0.0 {
        return growth_array[0];
    }
    let left_index = floor(float_index);
    let right_index = ceil(float_index);
    let left_weight = fract(float_index);
    let right_weight = 1.0 - fract(float_index);
    
    return (growth_array[u32(left_index)] * left_weight + growth_array[u32(right_index)] * right_weight);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));


    let current = clamp(textureLoad(texture, location).x, 0.0, 1.0);
    let potential = calculate_with_texture(location, kernel_texture, vec4<f32>(params.kernel_area), (params.kernel_resolution - 1.0)/2.0);
    let growth = 2.0 * calculate_growth(potential.x, params.growth_resolution) - 1.0;
    let timestep = params.dt;

    let color = vec4<f32>(clamp(current + timestep * growth, 0.0, 1.0));

    storageBarrier();

    textureStore(texture, location, color);
}