use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};

use crate::RENDER_TARGET_SIZE;

#[derive(ShaderType, Clone, Default, Debug)]
pub struct RayGPU {
    origin: Vec3,
    dir: Vec3,
    pixel: u32,
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct RayBufGPU {
    pub ray_count: u32,
    #[size(runtime)]
    pub rays: Vec<RayGPU>,
}

#[derive(Default)]
pub struct RayBufGPUStorage {
    pub buffer: StorageBuffer<RayBufGPU>,
}

pub struct RayTraceRaysPlugin;

impl Plugin for RayTraceRaysPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<RayBufGPUStorage>()
            .add_system_to_stage(RenderStage::Prepare, prepare);
    }
}

fn prepare(
    mut ray_buf: ResMut<RayBufGPUStorage>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    // How many rays should we need?
    let ray_count = (RENDER_TARGET_SIZE.0 * RENDER_TARGET_SIZE.1) as usize;

    // Only re-allocate this buffer if the number of rays changed.
    if ray_buf.buffer.get().rays.len() != ray_count {
        ray_buf.buffer.get_mut().ray_count = ray_count as u32;
        ray_buf.buffer.get_mut().rays.clear();
        ray_buf
            .buffer
            .get_mut()
            .rays
            .append(&mut vec![RayGPU::default(); ray_count]);

        ray_buf.buffer.write_buffer(&render_device, &render_queue);

        println!(
            "Ray Buffer: {:?} {:?}",
            ray_count,
            ray_buf.buffer.get().rays.size()
        );
    }
}

pub fn describe(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        count: None,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
    }
}
