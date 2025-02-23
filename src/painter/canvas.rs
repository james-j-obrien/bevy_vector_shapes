use bevy::{
    ecs::system::EntityCommands,
    image::ImageSampler,
    prelude::*,
    render::{
        camera::RenderTarget,
        view::{RenderLayers, ViewTarget},
    },
};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

/// Prepares the camera associated with each canvas.
///
/// Replaces the image handle when the canvas is resized and applies [`CanvasMode`] behaviours.
pub fn update_canvases(
    mut canvases: Query<(&mut Canvas, &mut Camera, &mut OrthographicProjection)>,
) {
    canvases
        .iter_mut()
        .for_each(|(mut canvas, mut camera, mut projection)| {
            if let RenderTarget::Image(camera_handle) = &camera.target {
                if camera_handle != &canvas.image {
                    camera.target = RenderTarget::Image(canvas.image.clone());
                    projection.set_changed();
                }
            }

            match canvas.mode {
                CanvasMode::Continuous => {
                    camera.clear_color = canvas.clear_color;
                    camera.is_active = true;
                }
                CanvasMode::Persistent => {
                    if canvas.redraw {
                        camera.clear_color = canvas.clear_color;
                    } else {
                        camera.clear_color = ClearColorConfig::None;
                    }
                }
                CanvasMode::OnDemand => {
                    camera.is_active = canvas.redraw;
                }
            }

            canvas.redraw = false;
        })
}

/// Enum that determines when canvases are cleared and redrawn.
#[derive(Default, Reflect)]
pub enum CanvasMode {
    /// Always clear and draw each frame
    #[default]
    Continuous,
    /// Always draw but don't clear until a call to Canvas::redraw
    Persistent,
    /// Don't draw or clear until a call to Canvas::redraw
    OnDemand,
}

/// Component containing data and methods for a given canvas.
///
/// Can be spawned as part of a [`CanvasBundle`] with [`CanvasCommands::spawn_canvas`].
#[derive(Component, Reflect)]
pub struct Canvas {
    /// Handle to the canvas' target texture.
    pub image: Handle<Image>,
    /// Width of the canvas' target texture in pixels.
    pub width: u32,
    /// Height of the canvas' target texture in pixels.
    pub height: u32,
    /// Determines when the canvas is cleared and drawn to, see [`CanvasMode`].
    pub mode: CanvasMode,
    /// Clear mode to revert to for [`CanvasMode::OnDemand`].
    pub clear_color: ClearColorConfig,
    redraw: bool,
}

impl Canvas {
    /// Create a [`Handle<Image>`] according to the given parameters that will function as a render target.
    pub fn create_image(
        assets: &mut Assets<Image>,
        width: u32,
        height: u32,
        sampler: ImageSampler,
        hdr: bool,
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
                format: if hdr {
                    ViewTarget::TEXTURE_FORMAT_HDR
                } else {
                    TextureFormat::bevy_default()
                },
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            sampler,
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

    /// Mark this canvas to be redraw this frame, behaviour depends on [`CanvasMode`].
    pub fn redraw(&mut self) {
        self.redraw = true;
    }
}

/// Configuration to be used when creating a [`CanvasBundle`]
#[derive(Default)]
pub struct CanvasConfig {
    /// Clear mode analogous to [`Camera2d`].
    pub clear_color: ClearColorConfig,
    /// Determines when the canvas is cleared and drawn to, see [`CanvasMode`].
    pub mode: CanvasMode,
    /// Width of the canvas' target texture in pixels.
    pub width: u32,
    /// Height of the canvas' target texture in pixels.
    pub height: u32,
    /// Camera order analogous to [`Camera`].
    pub order: isize,
    /// [`ImageSampler`] to be used when creating the target texture.
    pub sampler: ImageSampler,
    /// Whether to enable hdr for the associated camera and texture.
    pub hdr: bool,
}

impl CanvasConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            clear_color: ClearColorConfig::Default,
            mode: CanvasMode::default(),
            width,
            height,
            order: -1,
            sampler: ImageSampler::Default,
            hdr: false,
        }
    }
}

/// Bundle containing requisite components for a [`Canvas`] entity.
///
/// Can be spawned with [`CanvasCommands::spawn_canvas`].
#[derive(Bundle)]
pub struct CanvasBundle {
    camera_2d: Camera2d,
    camera: Camera,
    canvas: Canvas,
    render_layers: RenderLayers,
}

impl CanvasBundle {
    /// Create a [`CanvasBundle`] from a given image with the given configuration.
    pub fn new(image: Handle<Image>, config: CanvasConfig) -> Self {
        Self {
            camera_2d: Camera2d,
            camera: Camera {
                order: config.order,
                hdr: config.hdr,
                target: RenderTarget::Image(image.clone()),
                clear_color: config.clear_color,
                ..default()
            },
            canvas: Canvas {
                image,
                width: config.width,
                height: config.height,

                mode: config.mode,
                clear_color: config.clear_color,
                redraw: true,
            },
            render_layers: RenderLayers::none(),
        }
    }
}

/// Extension trait for [`Commands`] to allow spawning of [`CanvasBundle`] entities.
pub trait CanvasCommands<'w> {
    /// Spawns a [`CanvasBundle`] according to the given [`CanvasConfig`].
    ///
    /// Returns the created [`Handle<Image>`] and [`EntityCommands`].
    fn spawn_canvas(
        &mut self,
        assets: &mut Assets<Image>,
        config: CanvasConfig,
    ) -> (Handle<Image>, EntityCommands);
}

impl<'w> CanvasCommands<'w> for Commands<'w, '_> {
    fn spawn_canvas(
        &mut self,
        assets: &mut Assets<Image>,
        config: CanvasConfig,
    ) -> (Handle<Image>, EntityCommands) {
        let handle = Canvas::create_image(
            assets,
            config.width,
            config.height,
            config.sampler.clone(),
            config.hdr,
        );
        (
            handle.clone(),
            self.spawn(CanvasBundle::new(handle, config)),
        )
    }
}
