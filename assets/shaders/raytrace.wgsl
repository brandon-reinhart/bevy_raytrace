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
};

struct hit {
    t: f32;
    p: vec3<f32>;
    n: vec3<f32>;
    front_face: bool;
    c: vec4<f32>;
    can_extend: bool;
    extend: ray;
};

fn default_hit() -> hit {
    return hit ( f32(0.0), vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), false, 
    vec4<f32>(0.0, 0.0, 0.0, 0.0),
    false, ray(vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0,0.0,0.0)));
}

struct sphere {
    center: vec3<f32>;
    radius: f32;
};

struct object_list {
    sphere_count: u32;
    spheres: array<sphere>;
};

[[group(0), binding(0)]]
var texture: texture_storage_2d<rgba32float, read_write>;

[[group(0), binding(1)]]
var<uniform> camera: camera_config;

[[group(1), binding(0)]]
var<storage, read> objects: object_list;

//"Xorshift RNGs" by George Marsaglia
//http://excamera.com/sphinx/article-xorshift.html

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
	return f32(random_int( seed ) >> u32(16)) / f32(65535.0);
}

fn random_int_between_0_and_max( seed: u32, max: u32 ) -> u32
{
	return u32( random_float( seed ) * ( f32(max) + f32(0.99999) ) );
}

fn random_2d_stratified_sample( seed: u32 ) -> vec2<f32>
{
    //Set the size of the pixel in stratums.
	let width2d = u32(4);
	let height2d = u32(4);
	let pixel_width = 1.0 / f32(width2d);
	let pixel_height = 1.0 / f32(height2d);

	let chosen_stratum = random_int_between_0_and_max( seed, width2d * height2d );
	//Compute stratum X in [0, width-1] and Y in [0,height -1]
	let stratum_x = chosen_stratum % width2d;
	let stratum_y = (chosen_stratum / width2d) % height2d;

	// Now we split up the pixel into [stratumX,stratumY] pieces.
	// Let's get the width and height of this sample.

	let stratum_x_start = f32( pixel_width * f32(stratum_x) );
	let stratum_y_start = f32( pixel_height * f32(stratum_y) );

	let random_point_in_stratum_x = stratum_x_start + ( random_float( seed ) * pixel_width );
	let random_point_in_stratum_y = stratum_y_start + ( random_float( seed ) * pixel_height );
	return vec2<f32>( random_point_in_stratum_x, random_point_in_stratum_y );
}

fn random_vec3( seed: u32 ) -> vec3<f32> {
    return vec3<f32>( random_float(seed), random_float(seed), random_float(seed) );
}

[[stage(compute), workgroup_size(8, 8, 1)]]
fn init() {

}

fn point_at(r: ray, t: f32) -> vec3<f32> {
    return r.origin + r.dir * t;
}

fn intersect_sphere(r: ray, s: sphere) -> f32 {
    let oc = r.origin - s.center;
    let a = dot( r.dir, r.dir );
    let b = 2.0 * dot( oc, r.dir );
    let c = dot( oc, oc ) - s.radius * s.radius;
    let d = b*b - 4.0*a*c;
    if ( d < 0.0 ) {
        return -1.0;
    } else {
        return (-b - sqrt(d)) / ( 2.0 * a);
    }
}

fn intersect_sphere2(r: ray, s: sphere, t_min: f32, t_max: f32) -> hit {
    var hit_result = default_hit();

    let oc = r.origin - s.center;
    let a = length(r.dir)*length(r.dir);
    let half_b = dot(oc, r.dir );
    let c = length(oc)*length(oc) - s.radius * s.radius;
    let d = half_b*half_b - a*c;
    if ( d < 0.0 ) {
        return hit_result;
    }

    let sqrtd = sqrt(d);

    var root = (-half_b / sqrtd) / a;
    if ( root < t_min || t_max < root ) {
        root = (-half_b + sqrtd) / a;
        if ( root < t_min || t_max < root ) {
            return hit_result;
        }
    }

    hit_result.t = root;
    hit_result.p = point_at(r, root);
    hit_result.n = normalize((hit_result.p - s.center) / s.radius);
    hit_result.front_face = true;

    if ( dot(r.dir, hit_result.n) > 0.0) {
        hit_result.n = - hit_result.n;
        hit_result.front_face = false;
    }

    return hit_result;
}

fn hit_sphere(r: ray, hit_result: hit ) -> vec4<f32> {    
    return vec4<f32>(0.5 * (hit_result.n + vec3<f32>(1.0, 1.0, 1.0)), 1.0);
}

fn miss(r: ray) -> hit {
    let unit = normalize(r.dir);
    let t = 0.5 * unit.y + 1.0;
    let sky_gradient = (1.0-t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);

    var hit = default_hit();
    hit.c = vec4<f32>(sky_gradient, 1.0);
    return hit;
}

fn intersect_world(r: ray) -> hit {
//    for(var i: i32 = 0; i < i32(objects.sphere_count); i = i + 1 ) {
  //      let hit_result = intersect_sphere2( r, objects.spheres[i], 0.0, 2000.0 );
    //    if ( hit_result.t > 0.0 ) {
      //      return hit_sphere( r, hit_result );
//        }        
    //}

    return miss(r);
}

[[stage(compute), workgroup_size(8, 8, 1)]]
fn update([[builtin(global_invocation_id)]] invocation_id: vec3<u32>) 
{    
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let samples = i32(100);
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

	let seed = (camera.frame * u32(147565741)) * u32(720898027) * (invocation_id.x * invocation_id.y);

    for (var i: i32 = 0; i < samples; i = i + 1 ) {
        //let sample2d = random_2d_stratified_sample( u32(seed) );
        //let x = f32(invocation_id.x) - sample2d.x;
        //let y = f32(invocation_id.y) - sample2d.y;

        let x = f32(invocation_id.x) + random_float2(seed);
        let y = f32(invocation_id.y) + random_float2(seed);

        let u = x / f32(camera.render_width);
        let v = y / f32(camera.render_height);

        let horiz = vec3<f32>(4.0, 0.0, 0.0);
        let vert = vec3<f32>(0.0, -2.0, 0.0);
        let lower_left_corner = vec3<f32>(-2.0, 1.0, -1.0);

    //    let origin = camera.camera_position;//vec3<f32>(0.0, 0.0, 0.0);
        let origin = vec3<f32>(0.0, 0.0, 0.0);
        let dir = lower_left_corner + u*horiz + v*vert;

        let r = ray( origin, dir );
        let hit_result = intersect_world(r);

        color = color + hit_result.c;

        if ( hit_result.can_extend ) {
            r = hit_result.extend;
            hit_result = intersect_world(r);

            // add shade ...
        }

        //color = color + intersect_world(r);
    }
    color = color / f32(samples);

    textureStore(texture, location, color);

}
