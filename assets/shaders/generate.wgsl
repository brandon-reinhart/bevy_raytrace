let VERY_FAR: f32 = 1e20f;
let EPSILON: f32 = 0.001;
let PI:f32 = 3.14159265358979;

struct camera_config {
    transform: mat4x4<f32>,
    forward: vec3<f32>,
    fov: f32,
    up: vec3<f32>,
    image_plane_distance: f32,
    right: vec3<f32>,
    lens_focal_length: f32,
    position: vec3<f32>,
    fstop: f32,    
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

@group(0) @binding(0)
var<uniform> camera: camera_config;

@group(0) @binding(1)
var<storage, read_write> globals: globals_buf;

@group(1) @binding(0)
var<storage, read_write> ray_buffer: ray_buf;

fn hash3( ni: u32 ) -> vec3<f32>
{
    // integer hash copied from Hugo Elias
    var n = ni;
	n = (n << 13u) ^ n;
    n = n * (n * n * 15731u + 789221u) + 1376312589u;
    let k = n * vec3<u32>(n, n*16807u, n*48271u);

    let l = vec3<u32>(0x7fffffffu);
    let m = vec3<f32>(f32(k.x&l.x), f32(k.y&l.y), f32(k.z&l.z));
    return m / f32(0x7fffffff);
}

// "Essential Ray Generation Shaders", McGuire & Majercik
fn pinhole_ray( pixel: vec2<f32> ) -> ray {
    let tan_half_angle = tan(camera.fov / 2.f);
    var aspect_scale = 0.0;
//    if ( camera.fov_dir == 0 ) {
        aspect_scale = f32(globals.render_width);
  //  } else {
    //    aspect_scale = globals.render_height;
    //}

    let half_w = f32(globals.render_width) / 2.0;
    let half_h = f32(globals.render_height) / 2.0;

    var ray_dir = vec3<f32>( vec2<f32>(pixel.x - half_w, -pixel.y + half_h) * tan_half_angle / aspect_scale, -1.0);
    let ray_dir = normalize( ray_dir );

    let pixel_index = u32( pixel.y * f32(globals.render_width) + pixel.x );
    return ray( vec3<f32>(0.f), EPSILON, ray_dir, VERY_FAR, pixel_index, 0u );
}

fn thin_lens_ray( pixel: vec2<f32>, lens_offset: vec2<f32> ) -> ray {
    var ray = pinhole_ray( pixel );

    let theta = lens_offset.x + 2.f * PI;
    let radius = lens_offset.y;

    let u = cos(theta) * sqrt(radius);
    let v = sin(theta) * sqrt(radius);

    let focus_plane = (camera.image_plane_distance * camera.lens_focal_length) /
    (camera.image_plane_distance - camera.lens_focal_length);

    let focus_point = ray.dir * (focus_plane / dot(ray.dir, vec3<f32>(0.f, 0.f, -1.f)));

    let circle_of_confusion_radius = camera.lens_focal_length / (2.f * camera.fstop);

    ray.origin = vec3<f32>(1.f, 0.f, 0.f) * (u * circle_of_confusion_radius) +
    vec3<f32>(0.f, 1.f, 0.f) * (v * circle_of_confusion_radius);

    ray.dir = normalize(focus_point - ray.origin);

    return ray;
}

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.generate_index, 1u );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

    let x = index % globals.render_width;
    let y = (index / globals.render_width) % globals.render_height;
    let pixel = vec2<f32>(f32(x), f32(y));

    let lens_offset = vec2<f32>(0.0f, 0.0f);
    //var pray = pinhole_ray(pixel);
    var pray = thin_lens_ray(pixel, lens_offset);

    pray.origin += camera.transform.w.xyz;
    pray.dir = (camera.transform * vec4<f32>(pray.dir, 0.0)).xyz;

    storageBarrier();
    ray_buffer.rays[index] = pray;
}   