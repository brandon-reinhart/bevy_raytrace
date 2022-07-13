use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice},
    },
};
use std::borrow::Cow;

pub struct RayTraceBindGroups {
    pub ray_buffer: BindGroupLayout,
    pub globals: BindGroupLayout,
}

pub struct RayTracePipelines {
    pub generate: CachedComputePipelineId,
    // extend: CachedComputePipelineId,
    // shade: CachedComputePipelineId,
    // connect: CachedComputePipelineId,
}

pub struct RayTracePipeline {
    pub pipelines: RayTracePipelines,
    pub bind_groups: RayTraceBindGroups,
}

impl RayTracePipeline {
    fn create_generate_pipeline(
        world: &mut World,
    ) -> (BindGroupLayout, BindGroupLayout, CachedComputePipelineId) {
        let generate_shader = world
            .resource::<AssetServer>()
            .load("shaders/generate.wgsl");

        let ray_buffer =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    }],
                });

        let globals =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    }],
                });

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let generate_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![ray_buffer.clone(), globals.clone()]),
            shader: generate_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        });

        (ray_buffer, globals, generate_pipeline)
    }
}

impl FromWorld for RayTracePipeline {

    fn from_world(world: &mut World) -> Self {
        let (ray_buffer, globals, generate) = RayTracePipeline::create_generate_pipeline(world);
/*
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let shapes_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
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
                });

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/raytrace.wgsl");
*/                
        RayTracePipeline {
            pipelines: RayTracePipelines { 
                generate,
            },
            bind_groups: RayTraceBindGroups {
                ray_buffer,
                globals,
            },
        }
    }
}