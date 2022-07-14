use crate::RENDER_TARGET_SIZE;
use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};

#[derive(ShaderType, Clone, Default, Debug)]
pub struct IntersectionGPU {
    t: f32,
    point: Vec3,
    normal: Vec3,
    material: u32,
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct IntersectionBufGPU {
    pub intersection_count: u32,
    #[size(runtime)]
    pub intersections: Vec<IntersectionGPU>,
}

#[derive(Default)]
pub struct IntersectionGPUStorage {
    pub buffer: StorageBuffer<IntersectionBufGPU>,
}

pub struct RayTraceIntersectionsPlugin;

impl Plugin for RayTraceIntersectionsPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<IntersectionGPUStorage>()
            .add_system_to_stage(RenderStage::Prepare, prepare);
    }
}

fn prepare(
    mut intersections: ResMut<IntersectionGPUStorage>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    // Allocate as many intersections as we have rays.
    let ray_count = (RENDER_TARGET_SIZE.0 * RENDER_TARGET_SIZE.1) as usize;

    if intersections.buffer.get().intersections.len() != ray_count {
        intersections.buffer.get_mut().intersection_count = ray_count as u32;
        intersections.buffer.get_mut().intersections.clear();
        intersections
            .buffer
            .get_mut()
            .intersections
            .append(&mut vec![IntersectionGPU::default(); ray_count]);

        intersections
            .buffer
            .write_buffer(&render_device, &render_queue);

        println!(
            "Intersection Buffer: {:?} {:?}",
            ray_count,
            intersections.buffer.get().intersections.size()
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
