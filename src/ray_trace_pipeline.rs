use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};
use std::borrow::Cow;

pub struct RayTraceBindGroups {
    pub camera: BindGroupLayout,
    pub globals: BindGroupLayout,
    pub rays: BindGroupLayout,
    pub intersection: BindGroupLayout,
    pub objects: BindGroupLayout,
}

pub struct RayTracePipelines {
    pub generate: CachedComputePipelineId,
    pub extend: CachedComputePipelineId,
    // shade: CachedComputePipelineId,
    // connect: CachedComputePipelineId,
}

pub struct RayTracePipeline {
    pub pipelines: RayTracePipelines,
    pub bind_groups: RayTraceBindGroups,
}

impl RayTracePipeline {
    fn create_pipelines(world: &mut World, bind_groups: &RayTraceBindGroups) -> RayTracePipelines {
        RayTracePipelines {
            generate: RayTracePipeline::create_generate_pipeline(world, bind_groups),
            extend: RayTracePipeline::create_extend_pipeline(world, bind_groups),
        }
    }

    fn create_generate_pipeline(
        world: &mut World,
        bind_groups: &RayTraceBindGroups,
    ) -> CachedComputePipelineId {
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/generate.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![
                bind_groups.camera.clone(),
                bind_groups.globals.clone(),
                bind_groups.rays.clone(),
            ]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        })
    }

    fn create_extend_pipeline(
        world: &mut World,
        bind_groups: &RayTraceBindGroups,
    ) -> CachedComputePipelineId {
        let shader = world.resource::<AssetServer>().load("shaders/extend.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![
                bind_groups.globals.clone(),
                bind_groups.rays.clone(),
                bind_groups.intersection.clone(),
                bind_groups.objects.clone(),
            ]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        })
    }
}

impl FromWorld for RayTracePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_groups = RayTraceBindGroups {
            camera: render_device.create_bind_group_layout(&crate::ray_trace_camera::describe()),

            globals: render_device
                .create_bind_group_layout(&crate::ray_trace_globals::describe_globals()),

            rays: render_device
                .create_bind_group_layout(&crate::ray_trace_globals::describe_rays()),

            intersection: render_device
                .create_bind_group_layout(&crate::ray_trace_intersection::describe()),

            objects: render_device.create_bind_group_layout(&crate::sphere::describe()),
        };

        let pipelines = RayTracePipeline::create_pipelines(world, &bind_groups);

        RayTracePipeline {
            bind_groups,
            pipelines,
        }
    }
}

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

*/
