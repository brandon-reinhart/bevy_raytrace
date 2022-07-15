use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};
use std::borrow::Cow;

pub struct RayTraceBindGroups {
    pub camera_globals: BindGroupLayout,
    pub rays_intersections: BindGroupLayout,
    pub objects_materials: BindGroupLayout,
    pub output: BindGroupLayout,
}

pub struct RayTracePipelines {
    pub clear: CachedComputePipelineId,
    pub prepass: CachedComputePipelineId,
    pub generate: CachedComputePipelineId,
    pub intersect: CachedComputePipelineId,
    pub shade: CachedComputePipelineId,
    // connect: CachedComputePipelineId,
}

pub struct RayTracePipeline {
    pub pipelines: RayTracePipelines,
    pub bind_groups: RayTraceBindGroups,
}

impl RayTracePipeline {
    fn create_pipelines(world: &mut World, bind_groups: &RayTraceBindGroups) -> RayTracePipelines {
        RayTracePipelines {
            clear: RayTracePipeline::create_clear_pipeline(world, bind_groups),
            prepass: RayTracePipeline::create_prepass_pipeline(world, bind_groups),
            generate: RayTracePipeline::create_generate_pipeline(world, bind_groups),
            intersect: RayTracePipeline::create_intersect_pipeline(world, bind_groups),
            shade: RayTracePipeline::create_shade_pipeline(world, bind_groups),
        }
    }

    fn create_clear_pipeline(
        world: &mut World,
        bind_groups: &RayTraceBindGroups,
    ) -> CachedComputePipelineId {
        let shader = world.resource::<AssetServer>().load("shaders/clear.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("clear")),
            layout: Some(vec![
                bind_groups.camera_globals.clone(),
                bind_groups.rays_intersections.clone(),
                bind_groups.output.clone(),
            ]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        })
    }

    fn create_prepass_pipeline(
        world: &mut World,
        bind_groups: &RayTraceBindGroups,
    ) -> CachedComputePipelineId {
        let shader = world.resource::<AssetServer>().load("shaders/prepass.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("prepass")),
            layout: Some(vec![
                bind_groups.camera_globals.clone(),
                bind_groups.rays_intersections.clone(),
                bind_groups.output.clone(),
            ]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        })
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
            label: Some(Cow::from("generate")),
            layout: Some(vec![
                bind_groups.camera_globals.clone(),
                bind_groups.rays_intersections.clone(),
            ]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        })
    }

    fn create_intersect_pipeline(
        world: &mut World,
        bind_groups: &RayTraceBindGroups,
    ) -> CachedComputePipelineId {
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/intersect.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("intersect")),
            layout: Some(vec![
                bind_groups.camera_globals.clone(),
                bind_groups.rays_intersections.clone(),
                bind_groups.objects_materials.clone(),
            ]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        })
    }

    fn create_shade_pipeline(
        world: &mut World,
        bind_groups: &RayTraceBindGroups,
    ) -> CachedComputePipelineId {
        let shader = world.resource::<AssetServer>().load("shaders/shade.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("shade")),
            layout: Some(vec![
                bind_groups.camera_globals.clone(),
                bind_groups.rays_intersections.clone(),
                bind_groups.objects_materials.clone(),
                bind_groups.output.clone(),
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
            camera_globals: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("camera_globals_layout_descriptor"),
                entries: &[
                    crate::ray_trace_camera::describe(0),
                    crate::ray_trace_globals::describe(1),
                ],
            }),

            rays_intersections: render_device.create_bind_group_layout(
                &BindGroupLayoutDescriptor {
                    label: Some("rays_intersections_layout_descriptor"),
                    entries: &[
                        crate::ray_trace_rays::describe(0),
                        crate::ray_trace_intersection::describe(1),
                    ],
                },
            ),

            objects_materials: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("objects_materials_layout_descriptor"),
                entries: &[
                    crate::sphere::describe(0),
                    crate::ray_trace_materials::describe(1),
                ],
            }),

            output: render_device.create_bind_group_layout(&crate::ray_trace_output::describe()),
        };

        let pipelines = RayTracePipeline::create_pipelines(world, &bind_groups);

        RayTracePipeline {
            bind_groups,
            pipelines,
        }
    }
}
