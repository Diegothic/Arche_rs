use std::f32::consts::PI;

use bevy::{prelude::*, sprite::Anchor};

use super::{
    ai_controls::AIControls, animation::Animation, animation::AnimationMode, arrow::Arrow,
    collision::RectCollider, player_controls::PlayerControls, GameStage, GameStageSpawned,
    GameState, GameTextures, GameTurn, ROT_AXIS_Z,
};

pub struct ArcherPlugin;

impl Plugin for ArcherPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_archer_update_system)
            .add_system(enemy_archer_update_system)
            .add_system(shoot_ai_update_system.after(enemy_archer_update_system))
            .add_system(archers_visibility_update_system)
            .add_system(archers_look_at_target_update_system)
            .add_system(archers_react_to_pull_update_system)
            .add_system(archers_bow_update_system)
            .add_system(archers_look_at_bow_update_system)
            .add_system(archer_shooting_system)
            .add_startup_system(setup_trajectory)
            .add_system(trajectory_system)
            .add_system(trajectory_points_update_system);
    }
}

#[derive(Component, Clone)]
pub struct Archer {
    is_active: bool,
    is_combat: bool,
    pull_angle: f32,
    pull_power: f32,
    shoot_arrow: bool,
    flipped: bool,
}

impl Archer {
    pub fn new(flipped: bool) -> Self {
        Self {
            is_active: false,
            is_combat: false,
            pull_angle: 0.0,
            pull_power: 0.0,
            shoot_arrow: false,
            flipped,
        }
    }
}

#[derive(Component)]
pub struct ArcherPlayer;

#[derive(Component)]
pub struct ArcherEnemy;

#[derive(Component)]
struct ArcherIdle;

#[derive(Component)]
struct ArcherComponent {
    parent: Entity,
}

#[derive(Component)]
struct ReactToPull;

#[derive(Component)]
struct LookAtTarget;

#[derive(Component)]
struct Bow;

#[derive(Component)]
struct LookAtBow;

#[derive(Component)]
struct ShootingPoint;

#[derive(Component)]
struct ShootAI {
    angle: f32,
    power: f32,
    time: f32,
    current_time: f32,
}

#[derive(Component)]
struct ArrowTrajectory {
    is_enabled: bool,
    angle: f32,
    power: f32,
}

#[derive(Component)]
struct ArrowTrajectoryPoint;

#[derive(Component)]
struct ArrowTrajectoryReceiver;

#[derive(Component)]
pub struct DamageReceiver {
    pub hitpoints: i32,
}

