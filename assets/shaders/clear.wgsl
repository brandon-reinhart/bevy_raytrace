let VERY_FAR: f32 = 1e20f;
let EPSILON: f32 = 0.001;
let PI:f32 = 3.14159265358979;

struct camera_config {
    transform: mat4x4<f32>,
    forward: vec3<f32>,
    fov: f32,
    up: vec3<f32>,
    pad0: f32,
    right: vec3<f32>,
    pad1: f32,
    position: vec3<f32>,
    pad2: f32,
};

struct globals_buf {
    frame: u32,
    render_width: u32,
    render_height: u32,
    samples_per_ray: u32,
    clear_index: atomic<u32>,
    generate_index: atomic<u32>,
    intersect_index: atomic<u32>,
    shade_index: atomic<u32>,
    collect_index: atomic<u32>,
};

struct ray {
    origin: vec3<f32>,
    min: f32,
    dir: vec3<f32>,
    max: f32,
    pixel: u32,
    bounces: u32,
};

struct ray_buf {
    ray_count: u32,
    rays: array<ray>,
};

struct intersection {
    color: vec4<f32>,
    position: vec3<f32>,
    t: f32,
    normal: vec3<f32>,
    material: u32,
    front_face: u32,
};

struct intersection_buf {
    intersections: array<intersection>,
};

@group(0) @binding(0)
var<uniform> camera: camera_config;

@group(0) @binding(1)
var<storage, read_write> globals: globals_buf;

@group(1) @binding(0)
var<storage, read_write> ray_buffer: ray_buf;

@group(1) @binding(1)
var<storage, read_write> intersection_buffer: intersection_buf;

@group(2) @binding(0)
var output: texture_storage_2d<rgba32float, read_write>;

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.clear_index, 1u );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

    let x = index % globals.render_width;
    let y = (index / globals.render_width) % globals.render_height;

    let clear = vec4<f32>( vec3<f32>( 1.0 ), 1.0 );

    storageBarrier();
    textureStore(output, vec2<i32>(i32(x), i32(y)), clear);
    intersection_buffer.intersections[index].color = clear;
}
