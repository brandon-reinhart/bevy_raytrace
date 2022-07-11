struct Uniforms {
    render_width: u32;
    render_height: u32;

    camera_forward: vec3<f32>;
    camera_up: vec3<f32>;
    camera_right: vec3<f32>;
    camera_position: vec3<f32>;
};

[[group(0), binding(0)]]
var texture: texture_storage_2d<rgba32float, read_write>;

[[group(0), binding(1)]]
var<uniform> camera: Uniforms;

struct ray {
    origin: vec3<f32>;
    dir: vec3<f32>;
};

fn intersect(r: ray, t: f32) -> vec3<f32> {
    return r.origin + r.dir * t;
}


[[stage(compute), workgroup_size(8, 8, 1)]]
fn init() {

}

fn miss(r: ray) -> vec4<f32> {
    let unit = normalize(r.dir);
    let t = 0.5 * unit.y + 1.0;
    let sky_gradient = (1.0-t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);

    return vec4<f32>(sky_gradient, 1.0);
}

[[stage(compute), workgroup_size(8, 8, 1)]]
fn update([[builtin(global_invocation_id)]] invocation_id: vec3<u32>) 
{    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let x = f32(invocation_id.x);
    let y = f32(invocation_id.y);

    let u = x / f32(camera.render_width);
    let v = y / f32(camera.render_height);

    let horiz = vec3<f32>(4.0, 0.0, 0.0);
    let vert = vec3<f32>(0.0, 2.0, 0.0);

    let origin = vec3<f32>(0.0, 0.0, 0.0);
    let dir = vec3<f32>(-2.0, -1.0, -1.0) + u*horiz + v*vert;

    let r = ray( origin, dir );
    let color = miss(r);

    textureStore(texture, location, color);
}


