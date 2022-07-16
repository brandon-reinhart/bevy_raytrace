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

@group(0) @binding(0)
var<uniform> camera: camera_config;

@group(0) @binding(1)
var<storage, read_write> globals: globals_buf;

@group(1) @binding(0)
var<storage, read_write> ray_buffer: ray_buf;

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

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let index = atomicAdd( &globals.generate_index, 1u );
    if ( index >= ray_buffer.ray_count ) {
        return;
    }

    let x = index % globals.render_width;
    let y = (index / globals.render_width) % globals.render_height;

    // What is the origin of these bit masks?
    // todo: What is a standard way of generated a random seed?
	let seed = (globals.frame * u32(147565741)) * u32(720898027) * index;

    // Get a stratified point inside the pixel?
    // todo: Read about good techniques for determining the ray.
    // For now, a simple (bad) approach:
    let x = f32(x);// + random_float2(seed) / 1000.0;
    let y = f32(y);// + random_float2(seed) / 1000.0;

	let normalized_i = ( x / f32(globals.render_width) ) - 0.5;
    let normalized_j = ( ( f32(globals.render_height) - y ) / f32(globals.render_height) ) - 0.5;

    var dir_to_focal_plane = camera.camera_forward + normalized_i * camera.camera_right + normalized_j * camera.camera_up;
    dir_to_focal_plane = normalize( dir_to_focal_plane );
    let convergence_point = camera.camera_position + dir_to_focal_plane;

    let origin = camera.camera_position;// + camera.camera_right * lens.x + camera.camera_up * lens.y;
    let direction = normalize( convergence_point - origin );

    let pixel = u32( y * f32(globals.render_width) + x );
    var r = ray( origin, direction, pixel, 0u );

    storageBarrier();
    ray_buffer.rays[index] = r;
}   