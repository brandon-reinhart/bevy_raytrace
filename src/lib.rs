mod camera;
mod plugin;
mod sphere;

use bevy::{
    prelude::*,
    render::{extract_resource::ExtractResource, render_resource::*},
    window::{WindowDescriptor, WindowResized},
};

use camera::CameraPlugin;
use plugin::RayTracePlugin;
use sphere::SphereRenderPlugin;

pub const RENDER_TARGET_SIZE: (u32, u32) = (1024, 1024);

#[derive(Component)]
pub struct RenderTarget;

pub fn entry() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "bevy_raytrace".to_string(),
            resizable: true,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgba(0.35, 0.35, 0.35, 1.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(RayTracePlugin)
        .add_plugin(SphereRenderPlugin)
        //.add_plugin(ExtractComponentPlugin::<sphere::Sphere>::default())
        //.add_pluign(UniformComponentPlugin::<sphere::Sphere>::default())
        .add_startup_system(init_camera)
        .add_startup_system(init_render_target)
        //        .add_startup_system(sphere::init_spheres)
        .add_system(on_window_resized)
        .run();
}

pub fn init_camera(mut commands: Commands) {
    // A bevy camera that simply stares at the origin and our render target sprite.
    // This will never move and is not the ray trace camera.
    commands.spawn_bundle(Camera2dBundle::default());
}

fn vf_to_u8(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}

fn init_render_target(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Create an image of the size of the screen and attach it to a sprite with the same size.
    // This will become the render target for the compute pipeline.
    // todo: resize the render target and sprite when the screen is resized.

    let fill = vec![0f32, 0f32, 0f32, 1f32];
    let fill = vf_to_u8(&fill[..]);

    let mut image = Image::new_fill(
        Extent3d {
            width: RENDER_TARGET_SIZE.0,
            height: RENDER_TARGET_SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        fill,
        TextureFormat::Rgba32Float,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image = images.add(image);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(
                    RENDER_TARGET_SIZE.0 as f32,
                    RENDER_TARGET_SIZE.1 as f32,
                )),
                ..default()
            },
            texture: image.clone(),
            ..default()
        })
        .insert(RenderTarget);

    commands.insert_resource(RenderTargetImage(image));
}

fn on_window_resized(
    mut event: EventReader<WindowResized>,
    mut query: Query<&mut Sprite, With<RenderTarget>>,
) {
    if event.is_empty() || query.is_empty() {
        return;
    }

    // Resize the sprite to fit the window.
    for e in event.iter() {
        let mut sprite = query.single_mut();
        sprite.custom_size = Some(Vec2::new(e.width, e.height));
    }

    // Also need to resize the render target to some aspect ratio of the sprite size...
}

#[derive(Clone, Deref, ExtractResource)]
pub struct RenderTargetImage(Handle<Image>);
