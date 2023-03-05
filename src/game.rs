use bevy::{
    prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle, time::FixedTimestep,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0 / 60.0))
                    .with_system(rotate_system),
            )
            .add_system_set(
                SystemSet::new()
                    .with_system(trajectory_system)
                    .with_system(trajectory_pints_system.after(trajectory_system)),
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

#[derive(Component)]
struct Trajectory {
    angle: f32,
    power: f32,
}

#[derive(Component)]
struct TrajectoryPoint;

fn trajectory_system(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut Trajectory, &mut Transform)>,
) {
    if let Ok((mut trajectory, mut transform)) = query.get_single_mut() {
        if keyboard.pressed(KeyCode::W) {
            trajectory.angle += 2.0 * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::S) {
            trajectory.angle -= 2.0 * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::D) {
            trajectory.power += 10.0 * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::A) {
            trajectory.power -= 10.0 * time.delta_seconds();
        }

        if keyboard.pressed(KeyCode::Up) {
            transform.translation.y += 10.0 * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::Down) {
            transform.translation.y -= 10.0 * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::Left) {
            transform.translation.x -= 10.0 * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::Right) {
            transform.translation.x += 10.0 * time.delta_seconds();
        }
    }
}

fn trajectory_pints_system(
    query_trajectory: Query<&Trajectory>,
    mut query: Query<&mut Transform, With<TrajectoryPoint>>,
) {
    if let Ok(trajectory) = query_trajectory.get_single() {
        let mut t: f32 = 0.0;
        for mut transform in query.iter_mut() {
            let x: f32 = trajectory.power * t * f32::cos(trajectory.angle);
            let mut y: f32 = trajectory.power * t * f32::sin(trajectory.angle);
            y -= 0.5 * 9.0 * t * t;

            transform.translation.x = x;
            transform.translation.y = y;
            transform.translation.z = 5.0;
            transform.scale = Vec3::splat(5.0 - t) / 25.0;
            t += 0.1;
        }
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
        .insert(Trajectory {
            angle: 0.0,
            power: 1.0,
        })
        .with_children(|parent| {
            for _ in 0..=50 {
                parent
                    .spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                            .with_scale(Vec3::splat(0.3)),
                        material: materials.add(ColorMaterial::from(Color::WHITE)),
                        ..Default::default()
                    })
                    .insert(TrajectoryPoint);
            }
        });

    commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                    .with_scale(Vec3::splat(1.0)),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                ..Default::default()
            });
            parent
                .spawn(SpatialBundle::default())
                .with_children(|parent| {
                    parent.spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                        transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0))
                            .with_scale(Vec3::splat(0.5)),
                        material: materials.add(ColorMaterial::from(Color::WHITE)),
                        ..Default::default()
                    });
                })
                .insert(Rotate);
        });
}
