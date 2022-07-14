use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};

use crate::ray_trace_pipeline::RayTracePipeline;
use crate::RENDER_TARGET_SIZE;

pub struct GlobalsBindGroup(pub BindGroup);
pub struct RayBufBindGroup(pub BindGroup);

#[derive(ShaderType, Clone, Default, Debug)]
pub struct RayGPU {
    origin: Vec3,
    dir: Vec3,
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct GlobalsGPU {
    pub ray_index: u32,
}

#[derive(Default)]
struct GlobalsGPUStorage {
    pub buffer: StorageBuffer<GlobalsGPU>,
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct RayBufGPU {
    pub ray_count: u32,
    #[size(runtime)]
    pub rays: Vec<RayGPU>,
}

#[derive(Default)]
struct RayBufGPUStorage {
    pub buffer: StorageBuffer<RayBufGPU>,
}

pub struct RayTraceGlobalsPlugin;

impl Plugin for RayTraceGlobalsPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GlobalsGPU>()
            .init_resource::<GlobalsGPUStorage>()
            .add_system_to_stage(RenderStage::Prepare, prepare_globals)
            .add_system_to_stage(RenderStage::Queue, queue_globals)
            .init_resource::<RayBufGPUStorage>()
            .add_system_to_stage(RenderStage::Prepare, prepare_ray_buf)
            .add_system_to_stage(RenderStage::Queue, queue_ray_buf);
    }
}

fn prepare_globals(
    mut globals: ResMut<GlobalsGPUStorage>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    globals.buffer.get_mut().ray_index = 0;

    globals.buffer.write_buffer(&render_device, &render_queue);
}

fn queue_globals(
    mut commands: Commands,
    pipeline: Res<RayTracePipeline>,
    globals: Res<GlobalsGPUStorage>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.bind_groups.globals,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: globals.buffer.binding().unwrap(),
        }],
    });

    commands.insert_resource(GlobalsBindGroup(bind_group));
}

pub fn describe_globals<'a>() -> BindGroupLayoutDescriptor<'a> {
    BindGroupLayoutDescriptor {
        label: Some("globals"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            count: None,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }],
    }
}

fn prepare_ray_buf(
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
        //.append(&mut Vec::with_capacity(ray_count));

        ray_buf.buffer.write_buffer(&render_device, &render_queue);

        println!(
            "Ray Buffer: {:?} {:?}",
            ray_count,
            ray_buf.buffer.get().rays.size()
        );
    }
}

fn queue_ray_buf(
    mut commands: Commands,
    pipeline: Res<RayTracePipeline>,
    ray_buf: Res<RayBufGPUStorage>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.bind_groups.rays,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: ray_buf.buffer.binding().unwrap(),
        }],
    });

    commands.insert_resource(RayBufBindGroup(bind_group));
}

pub fn describe_rays<'a>() -> BindGroupLayoutDescriptor<'a> {
    BindGroupLayoutDescriptor {
        label: Some("rays"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            count: None,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }],
    }
}
