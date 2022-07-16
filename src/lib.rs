mod camera;
mod input;
mod plugin;
mod ray_trace_camera;
mod ray_trace_globals;
mod ray_trace_intersection;
mod ray_trace_materials;
mod ray_trace_node;
mod ray_trace_output;
mod ray_trace_pipeline;
mod ray_trace_rays;
mod sphere;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{WindowDescriptor, WindowMode},
};

use camera::CameraPlugin;
use input::InputPlugin;
use plugin::RayTracePlugin;
use sphere::SphereRenderPlugin;

pub const RENDER_TARGET_SIZE: (u32, u32) = (1920, 1080);
pub const SAMPLES_PER_RAY: usize = 1;

pub fn entry() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "bevy_raytrace".to_string(),
            width: RENDER_TARGET_SIZE.0 as f32,
            height: RENDER_TARGET_SIZE.1 as f32,
            resizable: true,
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgba(0.35, 0.35, 0.35, 1.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(CameraPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(RayTracePlugin)
        .add_plugin(SphereRenderPlugin)
        .add_startup_system(init_camera)
        .run();
}

pub fn init_camera(mut commands: Commands) {
    // A bevy camera that simply stares at the origin and our render target sprite.
    // This will never move and is not the ray trace camera.
    commands.spawn_bundle(Camera2dBundle::default());
}
