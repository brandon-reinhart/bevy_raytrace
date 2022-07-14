use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};

use crate::camera::RayTraceCamera;

use crate::RENDER_TARGET_SIZE;

//pub struct CameraBindGroup(pub BindGroup);

#[derive(Copy, Clone, Debug, ShaderType)]
pub struct CameraGPU {
    pub frame: u32,
    pub render_width: u32,
    pub render_height: u32,

    pub camera_forward: Vec4,
    pub camera_up: Vec4,
    pub camera_right: Vec4,
    pub camera_position: Vec4,
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
    mut frame: Local<u32>,
) {
    *frame += 1;

    camera_gpu.buffer.clear();

    let transform = camera.transform;

    camera_gpu.buffer.push(CameraGPU {
        frame: *frame,
        render_width: RENDER_TARGET_SIZE.0,
        render_height: RENDER_TARGET_SIZE.1,
        camera_forward: transform.forward().xyzz(),
        camera_up: transform.up().xyzz(),
        camera_right: transform.right().xyzz(),
        camera_position: transform.translation.xyzz(),
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
