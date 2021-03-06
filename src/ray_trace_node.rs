use crate::plugin::{
    CameraGlobalsBindGroup, ObjectsMaterialsBindGroup, RaysIntersectionsBindGroup,
};
use crate::ray_trace_output::OutputImageBindGroup;
use crate::ray_trace_pipeline::*;
use crate::{RENDER_TARGET_SIZE, SAMPLES_PER_RAY};
use bevy::{
    prelude::*,
    render::{
        render_graph::{self},
        render_resource::*,
        renderer::RenderContext,
    },
};

const WORKGROUP_SIZE: u32 = 128;

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
    fn clear<'a>(&self, world: &'a World, pass: &mut ComputePass<'a>) {
        let num_dispatch =
            (RENDER_TARGET_SIZE.0 * RENDER_TARGET_SIZE.1 * SAMPLES_PER_RAY as u32) / WORKGROUP_SIZE;

        let camera_globals = &world.resource::<CameraGlobalsBindGroup>().0;
        let rays_intersections = &world.resource::<RaysIntersectionsBindGroup>().0;
        let output = &world.resource::<OutputImageBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        pass.set_bind_group(0, camera_globals, &[]);
        pass.set_bind_group(1, rays_intersections, &[]);
        pass.set_bind_group(2, output, &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipelines.clear)
            .unwrap();
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_dispatch, 1, 1);
    }

    fn prepass<'a>(&self, world: &'a World, pass: &mut ComputePass<'a>) {
        let camera_globals = &world.resource::<CameraGlobalsBindGroup>().0;
        let rays_intersections = &world.resource::<RaysIntersectionsBindGroup>().0;
        let output = &world.resource::<OutputImageBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        pass.set_bind_group(0, camera_globals, &[]);
        pass.set_bind_group(1, rays_intersections, &[]);
        pass.set_bind_group(2, output, &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipelines.prepass)
            .unwrap();
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(1, 1, 1);
    }

    fn generate<'a>(&self, world: &'a World, pass: &mut ComputePass<'a>) {
        let num_dispatch =
            (RENDER_TARGET_SIZE.0 * RENDER_TARGET_SIZE.1 * SAMPLES_PER_RAY as u32) / WORKGROUP_SIZE;

        let camera_globals = &world.resource::<CameraGlobalsBindGroup>().0;
        let rays_intersections = &world.resource::<RaysIntersectionsBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        pass.set_bind_group(0, camera_globals, &[]);
        pass.set_bind_group(1, rays_intersections, &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipelines.generate)
            .unwrap();
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_dispatch, 1, 1);
    }

    fn intersect<'a>(&self, world: &'a World, pass: &mut ComputePass<'a>) {
        let num_dispatch =
            (RENDER_TARGET_SIZE.0 * RENDER_TARGET_SIZE.1 * SAMPLES_PER_RAY as u32) / WORKGROUP_SIZE;

        let camera_globals = &world.resource::<CameraGlobalsBindGroup>().0;
        let rays_intersections = &world.resource::<RaysIntersectionsBindGroup>().0;
        let objects_materials = &world.resource::<ObjectsMaterialsBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        pass.set_bind_group(0, camera_globals, &[]);
        pass.set_bind_group(1, rays_intersections, &[]);
        pass.set_bind_group(2, objects_materials, &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipelines.intersect)
            .unwrap();
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_dispatch, 1, 1);
    }

    fn shade<'a>(&self, world: &'a World, pass: &mut ComputePass<'a>) {
        let num_dispatch =
            (RENDER_TARGET_SIZE.0 * RENDER_TARGET_SIZE.1 * SAMPLES_PER_RAY as u32) / WORKGROUP_SIZE;

        let camera_globals = &world.resource::<CameraGlobalsBindGroup>().0;
        let rays_intersections = &world.resource::<RaysIntersectionsBindGroup>().0;
        let objects_materials = &world.resource::<ObjectsMaterialsBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        pass.set_bind_group(0, camera_globals, &[]);
        pass.set_bind_group(1, rays_intersections, &[]);
        pass.set_bind_group(2, objects_materials, &[]);
        // also shadow rays

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipelines.shade)
            .unwrap();
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_dispatch, 1, 1);
    }

    fn collect<'a>(&self, world: &'a World, pass: &mut ComputePass<'a>) {
        let num_dispatch =
            (RENDER_TARGET_SIZE.0 * RENDER_TARGET_SIZE.1 * SAMPLES_PER_RAY as u32) / WORKGROUP_SIZE;

        let camera_globals = &world.resource::<CameraGlobalsBindGroup>().0;
        let rays_intersections = &world.resource::<RaysIntersectionsBindGroup>().0;
        let output = &world.resource::<OutputImageBindGroup>().0;

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<RayTracePipeline>();

        pass.set_bind_group(0, camera_globals, &[]);
        pass.set_bind_group(1, rays_intersections, &[]);
        pass.set_bind_group(2, output, &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipelines.collect)
            .unwrap();
        pass.set_pipeline(pipeline);
        pass.dispatch_workgroups(num_dispatch, 1, 1);
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
                if is_pipeline_ready(pipeline_cache, pipeline.pipelines.clear)
                    && is_pipeline_ready(pipeline_cache, pipeline.pipelines.prepass)
                    && is_pipeline_ready(pipeline_cache, pipeline.pipelines.generate)
                    && is_pipeline_ready(pipeline_cache, pipeline.pipelines.intersect)
                    && is_pipeline_ready(pipeline_cache, pipeline.pipelines.shade)
                    && is_pipeline_ready(pipeline_cache, pipeline.pipelines.collect)
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
                let mut pass = render_context
                    .command_encoder
                    .begin_compute_pass(&ComputePassDescriptor::default());

                self.clear(world, &mut pass);

                self.generate(world, &mut pass);

                for _ in 0..3 {
                    self.prepass(world, &mut pass);
                    self.intersect(world, &mut pass);
                    self.shade(world, &mut pass);
                }

                self.collect(world, &mut pass);
            }
        }

        Ok(())
    }
}
