use std::f32::consts::PI;

use bevy::{prelude::*, sprite::Anchor};

use super::{animation::Animation, animation::AnimationMode, GameTextures};

pub struct ArcherPlugin;

impl Plugin for ArcherPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_archer_system)
            .add_system(target_rotation_system)
            .add_system(archer_controll)
            .add_system(archer_combat_limb_system);
    }
}

#[derive(Component)]
pub struct Archer {
    pub id: u32,
    pub aim_angle: f32,
    pub bow_pull: f32,
}

#[derive(Component)]
pub struct ArcherElement {
    pub archer_id: u32,
}

#[derive(Component)]
pub struct CombatLimb;

#[derive(Component)]
pub struct Bow;

pub enum TargetRotationType {
    Instant,
    Smooth(f32),
}

#[derive(Component)]
pub struct TargetRotation {
    pub is_enabled: bool,
    pub angle: f32,
    pub rotation_type: TargetRotationType,
}

#[derive(Component)]
pub struct TrajectoryReceiver;

fn spawn_archer_system(mut commands: Commands, game_textures: Res<GameTextures>) {
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: game_textures.archer_blue_idle.clone(),
            sprite: TextureAtlasSprite {
                index: 0,
                custom_size: Vec2::new(4.0, 4.0).into(),
                ..default()
            },
            transform: Transform::from_translation(Vec3::splat(0.0)),
            ..default()
        })
        .insert(Animation::new(0, 4, 1.0 / 2.0, AnimationMode::Automatic));

    commands
        .spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with_children(|parent| {
            // Right Arm
            parent
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.8, 0.8, 0.8),
                        custom_size: Vec2::new(0.7, 0.2).into(),
                        anchor: Anchor::CenterLeft,
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.25, 1.5, 1.0)),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(0.2, 0.2, 1.0),
                                custom_size: Vec2::new(0.25, 1.0).into(),
                                anchor: Anchor::CenterRight,
                                ..Default::default()
                            },
                            transform: Transform::from_translation(Vec3::new(0.7, 0.0, 10.0)),
                            ..Default::default()
                        })
                        .insert((ArcherElement { archer_id: 0 }, Bow));
                })
                .insert((
                    ArcherElement { archer_id: 0 },
                    CombatLimb,
                    TargetRotation {
                        is_enabled: true,
                        angle: 0.0,
                        rotation_type: TargetRotationType::Smooth(4.0),
                    },
                ));

            // Left Arm
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.8, 0.8),
                    custom_size: Vec2::new(0.7, 0.2).into(),
                    anchor: Anchor::CenterRight,
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(-0.25, 1.5, 2.0)),
                ..Default::default()
            });

            // Head
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.9, 0.9, 0.9),
                    custom_size: Vec2::new(0.3, 0.3).into(),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 1.65, 0.0)),
                ..Default::default()
            });

            // Torso
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.7, 0.7, 0.7),
                    custom_size: Vec2::new(0.5, 0.7).into(),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 1.15, 0.0)),
                ..Default::default()
            });

            // Legs
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.6, 0.6, 0.6),
                    custom_size: Vec2::new(0.5, 0.8).into(),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 0.4, 0.0)),
                ..Default::default()
            });
        })
        .insert((
            Archer {
                id: 0,
                aim_angle: 0.0,
                bow_pull: 0.0,
            },
            TrajectoryReceiver,
        ));
}

fn archer_controll(time: Res<Time>, keyboard: Res<Input<KeyCode>>, mut query: Query<&mut Archer>) {
    for mut archer in query.iter_mut() {
        if keyboard.pressed(KeyCode::Up) {
            archer.aim_angle += 5.0 * time.delta_seconds();
        }

        if keyboard.pressed(KeyCode::Down) {
            archer.aim_angle -= 5.0 * time.delta_seconds();
        }

        if archer.aim_angle <= -PI {
            archer.aim_angle += 2.0 * PI;
        }

        if archer.aim_angle >= PI {
            archer.aim_angle -= 2.0 * PI;
        }

        if keyboard.pressed(KeyCode::Left) {
            archer.bow_pull += 2.0 * time.delta_seconds();
        }

        if keyboard.pressed(KeyCode::Right) {
            archer.bow_pull -= 2.0 * time.delta_seconds();
        }

        archer.bow_pull = f32::clamp(archer.bow_pull, 0.0, 1.0);
    }
}

fn archer_combat_limb_system(
    mut query: Query<(&mut TargetRotation, &ArcherElement), With<CombatLimb>>,
    query_archer: Query<&Archer>,
) {
    for (mut target_rotation, element) in query.iter_mut() {
        let mut found_archer: Option<&Archer> = Option::None;
        for archer in query_archer.iter() {
            if archer.id == element.archer_id {
                found_archer = Option::Some(archer);
                break;
            }
        }
        if let Some(archer) = found_archer {
            target_rotation.angle = archer.aim_angle;
        }
    }
}

fn target_rotation_system(time: Res<Time>, mut query: Query<(&mut Transform, &TargetRotation)>) {
    for (mut transform, target_rotation) in query.iter_mut() {
        if !target_rotation.is_enabled {
            continue;
        }

        let rot_z = transform.rotation.to_euler(EulerRot::XYZ).2;
        let desired_rot_z = target_rotation.angle;
        let mut new_rot = rot_z;
        match target_rotation.rotation_type {
            TargetRotationType::Instant => {
                new_rot = desired_rot_z;
            }
            TargetRotationType::Smooth(speed) => {
                let rot_delta = speed * time.delta_seconds();
                let rot_diff = f32::abs(desired_rot_z - rot_z);
                let rot_diff_comp = (2.0 * PI) - rot_diff;
                if rot_diff < 0.01 {
                    new_rot = desired_rot_z;
                } else {
                    let mut direction = if rot_z < desired_rot_z { 1.0 } else { -1.0 };
                    if rot_diff_comp < rot_diff {
                        direction *= -1.0;
                    }

                    new_rot += rot_delta * direction;
                }
            }
        }

        transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, new_rot);
    }
}
