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

struct sphere {
    center: vec3<f32>,
    radius: f32,
    material: u32,
};

struct object_list {
    sphere_count: u32,
    spheres: array<sphere>,
};

struct material {
    color: vec4<f32>,
    reflectance: i32,
    fuzziness: f32,
    index_of_refraction: f32,
    pad2: i32,
}

struct material_buf {
    m: array<material>,
}

struct shade {
    color: vec4<f32>,
    extension: ray,
}

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

@group(2) @binding(1)
var<storage, read> materials: material_buf;

let NEWTON_ITER = 2;
let HALLEY_ITER = 0;

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

fn lambertian( r: ray, i: intersection, m: material, seed: vec3<f32> ) -> shade {    
    var offset = i.normal * EPSILON;
    
    var destination =  i.position + i.normal + normalize(seed);

    var e_origin = i.position;
    var e_dir = normalize(destination - e_origin);

    let c = m.color;
    let e = ray(e_origin, EPSILON, e_dir, VERY_FAR, r.pixel, r.bounces+1u);

    return shade( c, e );
}

fn reflect( v: vec3<f32>, n: vec3<f32> ) -> vec3<f32> {
    return v - 2.0*dot(v,n) * n;
}

fn metallic( r: ray, i: intersection, m: material, seed: vec3<f32> ) -> shade {
    let c = m.color;
    let offset = i.normal * EPSILON;
    let e_origin = i.position + offset;
    let reflected = normalize(reflect(r.dir, i.normal));
    let noise = m.fuzziness*normalize(seed);
    let e_dir = normalize( reflected + noise );
    let e = ray(e_origin, EPSILON, e_dir, VERY_FAR, r.pixel, r.bounces+1u);

    return shade( c, e );
}

fn refract( uv: vec3<f32>, n: vec3<f32>, etai_over_etat: f32 ) -> vec3<f32> {
    let cos_theta = min(dot(-uv, n), 1.0);
    let r_out_perp =  etai_over_etat * (uv + cos_theta*n);
    let l = length(r_out_perp);
    let r_out_parallel = -sqrt(abs(1.0 - (l*l))) * n;
    return normalize( r_out_perp + r_out_parallel );
}

fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    // Use Schlick's approximation for reflectance.
    var r0 = (1.0-ref_idx) / (1.0+ref_idx);
    r0 = r0 * r0;
    return r0 + (1.0-r0)*pow((1.0 - cosine), 5.0);
}

fn dielectric( r: ray, i: intersection, m: material, seed: vec3<f32> ) -> shade {
    var refraction_ratio = m.index_of_refraction;
    if ( i.front_face == 1u ) {
        refraction_ratio = 1.0/m.index_of_refraction;
     }

    let unit_dir = normalize(r.dir);
    let cos_theta = min(dot(-unit_dir, i.normal), 1.0);
    let sin_theta = sqrt(1.0 - cos_theta*cos_theta);

    let cannot_refract = refraction_ratio * sin_theta > 1.0;

    var e_dir = vec3<f32>(0.0, 0.0, 0.0);
    if ( cannot_refract || reflectance(cos_theta, refraction_ratio) > seed.x) {
        e_dir = reflect(r.dir, i.normal);
    } else {
        e_dir = refract(unit_dir, i.normal, refraction_ratio);
    }

    let e_origin = i.position + i.normal * EPSILON;
    let e = ray(e_origin, EPSILON, e_dir, VERY_FAR, r.pixel, r.bounces+1u);

    let attenuation = vec4<f32>(1.0);
    return shade( attenuation, e );
}

fn miss(r: ray) -> shade {
    let unit = normalize(r.dir);
    let t = 0.5 * unit.y + 1.0;
    let sky_gradient = (1.0-t) * vec3<f32>(1.0) + t * vec3<f32>(0.5, 0.7, 1.0);

    let no_extension = ray( vec3<f32>(VERY_FAR), EPSILON, vec3<f32>(VERY_FAR), VERY_FAR, r.pixel, r.bounces+1u );

    return shade( vec4<f32>(sky_gradient, 1.0), no_extension );
}

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.shade_index, 1u );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

    let r = ray_buffer.rays[index];
    if (r.origin.x == VERY_FAR) {
        return;
    }
    let i = intersection_buffer.intersections[index];
    var pixel = r.pixel;
    let y = pixel / globals.render_width;
    let x = pixel - (y*globals.render_width);

    let seed = hash3( x 
        + globals.render_width*y 
        + (globals.render_width*globals.render_height) * globals.frame );

    var st = vec2<f32>(
        f32(x) / f32(globals.render_width),
        f32(y) / f32(globals.render_height),
    );
    st *= f32(globals.frame) % 1000.0;
    st *= 100.0;

    var color = intersection_buffer.intersections[index].color;

    if ( i.t == VERY_FAR ) {
        var s = miss(r);
        color *= s.color;

        ray_buffer.rays[index] = s.extension;
    } else {
        let material = materials.m[i.material];
        if ( r.bounces == 2u ) {
            ray_buffer.rays[index] = ray( vec3<f32>(VERY_FAR), EPSILON, vec3<f32>(VERY_FAR), VERY_FAR, r.pixel, r.bounces+1u );
            color = vec4<f32>(0.0, 0.0, 0.0, 1.0 );
        } else {
            if ( material.reflectance == 0 ) {
                var s = lambertian(r, i, material, seed);
                color *= s.color;
                ray_buffer.rays[index] = s.extension;
            } else if ( material.reflectance == 1 ) {
                var s = metallic(r, i, material, seed);
                color *= s.color;
                ray_buffer.rays[index] = s.extension;
            } else if ( material.reflectance == 2 ) {
                var s = dielectric(r, i, material, seed);
                color *= s.color;
                ray_buffer.rays[index] = s.extension;
            }
        }
    }

    storageBarrier();
    intersection_buffer.intersections[index].color = color;
}