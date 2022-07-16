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

let VERY_FAR: f32 = 1e20f;
let EPSILON: f32 = 0.00001;
let PI:f32 = 3.14159265358979;

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

fn random_float2( seed: u32 ) -> f32
{
	return f32(random_int( seed ) >> 16u) / 65535.0f;
}

fn rand(c: vec2<f32>) -> f32 {
	return fract( sin( dot( c.xy, vec2(12.9898, 78.233 ) ) ) * 43758.5453 );
}

fn noise( p: vec2<f32>, freq: f32 ) -> f32 {
	let unit = f32(globals.render_width)/freq;
	let ij = floor(p/unit);
	var xy = (p%unit)/unit;
	//xy = 3.*xy*xy-2.*xy*xy*xy;
	xy = .5*(1.-cos(PI*xy));
	let a = rand(ij+vec2<f32>(0.,0.));
	let b = rand(ij+vec2<f32>(1.,0.));
	let c = rand(ij+vec2<f32>(0.,1.));
	let d = rand(ij+vec2<f32>(1.,1.));
	let x1 = mix(a, b, xy.x);
	let x2 = mix(c, d, xy.x);
	return mix(x1, x2, xy.y);
}

fn perlin_noise(p: vec2<f32>, res: u32) -> f32{
	let persistance = .5;
	var n = 0.;
	var normK = 0.;
	var f = 4.;
	var amp = 1.;
	var iCount = 0u;
	for (var i = 0u; i<50u; i++){
		n+=amp*noise(p, f);
		f*=2.;
		normK+=amp;
		amp*=persistance;
		if (iCount == res) {
            break;
        }
		iCount++;
	}
	let nf = n/normK;
	return nf*nf*nf*nf;
}

let NEWTON_ITER = 2;
let HALLEY_ITER = 0;

fn cbrt( x:f32 ) -> f32
{
	var y = sign(x) * f32( u32( abs(x) ) / 3u + 0x2a514067u );

	for( var i = 0; i < NEWTON_ITER; i=i+1 ) {
    	y = ( 2. * y + x / ( y * y ) ) * .333333333;
    }

    for( var i = 0; i < HALLEY_ITER; i=i+1 )
    {
    	let y3 = y * y * y;
        y *= ( y3 + 2. * x ) / ( 2. * y3 + x );
    }
    
    return y;
}

fn random_in_unit_sphere(rvec: vec3<f32>, seed: u32) -> vec3<f32> {
    var u = random_float2(seed);
    var v = random_float2(seed+(seed/2u));
    var w = random_float2(seed-(seed/2u));
    var theta = u * 2.0 * PI;
    var phi = acos(2.0 * v - 1.0);
    var r1 = cbrt(w);
    var sinTheta = sin(theta);
    var cosTheta = cos(theta);
    var sinPhi = sin(phi);
    var cosPhi = cos(phi);
    var x = r1 * sinPhi * cosTheta;
    var y = r1 * sinPhi * sinTheta;
    var z = r1 * cosPhi;

    return vec3<f32>(x, y, z);
}

fn random_unit_vector(rvec: vec3<f32>, seed: u32) -> vec3<f32> {
    return normalize(random_in_unit_sphere(rvec, seed));
}

fn lambertian( r: ray, i: intersection, m: material, c: vec4<f32>, seed: u32 ) -> shade {
    var offset = i.normal * EPSILON;
    var destination =  i.position + i.normal + offset + random_unit_vector(i.normal, seed);
    var e_origin = i.position + offset;
    var e_dir = normalize(destination - e_origin);

    let c = m.color;//vec4<f32>(0.5);
    let e = ray(e_origin, e_dir, r.pixel, r.bounces+1u);

    return shade( c, e );
}

fn reflect( v: vec3<f32>, n: vec3<f32> ) -> vec3<f32> {
    return v - 2.0*dot(v,n) * n;
}

