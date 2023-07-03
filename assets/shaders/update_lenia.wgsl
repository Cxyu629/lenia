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
    return vec2<i32>(i32(fract(f32(coords.x) / f32(dimensions.x)) * f32(dimensions.x)), i32(fract(f32(coords.y) / f32(dimensions.y)) * f32(dimensions.y)));
}

fn calculate_with_texture(location: vec2<i32>, the_texture: texture_storage_2d<rgba32float, read>, the_area: f32, radius: f32) -> f32 {
    var sum: f32 = 0.0;
    for (var dx: f32 = -radius; dx <= radius; dx += 1.0) {
        for (var dy: f32 = -radius; dy <= radius; dy += 1.0) {
            let weight = textureLoad(the_texture, wrap(vec2<i32>(i32(radius)) + vec2<i32>(i32(dx), i32(dy)))).x;
            let value = textureLoad(texture, wrap(location + vec2<i32>(i32(dx), i32(dy)))).x;
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
    
    return 2.8 * (growth_array[u32(left_index)] * left_weight + growth_array[u32(right_index)] + right_weight) - 1.0;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));


    let current = clamp(textureLoad(texture, location).x, 0.0, 1.0);
    let potential = calculate_with_texture(location, kernel_texture, params.kernel_area, (params.kernel_resolution - 1.0)/2.0);
    let growth = calculate_growth(potential, params.growth_resolution);
    let timestep = params.dt * params.delta_time;

    let color = vec4<f32>(current + timestep * growth);

    storageBarrier();

    textureStore(texture, location, color);
}