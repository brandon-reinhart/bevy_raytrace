struct camera_config {
    camera_forward: vec3<f32>,
    camera_up: vec3<f32>,
    camera_right: vec3<f32>,
    camera_position: vec3<f32>,
};

struct globals_buf {
    frame: u32,
    render_width: u32,
    render_height: u32,
    clear_index: u32,
    generate_index: u32,
    intersect_index: u32,
    shade_index: u32,
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

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    globals.clear_index = 0u;
    globals.generate_index = 0u;
    globals.intersect_index = 0u;
    globals.shade_index = 0u;
}