pub fn spawn_archer(
    commands: &mut Commands,
    game_textures: &GameTextures,
    parent_archer: Entity,
    receive_trajectory: bool,
) {
    let archer_idle = commands
        .spawn(SpriteSheetBundle {
            texture_atlas: game_textures.archer_blue_idle.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                custom_size: Vec2::new(4.0, 4.0).into(),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        })
        .insert(Animation::new(0, 4, 1.0 / 2.0, AnimationMode::Automatic))
        .insert(ArcherIdle)
        .insert(ArcherComponent {
            parent: parent_archer,
        })
        .id();

    let head_hitbox = commands
        .spawn(TransformBundle {
            local: Transform::from_translation(Vec3::new(0.0, 2.8, 0.0)),
            ..default()
        })
        .insert(ArcherComponent {
            parent: parent_archer,
        })
        .insert(RectCollider::new(
            parent_archer.into(),
            Vec2::ZERO,
            0.7,
            0.7,
        ))
        .insert(DamageReceiver { hitpoints: 4 })
        .id();

    commands.entity(parent_archer).add_child(head_hitbox);

    let body_hitbox = commands
        .spawn(TransformBundle {
            local: Transform::from_translation(Vec3::new(0.0, 1.9, 0.0)),
            ..default()
        })
        .insert(ArcherComponent {
            parent: parent_archer,
        })
        .insert(RectCollider::new(
            parent_archer.into(),
            Vec2::ZERO,
            0.8,
            1.0,
        ))
        .insert(DamageReceiver { hitpoints: 2 })
        .id();

    commands.entity(parent_archer).add_child(body_hitbox);

    let legs_hitbox = commands
        .spawn(TransformBundle {
            local: Transform::from_translation(Vec3::new(0.0, 0.7, 0.0)),
            ..default()
        })
        .insert(ArcherComponent {
            parent: parent_archer,
        })
        .insert(RectCollider::new(
            parent_archer.into(),
            Vec2::ZERO,
            0.8,
            1.2,
        ))
        .insert(DamageReceiver { hitpoints: 2 })
        .id();

    commands.entity(parent_archer).add_child(legs_hitbox);

    let mut shooting_point: Option<Entity> = None;
    let archer_combat = commands
        .spawn(SpriteBundle {
            texture: game_textures.archer_blue_body.clone(),
            sprite: Sprite {
                custom_size: Vec2::new(4.0, 4.0).into(),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.2)),
            visibility: Visibility::INVISIBLE,
            ..default()
        })
        .insert(ArcherComponent {
            parent: parent_archer,
        })
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    texture: game_textures.archer_blue_head.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(4.0, 4.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 2.4, 0.1)),
                    visibility: Visibility::INVISIBLE,
                    ..default()
                })
                .insert(LookAtTarget)
                .insert(ReactToPull)
                .insert(ArcherComponent {
                    parent: parent_archer,
                });

            parent
                .spawn(SpriteBundle {
                    texture: game_textures.archer_blue_arm.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(4.0, 4.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.3, 2.1, -0.1)),
                    visibility: Visibility::INVISIBLE,
                    ..default()
                })
                .insert(LookAtTarget)
                .insert(ArcherComponent {
                    parent: parent_archer,
                })
                .with_children(|parent| {
                    parent
                        .spawn(SpriteSheetBundle {
                            texture_atlas: game_textures.archer_bow.clone(),
                            sprite: TextureAtlasSprite {
                                index: 0,
                                custom_size: Vec2::new(4.0, 4.0).into(),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(1.3, 0.0, 0.2)),
                            visibility: Visibility::INVISIBLE,
                            ..default()
                        })
                        .insert(Animation::new(0, 5, 1.0 / 8.0, AnimationMode::Manual))
                        .insert(ReactToPull)
                        .insert(ArcherComponent {
                            parent: parent_archer,
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(TransformBundle {
                                    local: Transform::from_translation(Vec3::ZERO),
                                    ..default()
                                })
                                .insert(Bow)
                                .insert(ArcherComponent {
                                    parent: parent_archer,
                                });

                            shooting_point = parent
                                .spawn(TransformBundle {
                                    local: Transform::from_translation(Vec3::new(1.25, 0.0, 0.0)),
                                    ..default()
                                })
                                .insert(ShootingPoint)
                                .insert(ArcherComponent {
                                    parent: parent_archer,
                                })
                                .id()
                                .into();
                        });
                });

            parent
                .spawn(SpriteSheetBundle {
                    texture_atlas: game_textures.archer_blue_arm_pull.clone(),
                    sprite: TextureAtlasSprite {
                        index: 0,
                        custom_size: Vec2::new(4.0, 4.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(-0.45, 2.1, 0.2)),
                    ..default()
                })
                .insert(LookAtBow)
                .insert(Animation::new(0, 5, 1.0 / 8.0, AnimationMode::Manual))
                .insert(ReactToPull)
                .insert(ArcherComponent {
                    parent: parent_archer,
                });
        })
        .id();

    if let Some(entity) = shooting_point {
        if receive_trajectory {
            commands.entity(entity).insert(ArrowTrajectoryReceiver);
        }
    }

    commands
        .entity(parent_archer)
        .push_children(&[archer_idle, archer_combat]);
}

fn player_archer_update_system(
    mut game_state: ResMut<GameState>,
    mut player_controls: ResMut<PlayerControls>,
    mut archers: Query<&mut Archer, With<ArcherPlayer>>,
) {
    for mut archer in archers.iter_mut() {
        if game_state.waiting_for_hit || !player_controls.enabled() {
            archer.is_active = false;
            archer.is_combat = false;
            continue;
        }

        archer.is_combat = player_controls.aiming();
        if archer.is_combat {
            let min_angle = f32::to_radians(-80.0);
            let max_angle = f32::to_radians(80.0);
            let controls_angle = player_controls.angle();
            let angle_out_of_bounds = controls_angle > max_angle || controls_angle < min_angle;
            let angle = if angle_out_of_bounds {
                0.0
            } else {
                controls_angle.clamp(min_angle, max_angle)
            };
            archer.pull_angle = angle;
            let pull = if angle_out_of_bounds {
                0.0
            } else {
                player_controls.percent()
            };
            archer.pull_power = pull;

            player_controls.set_indicator_color(if angle_out_of_bounds {
                Color::RED
            } else {
                Color::WHITE
            });
        }

        if player_controls.should_shoot_arrow() && archer.pull_power > 0.0 {
            if game_state.stage == GameStage::Playing {
                game_state.waiting_for_hit = true;
                player_controls.set_enabled(false);
            }

            archer.shoot_arrow = true;
            player_controls.reset_shooting();
        }
    }
}

