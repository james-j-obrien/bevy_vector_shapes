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
    mut shapes: Query<(&GlobalTransform, &mut VectorShape)>,
    mut painter: ShapePainter<'_, '_>,
    shape_assets: Res<Assets<VectorShapeAsset>>,
    asset_server: Res<AssetServer>,
) {
    for (tsf, mut shape) in shapes.iter_mut() {
        let Some(vector_shape) = shape_assets.get(shape.asset.id()) else {
            debug!("Could not get vector shape asset!");
            continue;
        };

        shape.working_context = shape.base_context.clone();
        painter.reset();
        //Need to do this because the ShapePainter has no concept of the parent/child tsf hierarchy
        painter.set_translation(tsf.translation());
        painter.set_rotation(tsf.rotation());
        painter.set_scale(tsf.scale());
        painter.alignment = Alignment::Billboard;

        painter = vector_shape.paint(&mut shape.working_context, painter, asset_server.as_ref());
    }
}

#[derive(Component)]
pub struct VectorShape {
    pub asset: Handle<VectorShapeAsset>,

    /// The base_context that the asset gets passed each frame
    pub base_context: ShapeContext,

    /// The working context that the asset can modify, gets reset to base_context each frame
    pub working_context: ShapeContext,
}

impl VectorShape {
    pub fn new(asset: Handle<VectorShapeAsset>) -> Self {
        VectorShape {
            asset,
            base_context: ShapeContext::default(),
            working_context: ShapeContext::default(),
        }
    }
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct VectorShapeAsset(Vec<ShapePainterOperation>);

impl VectorShapeAsset {
    pub fn paint<'w, 's>(
        &self,
        context: &mut ShapeContext,
        mut painter: ShapePainter<'w, 's>,
        asset_server: &AssetServer,
    ) -> ShapePainter<'w, 's> {
        let operations = &self.0;
        let mut instr_idx = 0;

        loop {
            if instr_idx >= operations.len() {
                break;
            }

            let operation = &operations[instr_idx];

            let isr = operation.execute(context, &mut painter, asset_server);
            debug!("idx: {}", instr_idx);
            match isr.eval(instr_idx, operations) {
                Ok(new_line) => {
                    debug!("New Line: {}", new_line);
                    instr_idx = new_line
                }
                Err(e) => {
                    debug!("{e}");
                    break;
                }
            };
        }

        painter
    }
}

#[derive(Deserialize, Debug)]
pub enum Conditional<T> {
    Eq(T, String),
    NEq(T, String),
    Gt(T, String),
    Lt(T, String),
    And(Box<(Conditional<T>, Conditional<T>)>),
    Or(Box<(Conditional<T>, Conditional<T>)>),
}

#[derive(Deserialize, Debug)]
pub enum CompareFn {
    Eq,
    NEq,
    GtEq,
    Gt,
    Lt,
    LtEq,
}

#[derive(Deserialize, Debug)]
pub enum Dimension {
    X,
    Y,
    Z,
}

impl Conditional<()> {
    fn vec2(
        cond: &CompareFn,
        dim: &Dimension,
        value: &Vec2,
        ctx_key: &String,
        ctx: &HashMap<String, Vec2>,
    ) -> bool {
        let Some(ctx_value) = ctx.get(ctx_key) else {
            warn!("Could not find context value for key: {}", ctx_key);
            return false;
        };

        let (val, ctx_val) = match dim {
            Dimension::X => (value.x, ctx_value.x),
            Dimension::Y => (value.y, ctx_value.y),
            _ => {
                warn!("No Z dimension on vec2!");
                return false;
            }
        };

        match cond {
            CompareFn::Eq => val == ctx_val,
            CompareFn::NEq => val != ctx_val,
            CompareFn::GtEq => val >= ctx_val,
            CompareFn::Gt => val <= ctx_val,
            CompareFn::Lt => val < ctx_val,
            CompareFn::LtEq => val <= ctx_val,
        }
    }

    fn vec3(
        cond: &CompareFn,
        dim: &Dimension,
        value: &Vec3,
        ctx_key: &String,
        ctx: &HashMap<String, Vec3>,
    ) -> bool {
        let Some(ctx_value) = ctx.get(ctx_key) else {
            warn!("Could not find context value for key: {}", ctx_key);
            return false;
        };

        let (val, ctx_val) = match dim {
            Dimension::X => (value.x, ctx_value.x),
            Dimension::Y => (value.y, ctx_value.y),
            Dimension::Z => (value.z, ctx_value.z),
        };

        match cond {
            CompareFn::Eq => val == ctx_val,
            CompareFn::NEq => val != ctx_val,
            CompareFn::GtEq => val >= ctx_val,
            CompareFn::Gt => val <= ctx_val,
            CompareFn::Lt => val < ctx_val,
            CompareFn::LtEq => val <= ctx_val,
        }
    }

