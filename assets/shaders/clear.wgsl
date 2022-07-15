struct camera_config {
    frame: u32,
    render_width: u32,
    render_height: u32,

    camera_forward: vec3<f32>,
    camera_up: vec3<f32>,
    camera_right: vec3<f32>,
    camera_position: vec3<f32>,
};

struct globals_buf {
    clear_index: atomic<u32>,
    generate_index: atomic<u32>,
    intersect_index: atomic<u32>,
    shade_index: atomic<u32>,
};

struct ray {
    origin: vec3<f32>,
    dir: vec3<f32>,
    pixel: u32,
};

struct ray_buf {
    ray_count: u32,
    rays: array<ray>,
};

@group(0) @binding(0)
var<uniform> camera: camera_config;

@group(0) @binding(1)
var<storage, read_write> globals: globals_buf;

@group(1) @binding(0)
var<storage, read_write> ray_buffer: ray_buf;

@group(2) @binding(0)
var output: texture_storage_2d<rgba32float, read_write>;

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.clear_index, 1u );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

    let x = index % camera.render_width;
    let y = (index / camera.render_width) % camera.render_height;

    let black = vec4<f32>( vec3<f32>( 1.0 ), 1.0 );

    storageBarrier();
    textureStore(output, vec2<i32>(i32(x), i32(y)), black);
}