fn metallic( r: ray, i: intersection, m: material, seed: u32 ) -> shade {
    let c = m.color;
    let offset = i.normal * EPSILON;
    let e_origin = i.position + offset;
    let reflected = normalize(reflect(r.dir, i.normal));
    let noise = m.fuzziness*random_unit_vector(i.normal,seed);
    let e_dir = normalize( reflected + noise );
    let e = ray(e_origin, e_dir, r.pixel, r.bounces+1u);

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
    r0 = r0*r0;
    return r0 + (1.0-r0)*pow((1.0 - cosine),5.0);
}

fn dielectric( r: ray, i: intersection, m: material, seed: u32 ) -> shade {
    let c = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let offset = i.normal * EPSILON;
    let e_origin = i.position + offset;

    var refraction_ratio = m.index_of_refraction;
    if ( i.front_face == 1u ) {
        refraction_ratio = 1.0/m.index_of_refraction;
     }

    let unit_dir = normalize(r.dir);

    let cos_theta = min(dot(-unit_dir, i.normal), 1.0);
    let sin_theta = sqrt(1.0 - cos_theta*cos_theta);
    let cannot_refract = refraction_ratio * sin_theta > 1.0;

    var e_dir = vec3<f32>(0.0, 0.0, 0.0);
//    if ( cannot_refract ||  reflectance(cos_theta, refraction_ratio) > random_float2(seed)) {
  //      e_dir = reflect(r.dir, i.normal);
    //} else {
        e_dir = refract(unit_dir, i.normal, refraction_ratio);
    //}

    let e = ray(e_origin, e_dir, r.pixel, r.bounces+1u);

    return shade( c, e );
}

// The ray struck the sky.
fn miss(r: ray) -> shade {
    let unit = normalize(r.dir);
    let t = 0.5 * unit.y + 1.0;
    let sky_gradient = (1.0-t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);

    let no_extension = ray( vec3<f32>(VERY_FAR,VERY_FAR,VERY_FAR), vec3<f32>(VERY_FAR,VERY_FAR,VERY_FAR), r.pixel, r.bounces+1u );

    return shade( vec4<f32>(sky_gradient, 1.0), no_extension );
}

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.shade_index, 1u );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

	let seed = (globals.frame * u32(147565741)) * u32(720898027) * index;

    let r = ray_buffer.rays[index];
    if (r.origin.x == VERY_FAR) {
        return;
    }
    let i = intersection_buffer.intersections[index];
    var pixel = r.pixel;
    let y = u32( floor(f32(pixel) / f32(globals.render_width)) );
    let x = pixel - (y*globals.render_width);

    var color = textureLoad(output, vec2<i32>(i32(x), i32(y)));

    if ( i.t == VERY_FAR ) {
        var s = miss(r);
        color = vec4<f32>( color.xyz * s.color.xyz, 1.0);

        ray_buffer.rays[index] = s.extension;
    } else {
        let material = materials.m[i.material];
        if ( r.bounces == 5u ) {
            color = vec4<f32>(color.xyz * vec3<f32>(0.0), 1.0);
            ray_buffer.rays[index] = ray( vec3<f32>(VERY_FAR,VERY_FAR,VERY_FAR), vec3<f32>(VERY_FAR,VERY_FAR,VERY_FAR), r.pixel, r.bounces+1u );
        } else {
            if ( material.reflectance == 0 ) {
                var s = lambertian(r, i, material, color, seed);
                color = vec4<f32>(color.xyz * s.color.xyz, 1.0);
                ray_buffer.rays[index] = s.extension;
            } else if ( material.reflectance == 1 ) {
                var s = metallic(r, i, material, seed);
                color = vec4<f32>(color.xyz * s.color.xyz, 1.0);                
                ray_buffer.rays[index] = s.extension;
            } else if ( material.reflectance == 2 ) {
                var s = dielectric(r, i, material, seed);
                //color = vec4<f32>(color.xyz * s.color.xyz, 1.0);                
                ray_buffer.rays[index] = s.extension;
            }
        }
    }

    storageBarrier();
    textureStore(output, vec2<i32>(i32(x), i32(y)), color);
}