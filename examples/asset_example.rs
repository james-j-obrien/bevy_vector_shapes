use bevy::prelude::*;
use bevy_vector_shapes::{
    asset::{vector_asset_plugin, VectorShape, VectorShapeAsset},
    ShapePlugin,
};

/// Note: This example requires the 'assets' feature to be enabled
pub fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, ShapePlugin::default(), vector_asset_plugin));

    app.add_systems(Startup, setup);
    app.add_systems(Update, actuate_context);

    app.run();
}

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let camera_tsf = Transform::from_xyz(0.0, 5.0, 0.0).looking_at(Vec3::ZERO, Dir3::X);
    cmds.spawn((Name::new("Camera"), Camera3d::default(), camera_tsf));
    cmds.spawn(DirectionalLight::default());

    let asset = asset_server.load::<VectorShapeAsset>("test.vectorshape.ron");
    cmds.spawn((
        Name::new("Shape"),
        VectorShape::new(asset),
        Transform::default(),
    ));
}

fn actuate_context(mut shapes: Query<&mut VectorShape>, time: Res<Time>) {
    for mut shape in shapes.iter_mut() {
        shape.base_context.vec3s.insert(
            "lineScale".to_owned(),
            Vec3::splat(time.elapsed_secs().sin().abs()),
        );
    }
}
