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
    bounces: u32,
};

struct ray_buf {
    ray_count: u32,
    rays: array<ray>,
};

struct intersection {
    t: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
    material: u32,
    front_face: u32,
};

struct intersection_buf {
    intersections: array<intersection>,
};

struct sphere {
    center: vec3<f32>,
    radius: f32,
    material: u32,
};

struct object_list {
    sphere_count: u32,
    spheres: array<sphere>,
};

let VERY_FAR: f32 = 1e20f;
let EPSILON: f32 = 3.552713678800501e-15;

@group(0) @binding(0)
var<uniform> camera: camera_config;

@group(0) @binding(1)
var<storage, read_write> globals: globals_buf;

@group(1) @binding(0)
var<storage, read_write> ray_buffer: ray_buf;

@group(1) @binding(1)
var<storage, read_write> intersection_buffer: intersection_buf;

@group(2) @binding(0)
var<storage, read> objects: object_list;

fn point_at(r: ray, t: f32) -> vec3<f32> {
    return r.origin + r.dir * t;
}

fn default_intersection() -> intersection {
    return intersection ( VERY_FAR, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), u32(0), 0u );
}

fn sqr( x: f32 ) -> f32 {
    return x*x;
}

fn intersect_sphere(r: ray, s: sphere, t_min: f32, t_max: f32) -> intersection {
    var i = default_intersection();

    let oc = r.origin - s.center;
    let a = sqr(length(r.dir));
    let half_b = dot(oc, r.dir);
    let c = sqr(length(oc)) - sqr(s.radius);

    let dis = sqr(half_b) - a*c;
    if ( dis < 0.0 ) {
        return i;
    }

    let sqrtd = sqrt(dis);

    var root = (-half_b - sqrtd) / a;
    if ( root < t_min || t_max < root ) {
        root = (-half_b + sqrtd) / a;
        if ( root < t_min || t_max < root ) {
            return i;
        }
    }

    i.t = root;
    i.position = point_at(r, root);
    i.normal = normalize((i.position - s.center) / s.radius);
    i.front_face = 1u;

    if ( dot(r.dir, i.normal) > 0.0) {
        i.normal = -i.normal;
        i.front_face = 0u;
    }

    i.material = s.material;

    return i;
}

// Brute force. The world isn't partitioned in any way.
fn intersect_world(r: ray) -> intersection {
    var closest_hit = default_intersection();
    for(var i: i32 = 0; i < i32(objects.sphere_count); i = i + 1 ) {
        let hit = intersect_sphere( r, objects.spheres[i], EPSILON, VERY_FAR );
        if ( hit.t < closest_hit.t ) {
            closest_hit = hit;
        }        
    }

    return closest_hit;
}

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.intersect_index, 1u );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

    let r = ray_buffer.rays[index];
    if (r.origin.x == VERY_FAR) {
        return;
    }

    let i = intersect_world(r);

    storageBarrier();
    intersection_buffer.intersections[index] = i;
}