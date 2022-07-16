use crate::ray_trace_materials::{MaterialCache, RayTraceMaterial, Reflectance};
use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        MainWorld, RenderApp, RenderStage,
    },
};
use rand::Rng;

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

pub fn init_spheres(mut commands: Commands, mut materials: ResMut<MaterialCache>) {
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, -1000.0, -1.0))
        .insert(Sphere {
            radius: 1000.0,
            material: materials.get_index_of("ground"),
        });

    let mut rng = rand::thread_rng();

    let sphere_dim = 7;

    for a in -sphere_dim..sphere_dim {
        for b in -sphere_dim..sphere_dim {
            //            auto choose_mat = random_double();
            let center = Vec3::new(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );
            if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let material_name = format!("material_{}_{}", a, b);

                if rng.gen::<f32>() < 0.8 {
                    materials.materials.insert(
                        material_name.clone(),
                        RayTraceMaterial {
                            reflectance: Reflectance::Lambertian,
                            color: Color::rgba(
                                rng.gen::<f32>(),
                                rng.gen::<f32>(),
                                rng.gen::<f32>(),
                                1.0,
                            ),
                            fuzziness: 1.0,
                            index_of_refraction: 0.0,
                        },
                    );
                } else {
                    materials.materials.insert(
                        material_name.clone(),
                        RayTraceMaterial {
                            reflectance: Reflectance::Metallic,
                            color: Color::rgba(
                                rng.gen::<f32>(),
                                rng.gen::<f32>(),
                                rng.gen::<f32>(),
                                1.0,
                            ),
                            fuzziness: rng.gen::<f32>() * 0.5,
                            index_of_refraction: 0.0,
                        },
                    );
                }

                commands
                    .spawn()
                    .insert(Transform::from_translation(center))
                    .insert(Sphere {
                        radius: 0.2,
                        material: materials.get_index_of(&material_name),
                    });

                /*
                shared_ptr<material> sphere_material;

                if (choose_mat < 0.8) {
                    // diffuse
                    auto albedo = color::random() * color::random();
                    sphere_material = make_shared<lambertian>(albedo);
                    world.add(make_shared<sphere>(center, 0.2, sphere_material));
                } else if (choose_mat < 0.95) {
                    // metal
                    auto albedo = color::random(0.5, 1);
                    auto fuzz = random_double(0, 0.5);
                    sphere_material = make_shared<metal>(albedo, fuzz);
                    world.add(make_shared<sphere>(center, 0.2, sphere_material));
                } else {
                    // glass
                    sphere_material = make_shared<dielectric>(1.5);
                    world.add(make_shared<sphere>(center, 0.2, sphere_material));
                }
                */
            }
        }
    }

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 1.0, 0.0))
        .insert(Sphere {
            radius: 1.0,
            material: materials.get_index_of("center"),
        });

    commands
        .spawn()
        .insert(Transform::from_xyz(-4.0, 1.0, 0.0))
        .insert(Sphere {
            radius: 1.0,
            material: materials.get_index_of("left"),
        });

    commands
        .spawn()
        .insert(Transform::from_xyz(4.0, 1.0, 0.0))
        .insert(Sphere {
            radius: 1.0,
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
