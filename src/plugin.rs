use bevy::{
    prelude::*,
    render::{
        render_graph::RenderGraph, render_resource::*, renderer::RenderDevice, RenderApp,
        RenderStage,
    },
};

use crate::ray_trace_camera::{CameraGPUStorage, RayTraceCameraPlugin};
use crate::ray_trace_globals::{GlobalsGPUStorage, RayTraceGlobalsPlugin};
use crate::ray_trace_intersection::{IntersectionGPUStorage, RayTraceIntersectionsPlugin};
use crate::ray_trace_materials::RayTraceMaterialsPlugin;
use crate::ray_trace_node::RayTraceNode;
use crate::ray_trace_output::RayTraceOutputPlugin;
use crate::ray_trace_pipeline::*;
use crate::ray_trace_rays::{RayBufGPUStorage, RayTraceRaysPlugin};

pub struct RayTracePlugin;

pub struct CameraGlobalsBindGroup(pub BindGroup);

pub struct RaysIntersectionsBindGroup(pub BindGroup);

impl Plugin for RayTracePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RayTraceCameraPlugin)
            .add_plugin(RayTraceGlobalsPlugin)
            .add_plugin(RayTraceRaysPlugin)
            .add_plugin(RayTraceIntersectionsPlugin)
            .add_plugin(RayTraceMaterialsPlugin)
            .add_plugin(RayTraceOutputPlugin);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<RayTracePipeline>()
            .add_system_to_stage(RenderStage::Queue, queue_camera_globals)
            .add_system_to_stage(RenderStage::Queue, queue_rays_intersections);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("raytrace", RayTraceNode::default());
        render_graph
            .add_node_edge("raytrace", bevy::render::main_graph::node::CAMERA_DRIVER)
            .unwrap();
    }
}

fn queue_camera_globals(
    mut commands: Commands,
    pipeline: Res<RayTracePipeline>,
    camera: Res<CameraGPUStorage>,
    globals: Res<GlobalsGPUStorage>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("camera_globals_bind_group"),
        layout: &pipeline.bind_groups.camera_globals,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: camera.buffer.binding().unwrap(),
            },
            BindGroupEntry {
                binding: 1,
                resource: globals.buffer.binding().unwrap(),
            },
        ],
    });

    commands.insert_resource(CameraGlobalsBindGroup(bind_group));
}

fn queue_rays_intersections(
    mut commands: Commands,
    pipeline: Res<RayTracePipeline>,
    rays: Res<RayBufGPUStorage>,
    intersections: Res<IntersectionGPUStorage>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("rays_intersections_bind_group"),
        layout: &pipeline.bind_groups.rays_intersections,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: rays.buffer.binding().unwrap(),
            },
            BindGroupEntry {
                binding: 1,
                resource: intersections.buffer.binding().unwrap(),
            },
        ],
    });

    commands.insert_resource(RaysIntersectionsBindGroup(bind_group));
}
