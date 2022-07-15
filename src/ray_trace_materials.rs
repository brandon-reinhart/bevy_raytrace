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
}

// These conversion functions are probably temporary.

impl From<i32> for Reflectance {
    fn from(other: i32) -> Self {
        match other {
            0 => Reflectance::Lambertian,
            _ => Reflectance::Metallic,
        }
    }
}

impl Into<i32> for Reflectance {
    fn into(self) -> i32 {
        match self {
            Reflectance::Lambertian => 0,
            _ => 1,
        }
    }
}

impl Default for Reflectance {
    fn default() -> Self {
        Reflectance::Lambertian
    }
}

#[derive(Default, Clone, Debug)]
pub struct RayTraceMaterial {
    pub reflectance: Reflectance,
    pub color: Color,
}

impl From<MaterialGPU> for RayTraceMaterial {
    fn from(other: MaterialGPU) -> Self {
        RayTraceMaterial {
            color: Color::rgb(other.color.x, other.color.y, other.color.z),
            reflectance: other.reflectance.into(),
        }
    }
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct MaterialGPU {
    color: Vec4,
    reflectance: i32,
    pad0: i32,
    pad1: i32,
    pad2: i32,
}

impl From<RayTraceMaterial> for MaterialGPU {
    fn from(other: RayTraceMaterial) -> Self {
        MaterialGPU {
            color: Vec4::new(
                other.color.r(),
                other.color.g(),
                other.color.b(),
                other.color.a(),
            ),
            reflectance: other.reflectance.into(),
            pad0: 0,
            pad1: 0,
            pad2: 0,
        }
    }
}

#[derive(ShaderType, Clone, Default, Debug)]
pub struct MaterialBufGPU {
    pub material_count: u32,
    #[size(runtime)]
    pub materials: Vec<MaterialGPU>,
}

#[derive(Default)]
pub struct MaterialGPUStorage {
    pub buffer: StorageBuffer<MaterialBufGPU>,
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
        },
    );

    cache.materials.insert(
        "center".to_string(),
        RayTraceMaterial {
            reflectance: Reflectance::Lambertian,
            color: Color::rgba(0.0, 1.0, 0.0, 1.0),
        },
    );

    cache.materials.insert(
        "left".to_string(),
        RayTraceMaterial {
            reflectance: Reflectance::Metallic,
            color: Color::rgba(1.0, 0.0, 0.0, 1.0),
        },
    );
    
    cache.materials.insert(
        "right".to_string(),
        RayTraceMaterial {
            reflectance: Reflectance::Metallic,
            color: Color::rgba(0.0, 0.0, 1.0, 1.0),
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
    if materials.buffer.get().materials.len() != material_count {
        materials.buffer.get_mut().material_count = material_count as u32;
        materials.buffer.get_mut().materials.clear();

        for (_, mat) in cache.materials.iter() {
            materials.buffer.get_mut().materials.push(MaterialGPU {
                reflectance: mat.reflectance.clone().into(),
                color: Vec4::new(mat.color.r(), mat.color.g(), mat.color.b(), mat.color.a()),
                pad0: 0,
                pad1: 0,
                pad2: 0,
            });
        }

        materials.buffer.write_buffer(&render_device, &render_queue);

        println!(
            "Materials Buffer: {:?} {:?} {:?}",
            material_count,
            materials.buffer.get().materials.size(),
            materials.buffer.get().materials
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
