use std::ops::{Add, Div, Mul, Sub};

use crate::{
    prelude::ShapePainter,
    shapes::{
        Alignment, Cap, DiscPainter, LinePainter, RectPainter, RegularPolygonPainter,
        ShapeAlphaMode, ThicknessType, TrianglePainter,
    },
};
use bevy::{asset::AssetLoader, prelude::*, utils::HashMap};
use serde::Deserialize;
use thiserror::Error;

pub fn vector_asset_plugin(app: &mut App) {
    app.init_asset::<VectorShapeAsset>();
    app.init_asset_loader::<VectorShapeAssetLoader>();
    app.add_systems(Update, paint_vector_shapes);
}

pub fn paint_vector_shapes(
    shapes: Query<(&GlobalTransform, &VectorShape)>,
    mut painter: ShapePainter<'_, '_>,
    shape_assets: Res<Assets<VectorShapeAsset>>,
    asset_server: Res<AssetServer>,
) {
    for (tsf, shape) in shapes.iter() {
        let Some(vector_shape) = shape_assets.get(shape.asset.id()) else {
            debug!("Could not get vector shape asset!");
            continue;
        };

        painter.reset();
        //Need to do this because the ShapePainter has no concept of the parent/child tsf hierarchy
        painter.set_translation(tsf.translation());
        painter.set_rotation(tsf.rotation());
        painter.set_scale(tsf.scale());
        painter.alignment = Alignment::Billboard;

        painter = vector_shape.paint(&shape.context, painter, asset_server.as_ref());
    }
}

#[derive(Component)]
pub struct VectorShape {
    pub asset: Handle<VectorShapeAsset>,
    pub context: ShapeContext,
}

impl VectorShape {
    pub fn new(asset: Handle<VectorShapeAsset>) -> Self {
        VectorShape {
            asset,
            context: ShapeContext::default(),
        }
    }
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct VectorShapeAsset(Vec<ShapePainterOperation>);

impl VectorShapeAsset {
    pub fn paint<'w, 's>(
        &self,
        context: &ShapeContext,
        mut painter: ShapePainter<'w, 's>,
        asset_server: &AssetServer,
    ) -> ShapePainter<'w, 's> {
        for operation in &self.0 {
            painter = operation.execute(context, painter, asset_server);
        }

        painter
    }
}

/// Specified in the ron file, is a tuple of the operand + the key from
#[derive(Deserialize, Debug)]
pub enum ShapeParam<T> {
    Raw(T),
    MulCtx(T, String),
    AddCtx(T, String),
    SubCtx(T, String),
    DivCtx(T, String),
}

impl ShapeParam<Color> {
    fn apply_color(&self, context: &HashMap<String, Color>) -> Color {
        match &self {
            ShapeParam::Raw(raw) => *raw,
            ShapeParam::MulCtx(val, key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return *val;
                };

                Color::from(LinearRgba::from_vec4(
                    val.to_linear().to_vec4() * ctx_val.to_linear().to_vec4(),
                ))
            }
            ShapeParam::AddCtx(val, key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return *val;
                };

                Color::from(LinearRgba::from_vec4(
                    val.to_linear().to_vec4() + ctx_val.to_linear().to_vec4(),
                ))
            }
            ShapeParam::SubCtx(val, key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return *val;
                };

                Color::from(LinearRgba::from_vec4(
                    val.to_linear().to_vec4() - ctx_val.to_linear().to_vec4(),
                ))
            }
            ShapeParam::DivCtx(val, key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return *val;
                };

                Color::from(LinearRgba::from_vec4(
                    val.to_linear().to_vec4() / ctx_val.to_linear().to_vec4(),
                ))
            }
        }
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Copy>
    ShapeParam<T>
{
    fn apply(&self, context: &HashMap<String, T>) -> T {
        match &self {
            ShapeParam::Raw(raw) => *raw,
            ShapeParam::MulCtx(val, key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return *val;
                };

                *val * *ctx_val
            }
            ShapeParam::AddCtx(val, key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return *val;
                };

                *val + *ctx_val
            }
            ShapeParam::SubCtx(val, key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return *val;
                };

                *val - *ctx_val
            }
            ShapeParam::DivCtx(val, key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return *val;
                };

                *val / *ctx_val
            }
        }
    }
}

#[derive(Default)]
pub struct ShapeContext {
    pub floats: HashMap<String, f32>,
    pub vec2s: HashMap<String, Vec2>,
    pub vec3s: HashMap<String, Vec3>,
    pub colors: HashMap<String, Color>,
}

#[derive(Deserialize, Debug)]
pub enum ShapePainterOperation {
    CfgAlignment(Alignment),
    CfgCornerRadii(Vec4),
    CfgAlphaMode(ShapeAlphaMode),
    CfgHollow(bool),
    CfgRoundness(ShapeParam<f32>),
    CfgDisableLaa(bool),
    CfgCap(Cap),
    CfgOrigin(ShapeParam<Vec3>),
    CfgNoOrigin,
    CfgThickness(ShapeParam<f32>),
    CfgThicknessType(ThicknessType),
    SetTranslation(ShapeParam<Vec3>),
    Translate(ShapeParam<Vec3>),

    ///Euler Angle XYZ
    Rotate(ShapeParam<Vec3>),

    ///Euler Angle XYZ
    SetRotation(ShapeParam<Vec3>),

    ///Euler X in Radians
    RotateX(ShapeParam<f32>),

    /// Euler Y in Radians
    RotateY(ShapeParam<f32>),

    /// Euler Z in Radians
    RotateZ(ShapeParam<f32>),

    Scale(ShapeParam<Vec3>),
    SetScale(ShapeParam<Vec3>),

    Set3D,
    Set2D,

