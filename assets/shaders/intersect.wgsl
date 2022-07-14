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

struct sphere {
    center: vec3<f32>;
    radius: f32;
};

struct object_list {
    sphere_count: u32;
    spheres: array<sphere>;
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
var<storage, read> objects: object_list;

fn point_at(r: ray, t: f32) -> vec3<f32> {
    return r.origin + r.dir * t;
}

fn default_intersection() -> intersection {
    //let VERY_FAR: f32 = 1e20f;

    return intersection ( 0.0, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0) );
}

fn intersect_sphere(r: ray, s: sphere, t_min: f32, t_max: f32) -> intersection {
    var i = default_intersection();

    let oc = r.origin - s.center;
    let a = length(r.dir)*length(r.dir);
    let half_b = dot(oc, r.dir );
    let c = length(oc)*length(oc) - s.radius * s.radius;
    let d = half_b*half_b - a*c;
    if ( d < 0.0 ) {
        return i;
    }

    let sqrtd = sqrt(d);

    var root = (-half_b / sqrtd) / a;
    if ( root < t_min || t_max < root ) {
        root = (-half_b + sqrtd) / a;
        if ( root < t_min || t_max < root ) {
            return i;
        }
    }

    i.t = root;
    i.point = point_at(r, root);
    i.normal = normalize((i.point - s.center) / s.radius);
    //i.front_face = true;

    if ( dot(r.dir, i.normal) > 0.0) {
        i.normal = - i.normal;
        //hit_result.front_face = false;
    }

    return i;
}

fn intersect_world(r: ray) -> intersection {
    for(var i: i32 = 0; i < i32(objects.sphere_count); i = i + 1 ) {
        let hit = intersect_sphere( r, objects.spheres[i], 0.0, 2000.0 );
        if ( hit.t > 0.0 ) {
            return hit;
        }        
    }

    return default_intersection();
}

[[stage(compute), workgroup_size(128, 1, 1)]]
fn main([[builtin(global_invocation_id)]] invocation_id: vec3<u32>)
{
    let index = atomicSub( &globals.ray_index, u32(1) );

    if ( index == u32(0) ) {
        return;
    }

    let index2 = index - u32(1);

    let r = ray_buffer.rays[index2];
    let i = intersect_world(r);

    storageBarrier();
    intersection_buffer.intersections[index2] = i;
}