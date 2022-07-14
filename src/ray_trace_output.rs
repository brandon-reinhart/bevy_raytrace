use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_resource::*,
        renderer::RenderDevice,
        RenderApp, RenderStage,
    },
    window::WindowResized,
};

use crate::ray_trace_pipeline::RayTracePipeline;
use crate::RENDER_TARGET_SIZE;

#[derive(Component)]
pub struct RenderTarget;

#[derive(Clone, Deref, ExtractResource)]
pub struct RayTraceOutputImage(Handle<Image>);

pub struct OutputImageBindGroup(pub BindGroup);

pub struct RayTraceOutputPlugin;

impl Plugin for RayTraceOutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractResourcePlugin::<RayTraceOutputImage>::default())
            .add_startup_system(init_output)
            .add_system(on_window_resized);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system_to_stage(RenderStage::Queue, queue);
    }
}

fn vf_to_u8(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}

fn init_output(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Create an image of the size of the screen and attach it to a sprite with the same size.
    // This will become the render target for the compute pipeline.
    // todo: resize the render target and sprite when the screen is resized.

    let fill = vec![0f32, 0f32, 0f32, 1f32];
    let fill = vf_to_u8(&fill[..]);

    let mut image = Image::new_fill(
        Extent3d {
            width: RENDER_TARGET_SIZE.0,
            height: RENDER_TARGET_SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        fill,
        TextureFormat::Rgba32Float,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image = images.add(image);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(
                    RENDER_TARGET_SIZE.0 as f32,
                    RENDER_TARGET_SIZE.1 as f32,
                )),
                ..default()
            },
            texture: image.clone(),
            ..default()
        })
        .insert(RenderTarget);

    commands.insert_resource(RayTraceOutputImage(image));
}

fn on_window_resized(
    mut event: EventReader<WindowResized>,
    mut query: Query<&mut Sprite, With<RenderTarget>>,
) {
    if event.is_empty() || query.is_empty() {
        return;
    }

    // Resize the sprite to fit the window.
    for e in event.iter() {
        let mut sprite = query.single_mut();
        sprite.custom_size = Some(Vec2::new(e.width, e.height));
    }

    // Also need to resize the render target to some aspect ratio of the sprite size...
}

fn queue(
    mut commands: Commands,
    pipeline: Res<RayTracePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    output_image: Res<RayTraceOutputImage>,
    render_device: Res<RenderDevice>,
) {
    let view = &gpu_images[&output_image.0];
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("output_bind_group"),
        layout: &pipeline.bind_groups.output,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });

    commands.insert_resource(OutputImageBindGroup(bind_group));
}

pub fn describe<'a>() -> BindGroupLayoutDescriptor<'a> {
    BindGroupLayoutDescriptor {
        label: Some("output_layout_descriptor"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba32Float,
                view_dimension: TextureViewDimension::D2,
            },
            count: None,
        }],
    }
}
