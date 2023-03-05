use bevy::{
    prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle, time::FixedTimestep,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 60.0))
                .with_system(rotate_system),
        );
    }
}

#[derive(Component)]
struct Rotate;

fn rotate_system(mut query: Query<&mut Transform, With<Rotate>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_z(f32::to_radians(360.0) * 1.0 / 60.0);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal(16.0),
            scale: 5.0,
            ..Default::default()
        },
        ..Default::default()
    });

    commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                    .with_scale(Vec3::splat(4.0)),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                ..Default::default()
            });
            parent
                .spawn(SpatialBundle::default())
                .with_children(|parent| {
                    parent.spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                        transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0))
                            .with_scale(Vec3::splat(10.0)),
                        material: materials.add(ColorMaterial::from(Color::WHITE)),
                        ..Default::default()
                    });
                })
                .insert(Rotate);
        });
}
