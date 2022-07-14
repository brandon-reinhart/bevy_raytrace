use crate::ray_trace_pipeline::RayTracePipeline;
use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};

pub struct MaterialsBindGroup(pub BindGroup);

#[derive(ShaderType, Clone, Default, Debug)]
pub struct MaterialGPU {
    unknown: f32,
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct MaterialBufGPU {
    pub material_count: u32,
    #[size(runtime)]
    pub materials: Vec<MaterialGPU>,
}

#[derive(Default)]
struct MaterialGPUStorage {
    pub buffer: StorageBuffer<MaterialBufGPU>,
}

pub struct RayTraceMaterialsPlugin;

impl Plugin for RayTraceMaterialsPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<MaterialGPUStorage>()
            .add_system_to_stage(RenderStage::Prepare, prepare)
            .add_system_to_stage(RenderStage::Queue, queue);
    }
}

fn prepare(
    mut materials: ResMut<MaterialGPUStorage>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    let material_count = 1;

    // just hard code materials for now?

    if materials.buffer.get().materials.len() != material_count {
        materials.buffer.get_mut().material_count = material_count as u32;
        materials.buffer.get_mut().materials.clear();
        materials
            .buffer
            .get_mut()
            .materials
            .append(&mut vec![MaterialGPU::default(); material_count]);

        materials.buffer.write_buffer(&render_device, &render_queue);

        println!(
            "Materials Buffer: {:?} {:?}",
            material_count,
            materials.buffer.get().materials.size()
        );
    }
}

fn queue(
    mut commands: Commands,
    pipeline: Res<RayTracePipeline>,
    materials: Res<MaterialGPUStorage>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("materials_bind_group"),
        layout: &pipeline.bind_groups.materials,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: materials.buffer.binding().unwrap(),
        }],
    });

    commands.insert_resource(MaterialsBindGroup(bind_group));
}

pub fn describe<'a>() -> BindGroupLayoutDescriptor<'a> {
    BindGroupLayoutDescriptor {
        label: Some("materials_layout_descriptor"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            count: None,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }],
    }
}