    /// Note this converts the color to linearRgba, performs the operation, and then converts it back to a Color
    SetColor(ShapeParam<Color>),

    /// Line(start, end)
    Line(ShapeParam<Vec3>, ShapeParam<Vec3>),

    /// Circle(radius)
    Circle(ShapeParam<f32>),

    /// Arc(radius, start_angle, end_angle)
    Arc(ShapeParam<f32>, ShapeParam<f32>, ShapeParam<f32>),

    Rect(ShapeParam<Vec2>),
    /// Path to image (does assetserver lookup), image dimensions
    Image(String, ShapeParam<Vec2>),
    /// num_sides, radius
    Ngon(ShapeParam<f32>, ShapeParam<f32>),
    /// Triangle with specified corner vertices
    Triangle(ShapeParam<Vec2>, ShapeParam<Vec2>, ShapeParam<Vec2>),
}

impl ShapePainterOperation {
    pub fn execute<'w, 's>(
        &self,
        context: &ShapeContext,
        mut painter: ShapePainter<'w, 's>,
        asset_server: &AssetServer,
    ) -> ShapePainter<'w, 's> {
        match self {
            ShapePainterOperation::SetTranslation(location) => {
                painter.set_translation(location.apply(&context.vec3s))
            }
            ShapePainterOperation::Translate(translation) => {
                painter.translate(translation.apply(&context.vec3s))
            }
            ShapePainterOperation::Rotate(rotation) => {
                let euler = rotation.apply(&context.vec3s);
                painter.rotate(Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z));
            }
            ShapePainterOperation::SetRotation(rotation) => {
                let euler = rotation.apply(&context.vec3s);
                painter.set_rotation(Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z));
            }
            ShapePainterOperation::RotateX(radians) => {
                painter.rotate_x(radians.apply(&context.floats))
            }
            ShapePainterOperation::RotateY(radians) => {
                painter.rotate_y(radians.apply(&context.floats))
            }
            ShapePainterOperation::RotateZ(radians) => {
                painter.rotate_z(radians.apply(&context.floats))
            }
            ShapePainterOperation::Scale(scale) => painter.scale(scale.apply(&context.vec3s)),
            ShapePainterOperation::SetScale(scale) => {
                painter.set_scale(scale.apply(&context.vec3s))
            }
            ShapePainterOperation::Set3D => painter.set_3d(),
            ShapePainterOperation::Set2D => painter.set_2d(),
            ShapePainterOperation::SetColor(color) => {
                painter.set_color(color.apply_color(&context.colors))
            }
            ShapePainterOperation::Line(start, end) => {
                painter.line(start.apply(&context.vec3s), end.apply(&context.vec3s));
            }
            ShapePainterOperation::Circle(radius) => {
                painter.circle(radius.apply(&context.floats));
            }
            ShapePainterOperation::Arc(radius, start_angle, end_angle) => {
                painter.arc(
                    radius.apply(&context.floats),
                    start_angle.apply(&context.floats),
                    end_angle.apply(&context.floats),
                );
            }
            ShapePainterOperation::Rect(rect) => {
                painter.rect(rect.apply(&context.vec2s));
            }
            ShapePainterOperation::Image(path_to_asset, dimensions) => {
                let img = asset_server.get_handle(path_to_asset);
                if let Some(img_handle) = img {
                    painter.image(img_handle, dimensions.apply(&context.vec2s));
                } else {
                    warn!("Could not get image from asset server for path: {path_to_asset}");
                }
            }
            ShapePainterOperation::Ngon(sides, radius) => {
                painter.ngon(sides.apply(&context.floats), radius.apply(&context.floats));
            }
            ShapePainterOperation::Triangle(v_a, v_b, v_c) => {
                painter.triangle(
                    v_a.apply(&context.vec2s),
                    v_b.apply(&context.vec2s),
                    v_c.apply(&context.vec2s),
                );
            }
            ShapePainterOperation::CfgAlignment(alignment) => painter.alignment = *alignment,
            ShapePainterOperation::CfgCornerRadii(vec4) => painter.corner_radii = *vec4,
            ShapePainterOperation::CfgAlphaMode(shape_alpha_mode) => {
                painter.alpha_mode = *shape_alpha_mode
            }
            ShapePainterOperation::CfgHollow(hollow) => painter.hollow = *hollow,
            ShapePainterOperation::CfgRoundness(shape_param) => {
                painter.roundness = shape_param.apply(&context.floats)
            }
            ShapePainterOperation::CfgDisableLaa(disable_laa) => painter.disable_laa = *disable_laa,
            ShapePainterOperation::CfgCap(cap) => painter.cap = *cap,
            ShapePainterOperation::CfgOrigin(shape_param) => {
                painter.origin = Some(shape_param.apply(&context.vec3s))
            }
            ShapePainterOperation::CfgThickness(shape_param) => {
                painter.thickness = shape_param.apply(&context.floats)
            }
            ShapePainterOperation::CfgThicknessType(thickness_type) => {
                painter.thickness_type = *thickness_type
            }
            ShapePainterOperation::CfgNoOrigin => painter.origin = None,
        };

        painter
    }
}

///Custom Asset Loader
#[derive(Default)]
pub struct VectorShapeAssetLoader;

#[derive(Debug, Error)]
pub enum VectorShapeAssetLoaderError {
    #[error("Could not load VectorShapeAsset: {0}")]
    Io(#[from] std::io::Error),

    #[error("Ron Error: {0}")]
    RonError(#[from] ron::error::SpannedError),
}
impl AssetLoader for VectorShapeAssetLoader {
    type Asset = VectorShapeAsset;

    type Settings = ();

    type Error = VectorShapeAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = ron::de::from_bytes::<VectorShapeAsset>(&bytes)?;

        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["vectorshape.ron"]
    }
}
