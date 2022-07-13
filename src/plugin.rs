use crate::RenderTargetImage;
use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin,
        render_asset::RenderAssets,
        render_graph::{RenderGraph},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};

use crate::ray_trace_pipeline::*;
use crate::ray_trace_node::RayTraceNode;
use crate::ray_trace_globals::{RayTraceGlobalsPlugin};

const WORKGROUP_SIZE: u32 = 8;
use crate::camera::RayTraceCamera;
use crate::RENDER_TARGET_SIZE;

#[derive(Copy, Clone, Debug, ShaderType)]
pub struct RayTraceUniformRaw {
    pub frame: u32,
    pub render_width: u32,
    pub render_height: u32,

    pub camera_forward: Vec4,
    pub camera_up: Vec4,
    pub camera_right: Vec4,
    pub camera_position: Vec4,
}

#[derive(Default)]
pub struct RayTraceUniform {
    pub buffer: DynamicUniformBuffer<RayTraceUniformRaw>,
}

pub struct RayTracePlugin;

impl Plugin for RayTracePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractResourcePlugin::<RenderTargetImage>::default());
        app.add_plugin(RayTraceGlobalsPlugin);
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<RayTraceUniform>()
            .init_resource::<RayTracePipeline>()
            .add_system_to_stage(RenderStage::Prepare, prepare_uniform)
            .add_system_to_stage(RenderStage::Queue, queue_bind_group);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("raytrace", RayTraceNode::default());
        render_graph
            .add_node_edge("raytrace", bevy::render::main_graph::node::CAMERA_DRIVER)
            .unwrap();
    }
}

struct RenderTargetImageBindGroup(BindGroup);

fn prepare_uniform(
    gpu_camera: Res<RayTraceCamera>,
    mut rt_uniform: ResMut<RayTraceUniform>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
    mut frame: Local<u32>,
) {
    *frame += 1;

    rt_uniform.buffer.clear();

    let transform = gpu_camera.transform;

    rt_uniform.buffer.push(RayTraceUniformRaw {
        frame: *frame,
        render_width: RENDER_TARGET_SIZE.0,
        render_height: RENDER_TARGET_SIZE.1,
        camera_forward: transform.forward().xyzz(),
        camera_up: transform.up().xyzz(),
        camera_right: transform.right().xyzz(),
        camera_position: transform.translation.xyzz(),
    });

    rt_uniform
        .buffer
        .write_buffer(&render_device, &render_queue);
}

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<RayTracePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    rt_uniform: Res<RayTraceUniform>,
    render_target: Res<RenderTargetImage>,
    render_device: Res<RenderDevice>,
) {
    /*
    let view = &gpu_images[&render_target.0];
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view.texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: rt_uniform.buffer.binding().unwrap(),
            },
        ],
    });

    commands.insert_resource(RenderTargetImageBindGroup(bind_group));
    */
}