fn enemy_archer_update_system(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut ai_controls: ResMut<AIControls>,
    mut archers: Query<(Entity, &mut Archer), With<ArcherEnemy>>,
) {
    if !ai_controls.enabled() {
        return;
    }

    for (entity, mut archer) in archers.iter_mut() {
        if game_state.waiting_for_hit
            || game_state.turn != GameTurn::Enemy
            || game_state.stage != GameStage::Playing
        {
            break;
        }

        ai_controls.set_enabled(false);

        archer.is_active = true;
        archer.is_combat = true;

        ai_controls.think(&game_state);
        let angle = ai_controls.get_pull_angle();
        let pull = ai_controls.get_pull_power();

        commands.entity(entity).remove::<ShootAI>();
        commands.entity(entity).insert(ShootAI {
            angle,
            power: pull,
            time: 1.0,
            current_time: 0.0,
        });
    }
}

fn shoot_ai_update_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut archers: Query<(Entity, &mut ShootAI, &mut Archer), With<ArcherEnemy>>,
) {
    for (entity, mut shoot_ai, mut archer) in archers.iter_mut() {
        game_state.waiting_for_hit = true;
        shoot_ai.current_time += time.delta_seconds();

        if shoot_ai.current_time > shoot_ai.time {
            archer.pull_angle = shoot_ai.angle;
            archer.pull_power = shoot_ai.power;
            archer.shoot_arrow = true;
            commands.entity(entity).remove::<ShootAI>();
        } else {
            let angle_delta =
                shoot_ai.angle * (shoot_ai.current_time + shoot_ai.time * 0.3).clamp(0.0, 1.0);
            archer.pull_angle = angle_delta;

            let power_delta =
                shoot_ai.power * (shoot_ai.current_time + shoot_ai.time * 0.3).clamp(0.0, 1.0);
            archer.pull_power = power_delta;
        }
    }
}

fn archers_visibility_update_system(
    archers: Query<&Archer>,
    mut archer_idles: Query<(&ArcherComponent, &mut Visibility), With<ArcherIdle>>,
    mut archer_combats: Query<(&ArcherComponent, &mut Visibility), Without<ArcherIdle>>,
) {
    for (archer_component, mut visibility) in archer_idles.iter_mut() {
        if let Ok(archer) = archers.get(archer_component.parent) {
            visibility.is_visible = !archer.is_combat;
        }
    }

    for (archer_component, mut visibility) in archer_combats.iter_mut() {
        if let Ok(archer) = archers.get(archer_component.parent) {
            visibility.is_visible = archer.is_combat;
        }
    }
}

fn archers_look_at_target_update_system(
    archers: Query<&Archer>,
    mut look_at_targets: Query<(&ArcherComponent, &mut Transform), With<LookAtTarget>>,
) {
    for (archer_component, mut transform) in look_at_targets.iter_mut() {
        if let Ok(archer) = archers.get(archer_component.parent) {
            if !archer.is_combat {
                continue;
            }

            transform.rotation = Quat::from_axis_angle(ROT_AXIS_Z, archer.pull_angle);
        }
    }
}

fn archers_react_to_pull_update_system(
    archers: Query<&Archer>,
    mut pull_reactors: Query<(&ArcherComponent, &mut Animation), With<ReactToPull>>,
) {
    for (archer_component, mut animation) in pull_reactors.iter_mut() {
        if let Ok(archer) = archers.get(archer_component.parent) {
            if !archer.is_combat {
                continue;
            }

            animation.set_progress(archer.pull_power);
        }
    }
}

fn archers_bow_update_system(
    archers: Query<&Archer>,
    mut bows: Query<(&ArcherComponent, &mut Transform), With<Bow>>,
) {
    for (archer_component, mut transform) in bows.iter_mut() {
        if let Ok(archer) = archers.get(archer_component.parent) {
            transform.translation = Vec3::new(-(0.5 + archer.pull_power * 0.9), 0.0, 0.0);
        }
    }
}

