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
                    .with_system(bow_system)
                    .with_system(bow_angle_system)
                    .with_system(bow_pull_system)
                    .with_system(trajectory_system)
                    .with_system(trajectory_location_system)
                    .with_system(trajectory_points_system.after(trajectory_system)),
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

fn trajectory_system(mut query: Query<&mut Trajectory>, query_bow: Query<&Bow>) {
    if let Ok(bow) = query_bow.get_single() {
        if let Ok(mut trajectory) = query.get_single_mut() {
            trajectory.angle = bow.angle;
            trajectory.power = bow.pull * 21.0 + 4.0;
        }
    }
}

fn trajectory_location_system(
    mut query: Query<&mut Transform, With<Trajectory>>,
    query_shooting_point: Query<&GlobalTransform, (With<ShootingPoint>, Without<Trajectory>)>,
) {
    if let Ok(shooting_point_transform) = query_shooting_point.get_single() {
        if let Ok(mut transform) = query.get_single_mut() {
            transform.translation = shooting_point_transform.translation();
        }
    }
}

fn trajectory_points_system(
    query_trajectory: Query<&Trajectory>,
    mut query: Query<&mut Transform, With<TrajectoryPoint>>,
) {
    if let Ok(trajectory) = query_trajectory.get_single() {
        let mut t: f32 = 0.0;
        let t_delta = 1.0 / (trajectory.power * 2.0);
        for mut transform in query.iter_mut() {
            let x: f32 = trajectory.power * t * f32::cos(trajectory.angle);
            let mut y: f32 = trajectory.power * t * f32::sin(trajectory.angle);
            y -= 0.5 * 9.0 * t * t;

            transform.translation.x = x;
            transform.translation.y = y;
            transform.translation.z = 5.0;
            transform.scale = Vec3::splat(((1.0 - (t / t_delta / 50.0)) / 10.0) + 0.01);
            t += t_delta;
        }
    }
}

#[derive(Component)]
struct ShootingPoint;

#[derive(Component)]
struct Bow {
    angle: f32,
    pull: f32,
}

impl Default for Bow {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl Bow {
    fn new(angle: f32, pull: f32) -> Self {
        Self { angle, pull }
    }

    fn set_pull(&mut self, value: f32) {
        self.pull = f32::clamp(value, 0.0, 1.0);
    }
}

#[derive(Component)]
struct StringPoint {
    offset: f32,
}

impl Default for StringPoint {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl StringPoint {
    fn new(offset: f32) -> Self {
        Self { offset }
    }
}

fn bow_system(time: Res<Time>, keyboard: Res<Input<KeyCode>>, mut query: Query<&mut Bow>) {
    if let Ok(mut bow) = query.get_single_mut() {
        if keyboard.pressed(KeyCode::Up) {
            bow.angle += 5.0 * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::Down) {
            bow.angle -= 5.0 * time.delta_seconds();
        }

        if keyboard.pressed(KeyCode::Left) {
            let new_pull = bow.pull + 2.0 * time.delta_seconds();
            bow.set_pull(new_pull);
        }
        if keyboard.pressed(KeyCode::Right) {
            let new_pull = bow.pull - 2.0 * time.delta_seconds();
            bow.set_pull(new_pull);
        }
    }
}

fn bow_angle_system(mut query: Query<(&mut Transform, &Bow)>) {
    if let Ok((mut transform, bow)) = query.get_single_mut() {
        transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, bow.angle);
    }
}

fn bow_pull_system(
    mut query: Query<(&Parent, &StringPoint, &mut Transform)>,
    query_parent: Query<&Bow>,
) {
    for (parent, string_point, mut transform) in query.iter_mut() {
        if let Ok(parent_bow) = query_parent.get(parent.get()) {
            let offset = string_point.offset * parent_bow.pull;
            transform.translation.x = -offset;
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
            scaling_mode: ScalingMode::FixedVertical(9.0),
            scale: 2.0,
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
                    .with_scale(Vec3::new(1.0, 1.5, 1.0)),
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                ..Default::default()
            });
            parent
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                            .with_scale(Vec3::splat(0.1)),
                        material: materials.add(ColorMaterial::from(Color::RED)),
                        ..Default::default()
                    });
                })
                .insert(ShootingPoint);
            parent
                .spawn(SpatialBundle::default())
                .with_children(|parent| {
                    parent.spawn(MaterialMesh2dBundle {
                        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                            .with_scale(Vec3::splat(0.1)),
                        material: materials.add(ColorMaterial::from(Color::RED)),
                        ..Default::default()
                    });
                })
                .insert(StringPoint { offset: 0.5 });
        })
        .insert(Bow::default());
}
