use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};
use indexmap::IndexMap;

#[derive(Clone, Debug)]
pub enum Reflectance {
    Lambertian,
    Metallic,
    Dielectric,
}

impl Default for Reflectance {
    fn default() -> Self {
        Reflectance::Lambertian
    }
}

#[derive(Default, Clone, Debug)]
pub struct RayTraceMaterial {
    pub color: Color,
    pub reflectance: Reflectance,
    pub fuzziness: f32,
    pub index_of_refraction: f32,
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct MaterialGPU {
    color: Vec4,
    reflectance: i32,
    fuzziness: f32,
    index_of_refraction: f32,
    pad2: i32,
}

#[derive(Default)]
pub struct MaterialGPUStorage {
    pub buffer: StorageBuffer<Vec<MaterialGPU>>,
}

#[derive(Default, Clone, ExtractResource)]
pub struct MaterialCache {
    materials: IndexMap<String, RayTraceMaterial>,
}

impl MaterialCache {
    pub fn get(&self, key: &str) -> RayTraceMaterial {
        self.materials.get(key).unwrap().clone()
    }

    pub fn get_index_of(&self, key: &str) -> u32 {
        self.materials.get_index_of(key).unwrap() as u32
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }
}

pub struct RayTraceMaterialsPlugin;

impl Plugin for RayTraceMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(init_materials_cache())
            .add_plugin(ExtractResourcePlugin::<MaterialCache>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<MaterialGPUStorage>()
            .add_system_to_stage(RenderStage::Prepare, prepare);
    }
}

fn init_materials_cache() -> MaterialCache {
    let mut cache = MaterialCache::default();

    cache.materials.insert(
        "ground".to_string(),
        RayTraceMaterial {
            reflectance: Reflectance::Lambertian,
            color: Color::rgba(0.8, 0.8, 0.0, 1.0),
            fuzziness: 1.0,
            index_of_refraction: 0.0,
        },
    );

    cache.materials.insert(
        "center".to_string(),
        RayTraceMaterial {
            reflectance: Reflectance::Lambertian,
            color: Color::rgba(0.7, 0.3, 0.3, 1.0),
            fuzziness: 1.0,
            index_of_refraction: 0.0,
        },
    );

    cache.materials.insert(
        "left".to_string(),
        RayTraceMaterial {
            reflectance: Reflectance::Metallic,
            color: Color::rgba(0.8, 0.8, 0.8, 1.0),
            fuzziness: 0.05,
            index_of_refraction: 0.0,
        },
    );

    cache.materials.insert(
        "right".to_string(),
        RayTraceMaterial {
            reflectance: Reflectance::Metallic,
            color: Color::rgba(0.8, 0.6, 0.2, 1.0),
            fuzziness: 0.4,
            index_of_refraction: 1.5,
        },
    );

    cache
}

fn prepare(
    cache: Res<MaterialCache>,
    mut materials: ResMut<MaterialGPUStorage>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    let material_count = cache.len();

    // At the moment, the cache only grows so this is okay...
    if materials.buffer.get().len() != material_count {
        //materials.buffer.get_mut().material_count = material_count as u32;
        materials.buffer.get_mut().clear();

        for (_, mat) in cache.materials.iter() {
            materials.buffer.get_mut().push(MaterialGPU {
                reflectance: match mat.reflectance {
                    Reflectance::Lambertian => 0,
                    Reflectance::Metallic => 1,
                    Reflectance::Dielectric => 2,
                },
                color: Vec4::new(mat.color.r(), mat.color.g(), mat.color.b(), mat.color.a()),
                fuzziness: mat.fuzziness,
                index_of_refraction: mat.index_of_refraction,
                pad2: 0,
            });
        }

        materials.buffer.write_buffer(&render_device, &render_queue);

        println!(
            "Materials Buffer: {:?} {:?}",
            material_count,
            materials.buffer.get().size(),
        );
    }
}

pub fn describe(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        count: None,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
    }
}
