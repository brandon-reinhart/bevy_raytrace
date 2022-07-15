struct camera_config {
    frame: u32,
    render_width: u32,
    render_height: u32,

    camera_forward: vec3<f32>,
    camera_up: vec3<f32>,
    camera_right: vec3<f32>,
    camera_position: vec3<f32>,
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

struct globals_buf {
    ray_index: atomic<u32>,
};

struct intersection {
    t: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
    material: u32,
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
    color: vec3<f32>,
    reflectance: i32,
}

struct material_buf {
    m: array<material>,
}

struct shade {
    color: vec4<f32>,
    extension: ray,
}

let VERY_FAR: f32 = 1e20f;

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

@group(3) @binding(0)
var output: texture_storage_2d<rgba32float, read_write>;

fn random_int( seed: u32 ) -> u32
{
    var seed = seed;
	seed = seed ^ ( seed << u32(13) );
	seed = seed ^ ( seed >> u32(17) );
	seed = seed ^ ( seed << u32(5) );
	return seed;
}

fn random_float( seed: u32 ) -> f32 // [0,1]
{
	return f32( random_int( seed ) ) * f32(2.3283064365387e-10);
}

fn miss(r: ray) -> shade {
    let unit = normalize(r.dir);
    let t = 0.5 * unit.y + 1.0;
    let sky_gradient = (1.0-t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);

    let no_extension = ray( vec3<f32>(VERY_FAR,VERY_FAR,VERY_FAR), vec3<f32>(VERY_FAR,VERY_FAR,VERY_FAR), r.pixel );

    return shade( vec4<f32>(sky_gradient, 1.0), no_extension );
}

// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
fn radical_inverse_vdc(b: u32) -> f32 {
    var bits = b;
     bits = (bits << 16u) | (bits >> 16u);
     bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
     bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
     bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
     bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
     return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

let PI:f32 = 3.14159265358979;

fn hammersley2d(i: u32, n: u32) -> vec2<f32> {
     return vec2(f32(i)/f32(n), radical_inverse_vdc(i));
 }

fn hemisphere_sample_uniform(u: f32, v: f32) -> vec3<f32> {
     let phi = v * 2.0 * PI;
     let cos_theta = 1.0 - u;
     let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
     return vec3(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
}
    
fn hemisphere_sample_cos(u: f32, v:f32) -> vec3<f32> {
     let phi = v * 2.0 * PI;
     let cos_theta = sqrt(1.0 - u);
     let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
     return vec3(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
 }

 struct onb {
    b1: vec3<f32>,
    b2: vec3<f32>,
 }

//https://graphics.pixar.com/library/OrthonormalB/paper.pdf
fn revised_onb( n:vec3<f32> ) -> onb
{
    var onb = onb(vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0,0.0,0.0));

	if ( n.z < 0.f )
	{
		let a = 1.0f / (1.0f - n.z);
		let b = n.x * n.y * a;
		onb.b1 = vec3<f32>(1.0f - n.x * n.x * a, -b, n.x);
		onb.b2 = vec3<f32>(b, n.y * n.y*a - 1.0f, -n.y);
	}
	else
	{
		let a = 1.0f / (1.0f + n.z);
		let b = -n.x * n.y * a;
		onb.b1 = vec3<f32>(1.0f - n.x * n.x * a, b, -n.x);
		onb.b2 = vec3<f32>(b, 1.0f - n.y * n.y * a, -n.y);
	}

    return onb;
}

fn lambertian( r: ray, i: intersection, m: material, c: vec4<f32>, seed: u32 ) -> shade {
	let r1 = 2.f * PI * random_float( seed );
	let r2 = random_float( seed );
	let r2s = sqrt( r2 );

	let onb = revised_onb( i.normal );
    let u = onb.b1;
    let v = onb.b2;

    let dir = normalize( u * cos(r1) * r2s + v * sin(r1) * r2s + i.normal * sqrt( 1.0 - r2 ) );
//    dir.x = abs( dir.x ) > epsilon ? dir.x : ( dir.x >= 0 ? epsilon : -epsilon );
//    dir.y = abs( dir.y ) > epsilon ? dir.y : ( dir.y >= 0 ? epsilon : -epsilon );
//    dir.z = abs( dir.z ) > epsilon ? dir.z : ( dir.z >= 0 ? epsilon : -epsilon );
//    r.direction = vec4( ray_direction, 1.f );


    //let destination = i.position + i.normal + wooj;
    //let direction = normalize(destination - i.position);

//    let blah = cos(dir)

    let z = acos(dot(i.normal, r.dir));


    let c = vec4<f32>(m.color * z, 1.0);
    let e = ray(i.position, dir, r.pixel);

    return shade( c, e );
}

fn metallic( i: intersection, m: material ) -> vec4<f32> {
    return vec4<f32>(m.color, 1.0);
}

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.ray_index, u32(1) );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

	let seed = (camera.frame * u32(147565741)) * u32(720898027) * index;

    let r = ray_buffer.rays[index];
    if (r.origin.x == VERY_FAR) {
        return;
    }
    let i = intersection_buffer.intersections[index];
    var pixel = r.pixel;
    let y = u32( floor(f32(pixel) / f32(camera.render_width)) );
    let x = pixel - (y*camera.render_width);

    var color = textureLoad(output, vec2<i32>(i32(x), i32(y)));

    if ( i.t == VERY_FAR ) {
        var s = miss(r);
        color = 0.5 * color + 0.5 * s.color;
        ray_buffer.rays[index] = s.extension;
    } else {
        let material = materials.m[i.material];
        if ( material.reflectance == 0 ) {
            var s = lambertian(r, i, material, color, seed);
            //color *= 0.5 * color + 0.5 * s.color;
            color = s.color;
            ray_buffer.rays[index] = s.extension;
        } else if ( material.reflectance == 1 ) {
            color = metallic(i, material);
        }
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