use crate::{RENDER_TARGET_SIZE, SAMPLES_PER_RAY};
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
    color: Vec4,
    point: Vec3,
    t: f32,
    normal: Vec3,
    material: u32,
    front_face: u32,
}

#[derive(Default)]
pub struct IntersectionGPUStorage {
    pub buffer: StorageBuffer<Vec<IntersectionGPU>>,
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
    let ray_count = (RENDER_TARGET_SIZE.0 * RENDER_TARGET_SIZE.1) as usize * SAMPLES_PER_RAY;

    if intersections.buffer.get().len() != ray_count {
        intersections.buffer.get_mut().clear();
        intersections
            .buffer
            .get_mut()
            .append(&mut vec![IntersectionGPU::default(); ray_count]);

        intersections
            .buffer
            .write_buffer(&render_device, &render_queue);

        println!(
            "Intersection Buffer: {:?} {:?}",
            ray_count,
            intersections.buffer.get().size()
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
