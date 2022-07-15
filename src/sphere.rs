use crate::ray_trace_materials::MaterialCache;
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
    material: u32,
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct ObjectListGPU {
    sphere_count: u32,
    #[size(runtime)]
    spheres: Vec<SphereGPU>,
}

#[derive(Default)]
pub struct ObjectListStorage {
    pub buffer: StorageBuffer<ObjectListGPU>,
}

#[derive(Component, Default, Clone, Debug)]
pub struct Sphere {
    radius: f32,
    material: u32,
}

pub fn init_spheres(mut commands: Commands, materials: Res<MaterialCache>) {
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, -100.5, -1.0))
        .insert(Sphere {
            radius: 100.0,
            material: materials.get_index_of("ground"),
        });

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 0.0, -1.0))
        .insert(Sphere {
            radius: 0.5,
            material: materials.get_index_of("center"),
        });

    commands
        .spawn()
        .insert(Transform::from_xyz(-1.0, 0.0, -1.0))
        .insert(Sphere {
            radius: 0.5,
            material: materials.get_index_of("left"),
        });

    commands
        .spawn()
        .insert(Transform::from_xyz(1.0, 0.0, -1.0))
        .insert(Sphere {
            radius: 0.5,
            material: materials.get_index_of("right"),
        });
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
                .add_system_to_stage(RenderStage::Prepare, prepare);
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
            material: sphere.material,
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

pub fn describe(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
