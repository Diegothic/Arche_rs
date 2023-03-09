use bevy::{prelude::*, render::camera::ScalingMode};

use self::animation::AnimationPlugin;
use self::archer::{Archer, ArcherElement, ArcherPlugin, Bow, TrajectoryReceiver};

mod animation;
mod archer;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AnimationPlugin)
            .add_plugin(ArcherPlugin)
            .add_startup_system_set_to_stage(
                StartupStage::PreStartup,
                SystemSet::new()
                    .with_system(setup_camera)
                    .with_system(setup_resources),
            )
            .add_startup_system(setup_trajectory)
            .add_system(trajectory_system)
            .add_system(trajectory_points_system);
    }
}

#[derive(Resource, Default)]
struct GameTextures {
    archer_blue_idle: Handle<TextureAtlas>,
    // archer_blue_shooting_base: Handle<Image>,
    // archer_blue_shooting_head: Handle<Image>,
    // archer_blue_shooting_bow_arm: Handle<TextureAtlas>,
    // archer_blue_shooting_pull_arm: Handle<TextureAtlas>,
    // bow: Handle<TextureAtlas>,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(12.0),
            scale: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn setup_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture = asset_server.load("textures/archer_blue_idle.png");
    let atlas = TextureAtlas::from_grid(texture, Vec2::new(64.0, 64.0), 2, 2, None, None);

    let atlas_handle = texture_atlases.add(atlas);

    let game_textures = GameTextures {
        archer_blue_idle: atlas_handle,
        // archer_blue_shooting_base: ,
        // archer_blue_shooting_head: ,
        // archer_blue_shooting_bow_arm: ,
        // archer_blue_shooting_pull_arm: ,
        // bow: ,
    };

    commands.insert_resource(game_textures);
}

#[derive(Component)]
struct Trajectory {
    angle: f32,
    power: f32,
}

#[derive(Component)]
struct TrajectoryPoint;

fn trajectory_system(
    mut query: Query<(&mut Transform, &mut Trajectory)>,
    query_archer: Query<&Archer, With<TrajectoryReceiver>>,
    query_bow: Query<(&GlobalTransform, &ArcherElement), With<Bow>>,
) {
    if let Ok((mut transform, mut trajectory)) = query.get_single_mut() {
        if let Ok(archer) = query_archer.get_single() {
            let mut found_transform: Option<&GlobalTransform> = Option::None;
            for (transform, element) in query_bow.iter() {
                if element.archer_id == archer.id {
                    found_transform = Some(transform);
                }
            }

            if let Some(bow_transform) = found_transform {
                transform.translation = bow_transform.translation();
                trajectory.angle = archer.aim_angle;
                trajectory.power = archer.bow_pull * 28.0 + 2.0;
            }
        }
    }
}

fn trajectory_points_system(
    query_trajectory: Query<&Trajectory>,
    mut query: Query<&mut Transform, With<TrajectoryPoint>>,
) {
    if let Ok(trajectory) = query_trajectory.get_single() {
        let mut t: f32 = 0.0;
        let t_delta = 1.0 / (trajectory.power * 10.0);
        for mut transform in query.iter_mut() {
            let x: f32 = trajectory.power * t * f32::cos(trajectory.angle);
            let mut y: f32 = trajectory.power * t * f32::sin(trajectory.angle);
            y -= 0.5 * 9.0 * t * t;

            transform.translation.x = x;
            transform.translation.y = y;
            transform.translation.z = 5.0;
            transform.scale = Vec3::splat(((1.0 - (t / t_delta / 50.0)) / 20.0) + 0.01);
            t += t_delta;
        }
    }
}

fn setup_trajectory(mut commands: Commands) {
    commands
        .spawn(SpatialBundle::default())
        .insert(Trajectory {
            angle: 0.0,
            power: 1.0,
        })
        .with_children(|parent| {
            for _ in 0..=50 {
                parent
                    .spawn(SpriteBundle {
                        transform: Transform::from_scale(Vec3::splat(0.3)),
                        ..Default::default()
                    })
                    .insert(TrajectoryPoint);
            }
        });
}
