use bevy::{
    prelude::*,
    render::{
        render_graph::{self},
        render_resource::*,
        renderer::RenderContext,
    },
};

use crate::ray_trace_camera::CameraBindGroup;
use crate::ray_trace_globals::{GlobalsBindGroup, RayBufBindGroup};
use crate::ray_trace_intersection::IntersectionBindGroup;
use crate::ray_trace_pipeline::*;
use crate::sphere::ShapesBindGroup;

const WORKGROUP_SIZE: u32 = 8;
use crate::RENDER_TARGET_SIZE;

enum RayTraceState {
    Loading,
    Ready,
}

pub struct RayTraceNode {
    state: RayTraceState,
}

impl Default for RayTraceNode {
    fn default() -> Self {
        Self {
            state: RayTraceState::Loading,
        }
    }
}

impl RayTraceNode {
    fn generate<'a>(&self, world: &'a World, pass: &mut ComputePass<'a>) {
        let camera = &world.resource::<CameraBindGroup>().0;
        let globals = &world.resource::<GlobalsBindGroup>().0;
        let rays = &world.resource::<RayBufBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        pass.set_bind_group(0, camera, &[]);
        pass.set_bind_group(1, globals, &[]);
        pass.set_bind_group(2, rays, &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipelines.generate)
            .unwrap();
        pass.set_pipeline(pipeline);
        pass.dispatch(RENDER_TARGET_SIZE.0 / WORKGROUP_SIZE, 1, 1);
    }

    fn extend<'a>(&self, world: &'a World, pass: &mut ComputePass<'a>) {
        let globals = &world.resource::<GlobalsBindGroup>().0;
        let rays = &world.resource::<RayBufBindGroup>().0;
        let intersections = &world.resource::<IntersectionBindGroup>().0;
        let objects = &world.resource::<ShapesBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        pass.set_bind_group(0, globals, &[]);
        pass.set_bind_group(1, rays, &[]);
        pass.set_bind_group(2, intersections, &[]);
        pass.set_bind_group(3, objects, &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipelines.extend)
            .unwrap();
        pass.set_pipeline(pipeline);
        pass.dispatch(RENDER_TARGET_SIZE.0 / WORKGROUP_SIZE, 1, 1);
    }
}

fn is_pipeline_ready(pipeline_cache: &PipelineCache, pipeline: CachedComputePipelineId) -> bool {
    if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline) {
        true
    } else {
        false
    }
}

impl render_graph::Node for RayTraceNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<RayTracePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            RayTraceState::Loading => {
                if is_pipeline_ready(pipeline_cache, pipeline.pipelines.generate)
                    && is_pipeline_ready(pipeline_cache, pipeline.pipelines.extend)
                {
                    self.state = RayTraceState::Ready;
                }
            }
            RayTraceState::Ready => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        match self.state {
            RayTraceState::Loading => {}

            RayTraceState::Ready => {
                //let mut pass = render_context
                //    .command_encoder
                //    .begin_compute_pass(&ComputePassDescriptor::default());

                //self.generate(world, &mut pass);
                //self.extend(world, &mut pass);
            }
        }

        Ok(())
    }
}
