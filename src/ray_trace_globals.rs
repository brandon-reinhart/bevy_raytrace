use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};

#[derive(ShaderType, Clone, Default, Debug)]
pub struct GlobalsGPU {
    pub ray_index: u32,
}

#[derive(Default)]
pub struct GlobalsGPUStorage {
    pub buffer: StorageBuffer<GlobalsGPU>,
}

pub struct RayTraceGlobalsPlugin;

impl Plugin for RayTraceGlobalsPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GlobalsGPU>()
            .init_resource::<GlobalsGPUStorage>()
            .add_system_to_stage(RenderStage::Prepare, prepare);
    }
}

fn prepare(
    mut globals: ResMut<GlobalsGPUStorage>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    globals.buffer.get_mut().ray_index = 0;

    globals.buffer.write_buffer(&render_device, &render_queue);
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
