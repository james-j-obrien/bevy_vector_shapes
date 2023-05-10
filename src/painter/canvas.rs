use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    ecs::system::EntityCommands,
    prelude::*,
    render::{camera::RenderTarget, texture::ImageSampler, view::RenderLayers},
};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use crate::prelude::*;

pub fn update_canvases(
    mut canvases: Query<(&mut Canvas, &mut Camera, &mut OrthographicProjection), Changed<Canvas>>,
) {
    canvases.for_each_mut(|(canvas, mut camera, mut projection)| {
        camera.target = RenderTarget::Image(canvas.image.clone());
        projection.set_changed();
    })
}

#[derive(Default)]
pub enum CanvasClearMode {
    #[default]
    Always,
    OnDemand,
}

#[derive(Component)]
pub struct Canvas {
    pub image: Handle<Image>,
    pub width: u32,
    pub height: u32,
    pub clear_mode: CanvasClearMode,
    pub zoom: f32,
    will_clear: bool,
}

impl Canvas {
    /// Create a [`Handle<Image>`] according to the given parameters that will function as a render target.
    pub fn create_image(
        assets: &mut Assets<Image>,
        width: u32,
        height: u32,
        sampler: ImageSampler,
    ) -> Handle<Image> {
        let size = Extent3d {
            width,
            height,
            ..default()
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            sampler_descriptor: sampler,
            ..default()
        };

        image.resize(size);
        assets.add(image)
    }

    /// Resize a canvas returning the new [`Handle<Image>`].
    ///
    /// Unfortunately due to a quirk in the bevy renderer you cannot re-use an image handle as a render target once it has been resized.
    pub fn resize(&mut self, assets: &mut Assets<Image>, width: u32, height: u32) -> Handle<Image> {
        self.width = width;
        self.height = height;
        let image = assets
            .get_mut(&self.image)
            .expect("Tried to resize canvas image that does not exist.");
        let size = Extent3d {
            width,
            height,
            ..default()
        };
        let mut new_image = image.clone();
        new_image.resize(size);
        let handle = assets.add(new_image);
        self.image = handle.clone();
        handle
    }
}

#[derive(Default)]
pub struct CanvasConfig {
    pub clear_color: ClearColorConfig,
    pub clear_mode: CanvasClearMode,
    pub width: u32,
    pub height: u32,
    pub order: isize,
    pub sampler: ImageSampler,
}

impl CanvasConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            clear_color: ClearColorConfig::Default,
            clear_mode: CanvasClearMode::Always,
            width,
            height,
            order: -1,
            sampler: ImageSampler::Default,
        }
    }
}

#[derive(Bundle)]
pub struct CanvasBundle {
    camera: Camera2dBundle,
    canvas: Canvas,
    render_layers: RenderLayers,
}

impl CanvasBundle {
    /// Create a [`CanvasBundle`] from a given image with the given configuration.
    pub fn new(image: Handle<Image>, config: CanvasConfig) -> Self {
        Self {
            camera: Camera2dBundle {
                camera_2d: Camera2d {
                    clear_color: config.clear_color,
                },
                camera: Camera {
                    order: config.order,
                    target: RenderTarget::Image(image.clone()),
                    ..default()
                },
                ..default()
            },
            canvas: Canvas {
                image,
                width: config.width,
                height: config.height,

                clear_mode: config.clear_mode,
                will_clear: false,
                zoom: 1.0,
            },
            render_layers: RenderLayers::none(),
        }
    }
}

pub trait CanvasCommands<'w, 's> {
    /// Spawns a [`CanvasBundle`] according to the given [`CanvasConfig`].
    ///
    /// Returns the created [`Handle<Image>`] and [`EntityCommands`].
    fn spawn_canvas(
        &mut self,
        assets: &mut Assets<Image>,
        config: CanvasConfig,
    ) -> (Handle<Image>, EntityCommands<'w, 's, '_>);
}

impl<'w, 's> CanvasCommands<'w, 's> for Commands<'w, 's> {
    fn spawn_canvas(
        &mut self,
        assets: &mut Assets<Image>,
        config: CanvasConfig,
    ) -> (Handle<Image>, EntityCommands<'w, 's, '_>) {
        let handle =
            Canvas::create_image(assets, config.width, config.height, config.sampler.clone());
        (
            handle.clone(),
            self.spawn(CanvasBundle::new(handle, config)),
        )
    }
}
