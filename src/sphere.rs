use crate::plugin::RayTracePipeline;
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
                .add_system_to_stage(RenderStage::Extract, extract_spheres)
                .add_system_to_stage(RenderStage::Prepare, prepare_spheres)
                .add_system_to_stage(RenderStage::Queue, queue_spheres);
        }
    }
}

fn extract_spheres(mut world: ResMut<MainWorld>, mut object_list: ResMut<ObjectListGPU>) {
    let mut query = world.query::<(&Sphere, &Transform)>();

    object_list.spheres.clear();

    for (sphere, transform) in query.iter(&world) {
        object_list.spheres.push(SphereGPU {
            center: transform.translation,
            radius: sphere.radius,
        });
    }
}

fn prepare_spheres(
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

fn queue_spheres(
    mut commands: Commands,
    object_list_storage: Res<ObjectListStorage>,
    pipeline: Res<RayTracePipeline>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.shapes_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: object_list_storage.buffer.binding().unwrap(),
        }],
    });

    commands.insert_resource(ShapesBindGroup(bind_group));
}