    fn bool_eq_ctx(val: bool, key: &String, ctx: &HashMap<String, bool>) -> bool {
        let Some(res) = ctx.get(key) else {
            warn!("Could not get bool from context: {key}");
            return false;
        };

        *res == val
    }

    fn bool_neq_ctx(val: bool, key: &String, ctx: &HashMap<String, bool>) -> bool {
        let Some(res) = ctx.get(key) else {
            warn!("Could not get bool from context: {key}");
            return false;
        };

        *res != val
    }
}

impl<T: PartialOrd> Conditional<T> {
    fn eval(&self, ctx: &HashMap<String, T>) -> bool {
        match self {
            Conditional::Eq(val, key) => {
                let Some(ctx_val) = ctx.get(key) else {
                    warn!("Could not find context for: {key}");
                    return false;
                };

                val == ctx_val
            }
            Conditional::NEq(val, key) => {
                let Some(ctx_val) = ctx.get(key) else {
                    warn!("Could not find context for: {key}");
                    return false;
                };

                val != ctx_val
            }
            Conditional::Gt(val, key) => {
                let Some(ctx_val) = ctx.get(key) else {
                    warn!("Could not find context for: {key}");
                    return false;
                };

                val > ctx_val
            }
            Conditional::Lt(val, key) => {
                let Some(ctx_val) = ctx.get(key) else {
                    warn!("Could not find context for: {key}");
                    return false;
                };

                val < ctx_val
            }
            Conditional::And(operands) => operands.0.eval(ctx) && operands.1.eval(ctx),
            Conditional::Or(operands) => operands.0.eval(ctx) || operands.1.eval(ctx),
        }
    }
}

/// Specified in the ron file, is a tuple of the operand + the key from
#[derive(Deserialize, Debug)]
pub enum ShapeParam<T> {
    Raw(T),
    Ctx(String),
    MulCtx(T, String),
    AddCtx(T, String),
    SubCtx(T, String),
    DivCtx(T, String),
}

impl ShapeParam<Color> {
    fn apply_color(&self, context: &HashMap<String, Color>) -> Color {
        match &self {
            ShapeParam::Raw(raw) => *raw,
            ShapeParam::Ctx(key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return Color::default();
                };

                *ctx_val
            }
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

impl<T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Copy + Default>
    ShapeParam<T>
{
    fn apply(&self, context: &HashMap<String, T>) -> T {
        match &self {
            ShapeParam::Raw(raw) => *raw,
            ShapeParam::Ctx(key) => {
                let Some(ctx_val) = context.get(key) else {
                    warn!("Could not find context for: {key}");
                    return T::default();
                };

                *ctx_val
            }
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

#[derive(Default, Clone)]
pub struct ShapeContext {
    pub floats: HashMap<String, f32>,
    pub vec2s: HashMap<String, Vec2>,
    pub vec3s: HashMap<String, Vec3>,
    pub colors: HashMap<String, Color>,
    pub bools: HashMap<String, bool>,
}

#[derive(Deserialize, Debug)]
pub enum ShapePainterOperation {
    Alignment(Alignment),
    CornerRadii(Vec4),
    AlphaMode(ShapeAlphaMode),
    Hollow(bool),
    Roundness(ShapeParam<f32>),
    LaaDisabled(bool),
    Cap(Cap),
    Origin(ShapeParam<Vec3>),
    NoOrigin,
    Thickness(ShapeParam<f32>),
    ThicknessType(ThicknessType),
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

    // Now we're getting a little carried away
    /// move the instruction pointer relative to this one
    Goto(IsrTgt),
    CondF32(Conditional<f32>, Box<ShapePainterOperation>),
    CondVec2(
        Dimension,
        CompareFn,
        Vec2,
        String,
        Box<ShapePainterOperation>,
    ),
    CondVec3(
        Dimension,
        CompareFn,
        Vec3,
        String,
        Box<ShapePainterOperation>,
    ),
    Eq(bool, String, Box<ShapePainterOperation>),
    Neq(bool, String, Box<ShapePainterOperation>),
    Label(String),
    SetF32(ShapeParam<f32>, String),
    SetVec2(ShapeParam<Vec2>, String),
    SetVec3(ShapeParam<Vec3>, String),
    SetBool(bool, String),
}

#[derive(Deserialize, Debug, Clone)]
pub enum IsrTgt {
    Relative(isize),
    Abs(usize),
    Label(String),
}

impl IsrTgt {
    fn eval(
        &self,
        current_line: usize,
        operations: &[ShapePainterOperation],
    ) -> Result<usize, String> {
        let new_size = match self {
            IsrTgt::Relative(rel) => current_line as isize + rel,
            IsrTgt::Abs(line) => *line as isize,
            IsrTgt::Label(label) => {
                let Some(label_line) = operations.iter().position(|operation| match operation {
                    ShapePainterOperation::Label(op_label) => op_label == label,
                    _ => false,
                }) else {
                    return Err(format!("Could not find label: {}", label));
                };

                label_line as isize
            }
        };

        if new_size < 0 {
            return Err("New ISR lower than zero".to_owned());
        } else if new_size >= operations.len() as isize {
            return Err("New ISR Exceeds bounds of operations length".to_owned());
        }

        Ok(new_size as usize)
    }
}

impl ShapePainterOperation {
    /// Returns a tuple of the modified shape painter and an isr modification (will almost always be 1 except in the event of a GOTO)
    pub fn execute(
        &self,
        context: &mut ShapeContext,
        painter: &mut ShapePainter,
        asset_server: &AssetServer,
    ) -> IsrTgt {
        let mut isr = IsrTgt::Relative(1);
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
            ShapePainterOperation::Alignment(alignment) => painter.alignment = *alignment,
            ShapePainterOperation::CornerRadii(vec4) => painter.corner_radii = *vec4,
            ShapePainterOperation::AlphaMode(shape_alpha_mode) => {
                painter.alpha_mode = *shape_alpha_mode
            }
            ShapePainterOperation::Hollow(hollow) => painter.hollow = *hollow,
            ShapePainterOperation::Roundness(shape_param) => {
                painter.roundness = shape_param.apply(&context.floats)
            }
            ShapePainterOperation::LaaDisabled(disable_laa) => painter.disable_laa = *disable_laa,
            ShapePainterOperation::Cap(cap) => painter.cap = *cap,
            ShapePainterOperation::Origin(shape_param) => {
                painter.origin = Some(shape_param.apply(&context.vec3s))
            }
            ShapePainterOperation::Thickness(shape_param) => {
                painter.thickness = shape_param.apply(&context.floats)
            }
            ShapePainterOperation::ThicknessType(thickness_type) => {
                painter.thickness_type = *thickness_type
            }
            ShapePainterOperation::NoOrigin => painter.origin = None,
            ShapePainterOperation::Goto(isr_tgt) => {
                isr = isr_tgt.clone();
            }
            ShapePainterOperation::CondF32(conditional, shape_painter_operation) => {
                if conditional.eval(&context.floats) {
                    return shape_painter_operation.execute(context, painter, asset_server);
                };
            }
            ShapePainterOperation::CondVec2(dim, comp, vec, ctx_key, op) => {
                if Conditional::vec2(comp, dim, vec, ctx_key, &context.vec2s) {
                    return op.execute(context, painter, asset_server);
                }
            }
            ShapePainterOperation::CondVec3(dim, comp, vec, ctx_key, op) => {
                if Conditional::vec3(comp, dim, vec, ctx_key, &context.vec3s) {
                    return op.execute(context, painter, asset_server);
                }
            }
            ShapePainterOperation::Eq(val, key, op) => {
                if Conditional::bool_eq_ctx(*val, key, &context.bools) {
                    return op.execute(context, painter, asset_server);
                }
            }
            ShapePainterOperation::Neq(val, key, op) => {
                if Conditional::bool_neq_ctx(*val, key, &context.bools) {
                    return op.execute(context, painter, asset_server);
                }
            }
            ShapePainterOperation::Label(_) => {}
            ShapePainterOperation::SetF32(val, key) => {
                let computed = val.apply(&context.floats);
                context.floats.insert(key.clone(), computed);
            }
            ShapePainterOperation::SetVec2(vec2, key) => {
                let computed = vec2.apply(&context.vec2s);
                context.vec2s.insert(key.clone(), computed);
            }
            ShapePainterOperation::SetVec3(vec3, key) => {
                let computed = vec3.apply(&context.vec3s);
                context.vec3s.insert(key.clone(), computed);
            }
            ShapePainterOperation::SetBool(val, key) => {
                context.bools.insert(key.clone(), *val);
            }
        };

        isr
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
