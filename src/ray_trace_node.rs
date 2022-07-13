use bevy::{
    prelude::*,
    render::{
        render_graph::{self},
        render_resource::*,
        renderer::RenderContext,
    },
};

use crate::ray_trace_globals::{GlobalsBindGroup, RayBufBindGroup};
use crate::ray_trace_pipeline::*;

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

impl render_graph::Node for RayTraceNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<RayTracePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            RayTraceState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.pipelines.generate)
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
                let ray_buffer = &world.resource::<RayBufBindGroup>().0;
                let globals = &world.resource::<GlobalsBindGroup>().0;

                //        let texture_bind_group = &world.resource::<RenderTargetImageBindGroup>().0;
                //      let shapes_bind_group = &world.resource::<ShapesBindGroup>().0;
                let pipeline_cache = world.resource::<PipelineCache>();
                let pipeline = world.resource::<RayTracePipeline>();

                let mut pass = render_context
                    .command_encoder
                    .begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_bind_group(0, ray_buffer, &[]);
                pass.set_bind_group(1, globals, &[]);

                let generate_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.pipelines.generate)
                    .unwrap();
                pass.set_pipeline(generate_pipeline);
                pass.dispatch(RENDER_TARGET_SIZE.0 / WORKGROUP_SIZE, 1, 1);
            }
        }

        Ok(())
    }
}
