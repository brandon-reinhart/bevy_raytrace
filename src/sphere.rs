use crate::ray_trace_pipeline::RayTracePipeline;
use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        MainWorld, RenderApp, RenderStage,
    },
};

#[derive(Component, ShaderType, Clone, Default, Debug)]
struct SphereGPU {
    center: Vec3,
    radius: f32,
}

#[derive(ShaderType, Clone, Default, Debug)]
struct ObjectListGPU {
    sphere_count: u32,
    #[size(runtime)]
    spheres: Vec<SphereGPU>,
}

#[derive(Default)]
struct ObjectListStorage {
    pub buffer: StorageBuffer<ObjectListGPU>,
}

#[derive(Component, Default, Clone, Debug)]
pub struct Sphere {
    radius: f32,
}

pub struct ShapesBindGroup(pub BindGroup);

pub fn init_spheres(mut commands: Commands) {
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 0.0, -1.0))
        .insert(Sphere { radius: 0.5 });

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, -100.5, -1.0))
        .insert(Sphere { radius: 100.0 });
}

pub struct SphereRenderPlugin;

impl Plugin for SphereRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_spheres);

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .insert_resource(ObjectListGPU::default())
                .insert_resource(ObjectListStorage::default())
                .add_system_to_stage(RenderStage::Extract, extract)
                .add_system_to_stage(RenderStage::Prepare, prepare)
                .add_system_to_stage(RenderStage::Queue, queue);
        }
    }
}

fn extract(mut world: ResMut<MainWorld>, mut object_list: ResMut<ObjectListGPU>) {
    let mut query = world.query::<(&Sphere, &Transform)>();

    object_list.spheres.clear();

    for (sphere, transform) in query.iter(&world) {
        object_list.spheres.push(SphereGPU {
            center: transform.translation,
            radius: sphere.radius,
        });
    }
}

fn prepare(
    mut object_list: ResMut<ObjectListGPU>,
    mut object_list_storage: ResMut<ObjectListStorage>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    object_list_storage.buffer.get_mut().sphere_count = object_list.spheres.len() as u32;
    object_list_storage.buffer.get_mut().spheres.clear();
    object_list_storage
        .buffer
        .get_mut()
        .spheres
        .append(&mut object_list.spheres);

    object_list_storage
        .buffer
        .write_buffer(&render_device, &render_queue);
}

fn queue(
    mut commands: Commands,
    object_list: Res<ObjectListStorage>,
    pipeline: Res<RayTracePipeline>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.bind_groups.objects,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: object_list.buffer.binding().unwrap(),
        }],
    });

    commands.insert_resource(ShapesBindGroup(bind_group));
}

pub fn describe<'a>() -> BindGroupLayoutDescriptor<'a> {
    BindGroupLayoutDescriptor {
        label: Some("objects"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    }
}
