use crate::RenderTargetImage;
use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin, render_graph::RenderGraph, render_resource::*,
        RenderApp,
    },
};

use crate::ray_trace_camera::RayTraceCameraPlugin;
use crate::ray_trace_globals::RayTraceGlobalsPlugin;
use crate::ray_trace_intersection::RayTraceIntersectionsPlugin;
use crate::ray_trace_node::RayTraceNode;
use crate::ray_trace_pipeline::*;

pub struct RayTracePlugin;

impl Plugin for RayTracePlugin {
    fn build(&self, app: &mut App) {
        app//.add_plugin(ExtractResourcePlugin::<RenderTargetImage>::default())
            .add_plugin(RayTraceCameraPlugin)
            .add_plugin(RayTraceGlobalsPlugin)
            .add_plugin(RayTraceIntersectionsPlugin);
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<RayTracePipeline>();

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("raytrace", RayTraceNode::default());
        render_graph
            .add_node_edge("raytrace", bevy::render::main_graph::node::CAMERA_DRIVER)
            .unwrap();
    }
}

struct RenderTargetImageBindGroup(BindGroup);
