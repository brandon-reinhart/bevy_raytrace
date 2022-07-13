struct ray {
    origin: vec3<f32>;
    dir: vec3<f32>;
};

struct ray_buf {
    ray_count: u32;
    rays: array<ray>;
};

struct globals_buf {
    ray_index: atomic<u32>;
};

[[group(0), binding(0)]]
var<storage, read_write> ray_buffer: ray_buf;

[[group(1), binding(0)]]
var<storage, read_write> globals: globals_buf;

[[stage(compute), workgroup_size(128, 1, 1)]]
fn main()
{
    let index = atomicAdd( &globals.ray_index, u32(1) );

    if ( index >= ray_buffer.ray_count ) {
        return;
    }
}