struct camera_config {
    frame: u32;
    render_width: u32;
    render_height: u32;

    camera_forward: vec3<f32>;
    camera_up: vec3<f32>;
    camera_right: vec3<f32>;
    camera_position: vec3<f32>;
};

struct ray {
    origin: vec3<f32>;
    dir: vec3<f32>;
    pixel: u32;
};

struct ray_buf {
    ray_count: u32;
    rays: array<ray>;
};

struct globals_buf {
    ray_index: atomic<u32>;
};

struct intersection {
    t: f32;
    point: vec3<f32>;
    normal: vec3<f32>;
};

struct intersection_buf {
    intersections: array<intersection>;
};

[[group(0), binding(0)]]
var<uniform> camera: camera_config;

[[group(0), binding(1)]]
var<storage, read_write> globals: globals_buf;

[[group(1), binding(0)]]
var<storage, read_write> ray_buffer: ray_buf;

[[group(1), binding(1)]]
var<storage, read_write> intersection_buffer: intersection_buf;

[[group(2), binding(0)]]
var output: texture_storage_2d<rgba32float, read_write>;

fn miss(r: ray) -> vec4<f32> {
    let unit = normalize(r.dir);
    let t = 0.5 * unit.y + 1.0;
    let sky_gradient = (1.0-t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);

    return vec4<f32>(sky_gradient, 1.0);
}

[[stage(compute), workgroup_size(128, 1, 1)]]
fn main([[builtin(global_invocation_id)]] invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.ray_index, u32(1) );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

    let r = ray_buffer.rays[index];
    let i = intersection_buffer.intersections[index];
    var pixel = r.pixel;
    let y = u32( floor(f32(pixel) / f32(camera.render_width)) );
    let x = pixel - (y*camera.render_width);

    var color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
    if ( i.t == 0.0 ) {
        color = miss(r);
    } else {
        color = vec4<f32>(0.5 * (i.normal + vec3<f32>(1.0, 1.0, 1.0)), 1.0);
    }

    // Necessary bind groups:
    // rays (to write extensions)
    // intersections (for normals and material indexes)
    // materials (to look up refraction, etc)

    // In this shader we will determine the pixel color
    // based on the material at the point of intersection.

    // We will also generate extension rays based on the material.


    storageBarrier();
    textureStore(output, vec2<i32>(i32(x), i32(y)), color);
}