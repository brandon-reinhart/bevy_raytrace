use crate::RENDER_TARGET_SIZE;
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::extract_resource::{ExtractResource, ExtractResourcePlugin},
};

const CAMERA_SPEED: f32 = 10.0;

// The ray trace camera will be extracted to the render world
// so we can use it to maintain and update a uniform buffer.
// This is not a Bevy camera bundle or entity.
#[derive(Clone, ExtractResource)]
pub struct RayTraceCamera {
    pub render_width: u32, // These should live elsewhere...
    pub render_height: u32,

    pub transform: Transform,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractResourcePlugin::<RayTraceCamera>::default())
            .add_startup_system(setup)
            .add_system(update);
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(RayTraceCamera {
        render_width: RENDER_TARGET_SIZE.0,
        render_height: RENDER_TARGET_SIZE.1,
        transform: Transform::from_xyz(0., 5., 5.).looking_at(Vec3::ZERO, Vec3::Y),
    });
}

fn update(
    time: Res<Time>,
    mut mouse_motion: EventReader<MouseMotion>,
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut rt_camera: ResMut<RayTraceCamera>,
) {
    let mut camera_transform = &mut rt_camera.transform;
    let delta_time = time.delta_seconds();

    if keys.pressed(KeyCode::W) {
        let forward = camera_transform.forward() * CAMERA_SPEED * delta_time;
        camera_transform.translation += forward;
    }

    if keys.pressed(KeyCode::A) {
        let left = camera_transform.left() * CAMERA_SPEED * delta_time;
        camera_transform.translation += left;
    }

    if keys.pressed(KeyCode::S) {
        let back = camera_transform.back() * CAMERA_SPEED * delta_time;
        camera_transform.translation += back;
    }

    if keys.pressed(KeyCode::D) {
        let right = camera_transform.right() * CAMERA_SPEED * delta_time;
        camera_transform.translation += right;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        for motion in mouse_motion.iter() {
            let yaw = Quat::from_rotation_y(-motion.delta.x * delta_time);
            let pitch = Quat::from_rotation_x(-motion.delta.y * delta_time);
            camera_transform.rotation = yaw * camera_transform.rotation;
            camera_transform.rotation = camera_transform.rotation * pitch;
        }
    }
}
