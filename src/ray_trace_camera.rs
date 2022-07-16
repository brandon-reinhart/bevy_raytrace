use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};

use crate::camera::RayTraceCamera;

const CAMERA_FOV: f32 = 1.5708;

#[derive(Copy, Clone, Debug, ShaderType)]
pub struct CameraGPU {
    pub transform: Mat4,
    pub forward: Vec3,
    pub fov: f32,
    pub up: Vec3,
    pub image_plane_distance: f32,
    pub right: Vec3,
    pub lens_focal_length: f32,
    pub position: Vec3,
    pub fstop: f32,
}

#[derive(Default)]
pub struct CameraGPUStorage {
    pub buffer: DynamicUniformBuffer<CameraGPU>,
}

pub struct RayTraceCameraPlugin;

impl Plugin for RayTraceCameraPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<CameraGPUStorage>()
            .add_system_to_stage(RenderStage::Prepare, prepare);
    }
}

fn prepare(
    camera: Res<RayTraceCamera>,
    mut camera_gpu: ResMut<CameraGPUStorage>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    camera_gpu.buffer.clear();

    let transform = camera.transform;

    camera_gpu.buffer.push(CameraGPU {
        transform: transform.compute_matrix(),
        forward: transform.forward(),
        up: transform.up(),
        right: transform.right(),
        position: transform.translation,
        fov: CAMERA_FOV,
        image_plane_distance: 10.0,
        lens_focal_length: 0.1, // millimeters
        fstop: 1.0 / 32.0,
    });

    camera_gpu
        .buffer
        .write_buffer(&render_device, &render_queue);
}

pub fn describe(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
