use crate::sphere::ShapesBindGroup;
use crate::RenderTargetImage;
use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin,
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        RenderApp, RenderStage,
    },
};
use std::borrow::Cow;

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
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app.add_plugin(ExtractResourcePlugin::<RenderTargetImage>::default());
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
}

pub struct RayTracePipeline {
    pub texture_bind_group_layout: BindGroupLayout,
    pub shapes_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for RayTracePipeline {
    fn from_world(world: &mut World) -> Self {
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

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![
                texture_bind_group_layout.clone(),
                shapes_bind_group_layout.clone(),
            ]),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: Some(vec![
                texture_bind_group_layout.clone(),
                shapes_bind_group_layout.clone(),
            ]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        RayTracePipeline {
            texture_bind_group_layout,
            shapes_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

enum RayTraceState {
    Loading,
    Init,
    Update,
}

struct RayTraceNode {
    state: RayTraceState,
}

impl Default for RayTraceNode {
    fn default() -> Self {
        Self {
            state: RayTraceState::Loading,
        }
    }
}

impl render_graph::Node for RayTraceNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<RayTracePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            RayTraceState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = RayTraceState::Init;
                }
            }
            RayTraceState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = RayTraceState::Update;
                }
            }
            RayTraceState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<RenderTargetImageBindGroup>().0;
        let shapes_bind_group = &world.resource::<ShapesBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        let mut pass = render_context
            .command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);
        pass.set_bind_group(1, shapes_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            RayTraceState::Loading => {}

            RayTraceState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch(
                    RENDER_TARGET_SIZE.0 / WORKGROUP_SIZE,
                    RENDER_TARGET_SIZE.1 / WORKGROUP_SIZE,
                    1,
                );
            }

            RayTraceState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch(
                    RENDER_TARGET_SIZE.0 / WORKGROUP_SIZE,
                    RENDER_TARGET_SIZE.1 / WORKGROUP_SIZE,
                    1,
                );
            }
        }

        Ok(())
    }
}