fn archers_look_at_bow_update_system(
    archers: Query<&Archer>,
    bows: Query<(&ArcherComponent, &GlobalTransform), With<Bow>>,
    mut look_at_bows: Query<(&ArcherComponent, &GlobalTransform, &mut Transform), With<LookAtBow>>,
) {
    for (archer_component, global_transform, mut transform) in look_at_bows.iter_mut() {
        if let Ok(archer) = archers.get(archer_component.parent) {
            if !archer.is_combat {
                continue;
            }

            for (bow_archer_component, bow_global_transform) in bows.iter() {
                if bow_archer_component.parent == archer_component.parent {
                    let look_target = bow_global_transform.translation();
                    let diff = look_target - global_transform.translation();
                    let mut angle = f32::atan2(diff.y, diff.x);
                    if archer.flipped {
                        angle -= PI;
                        angle *= -1.0;
                    }

                    transform.rotation = Quat::from_axis_angle(ROT_AXIS_Z, angle);
                }
            }
        }
    }
}

fn archer_shooting_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut archers: Query<(Entity, &mut Archer)>,
    shooting_points: Query<(&ArcherComponent, &GlobalTransform), With<ShootingPoint>>,
) {
    for (archer_component, transform) in shooting_points.iter() {
        if let Ok((entity, mut archer)) = archers.get_mut(archer_component.parent) {
            if archer.shoot_arrow {
                archer.shoot_arrow = false;
                let translation = transform.translation();
                let start_pos = Vec2::new(translation.x, translation.y);
                let arrow_velocity = archer.pull_power * 10.0;
                let mut arrow_angle = archer.pull_angle;
                if archer.flipped {
                    arrow_angle -= PI;
                    arrow_angle *= -1.0;
                }

                commands
                    .spawn(SpriteBundle {
                        texture: game_textures.archer_arrow.clone(),
                        sprite: Sprite {
                            custom_size: Vec2::new(4.0, 4.0).into(),
                            ..default()
                        },
                        transform: Transform::from_translation(translation)
                            .with_rotation(Quat::from_rotation_z(arrow_angle)),
                        ..default()
                    })
                    .insert(Arrow::new(entity, start_pos, arrow_velocity, arrow_angle))
                    .insert(GameStageSpawned)
                    .insert(RectCollider::new(entity.into(), Vec2::ZERO, 0.3, 0.3));

                archer.is_active = false;
                archer.is_combat = false;
            }
        }
    }
}

fn setup_trajectory(mut commands: Commands) {
    commands
        .spawn(SpatialBundle::default())
        .insert(ArrowTrajectory {
            is_enabled: false,
            angle: 0.0,
            power: 0.0,
        })
        .with_children(|parent| {
            for _ in 0..=40 {
                parent
                    .spawn(SpriteBundle {
                        transform: Transform::from_scale(Vec3::splat(0.3)),
                        ..Default::default()
                    })
                    .insert(ArrowTrajectoryPoint);
            }
        });
}

fn trajectory_system(
    archers: Query<&Archer>,
    trajectory_receivers: Query<
        (&ArcherComponent, &GlobalTransform),
        With<ArrowTrajectoryReceiver>,
    >,
    mut trajectories: Query<(&mut ArrowTrajectory, &mut Transform, &mut Visibility)>,
) {
    let (mut trajectory, mut transform, mut visibility) = trajectories.single_mut();
    if let Ok((receiver_component, receiver_transform)) = trajectory_receivers.get_single() {
        if let Ok(archer) = archers.get(receiver_component.parent) {
            if !archer.is_combat {
                trajectory.is_enabled = false;
                visibility.is_visible = false;
                trajectory.angle = 0.0;
                trajectory.power = 0.0;
                return;
            }

            if !trajectory.is_enabled {
                trajectory.is_enabled = true;
                visibility.is_visible = true;
            }

            transform.translation = receiver_transform.translation();
            trajectory.angle = archer.pull_angle;
            trajectory.power = archer.pull_power * 10.0;
        }
    } else {
        visibility.is_visible = false;
    }
}

fn trajectory_points_update_system(
    trajectories: Query<&ArrowTrajectory>,
    mut trajectory_points: Query<&mut Transform, With<ArrowTrajectoryPoint>>,
) {
    if let Ok(trajectory) = trajectories.get_single() {
        if !trajectory.is_enabled {
            return;
        }

        let mut t: f32 = 0.0;
        let t_delta = 0.005;
        for mut transform in trajectory_points.iter_mut() {
            let position = Arrow::get_trajectory(trajectory.power, trajectory.angle, t);
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.translation.z = 5.0;
            transform.scale = Vec3::splat(((1.0 - (t / t_delta / 50.0)) / 20.0) + 0.01);
            t += t_delta;
        }
    }
}
