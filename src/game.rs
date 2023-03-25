use bevy::app::AppExit;
use bevy::sprite::Anchor;
use bevy::{prelude::*, render::camera::ScalingMode};
use rand::Rng;

use self::ai_controls::{AIControls, AIControlsPlugin};
use self::animation::AnimationPlugin;
use self::archer::{spawn_archer, Archer, ArcherEnemy, ArcherPlayer, ArcherPlugin, DamageReceiver};
use self::arrow::{Arrow, ArrowPlugin};
use self::collision::{CollisionPlugin, RectCollider};
use self::player_controls::{PlayerControls, PlayerControlsPlugin};

mod ai_controls;
mod animation;
mod archer;
mod arrow;
mod collision;
mod player_controls;

const DIFFICULTY: f32 = 0.8;

const ROT_AXIS_Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const CAMERA_SCALING_MENU: f32 = 6.0;
const CAMERA_SCALING_GAME: f32 = 17.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AnimationPlugin)
            .add_plugin(PlayerControlsPlugin)
            .add_plugin(AIControlsPlugin)
            .add_plugin(ArcherPlugin)
            .add_plugin(ArrowPlugin)
            .add_plugin(CollisionPlugin)
            .add_startup_system_set_to_stage(
                StartupStage::PreStartup,
                SystemSet::new()
                    .with_system(setup_camera)
                    .with_system(setup_resources),
            )
            .add_system(setup_game_stage_update_system)
            .add_system(menu_buttons_update_system)
            .add_system(game_arrow_update_system)
            .add_system(finished_game_update_system);
    }
}

#[derive(Resource, Default)]
pub struct GameTextures {
    archer_blue_idle: Handle<TextureAtlas>,
    archer_blue_body: Handle<Image>,
    archer_blue_head: Handle<Image>,
    archer_blue_arm: Handle<Image>,
    archer_blue_arm_pull: Handle<TextureAtlas>,
    archer_bow: Handle<TextureAtlas>,
    archer_arrow: Handle<Image>,
    start_button: Handle<Image>,
    credits_button: Handle<Image>,
    back_button: Handle<Image>,
    quit_button: Handle<Image>,
    credits: Handle<Image>,
    menu_background: Handle<Image>,
    game_background: Handle<Image>,
    tower: Handle<Image>,
    victory: Handle<Image>,
    defeat: Handle<Image>,
}

#[derive(Component)]
pub struct MainCamera;

#[derive(PartialEq, Eq)]
pub enum GameStage {
    Menu,
    Credits,
    StartGame,
    Playing,
    ChangeTurn,
    Finished(GameTurn),
}

#[derive(Component)]
pub struct GameStageSpawned;

#[derive(Component)]
pub struct DespawnedOnNewTurn;

#[derive(PartialEq, Eq)]
pub enum GameTurn {
    Player,
    Enemy,
}

#[derive(Resource)]
pub struct GameState {
    pub stage: GameStage,
    pub needs_refresh: bool,
    pub turn: GameTurn,
    pub wait_for: f32,
    pub waiting_for_hit: bool,
    pub turn_count: i32,
    pub player_height: f32,
    pub enemy_height: f32,
    pub player_health: i32,
    pub enemy_health: i32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            stage: GameStage::Menu,
            needs_refresh: true,
            turn: GameTurn::Enemy,
            wait_for: 0.0,
            waiting_for_hit: false,
            turn_count: -1,
            player_height: 0.5,
            enemy_height: 0.5,
            player_health: 10,
            enemy_health: 10,
        }
    }
}

#[derive(Component)]
enum MenuButton {
    Start,
    Credits,
    BackFromCredits,
    Quit,
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(CAMERA_SCALING_GAME),
                scale: 1.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MainCamera);
}

fn setup_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.insert_resource(GameState::new());

    let texture = asset_server.load("textures/archer_blue_idle.png");
    let atlas = TextureAtlas::from_grid(texture, Vec2::new(64.0, 64.0), 2, 2, None, None);
    let archer_blue_idle_atlas_handle = texture_atlases.add(atlas);

    let archer_blue_body_texture = asset_server.load("textures/archer_blue_body.png");

    let archer_blue_head_texture = asset_server.load("textures/archer_blue_head.png");

    let archer_blue_arm_texture = asset_server.load("textures/archer_blue_arm.png");

    let texture = asset_server.load("textures/archer_blue_arm_pull.png");
    let atlas = TextureAtlas::from_grid(texture, Vec2::new(64.0, 64.0), 3, 2, None, None);
    let archer_blue_arm_pull_atlas_handle = texture_atlases.add(atlas);

    let texture = asset_server.load("textures/archer_bow.png");
    let atlas = TextureAtlas::from_grid(texture, Vec2::new(64.0, 64.0), 3, 2, None, None);
    let archer_bow_atlas_handle = texture_atlases.add(atlas);

    let archer_arrow_texture = asset_server.load("textures/archer_arrow.png");

    let start_button_texture = asset_server.load("textures/start_button.png");
    let credits_button_texture = asset_server.load("textures/credits_button.png");
    let back_button_texture = asset_server.load("textures/back_button.png");
    let quit_button_texture = asset_server.load("textures/quit_button.png");
    let credits_texture = asset_server.load("textures/credits.png");

    let menu_background_texture = asset_server.load("textures/menu_background.png");
    let game_background_texture = asset_server.load("textures/game_background.png");

    let tower_texture = asset_server.load("textures/tower.png");

    let victory_texture = asset_server.load("textures/victory.png");
    let defeat_texture = asset_server.load("textures/defeat.png");

    let game_textures = GameTextures {
        archer_blue_idle: archer_blue_idle_atlas_handle,
        archer_blue_body: archer_blue_body_texture,
        archer_blue_head: archer_blue_head_texture,
        archer_blue_arm: archer_blue_arm_texture,
        archer_blue_arm_pull: archer_blue_arm_pull_atlas_handle,
        archer_bow: archer_bow_atlas_handle,
        archer_arrow: archer_arrow_texture,
        start_button: start_button_texture,
        credits_button: credits_button_texture,
        back_button: back_button_texture,
        quit_button: quit_button_texture,
        credits: credits_texture,
        menu_background: menu_background_texture,
        game_background: game_background_texture,
        tower: tower_texture,
        victory: victory_texture,
        defeat: defeat_texture,
    };

    commands.insert_resource(game_textures);
}

fn setup_game_stage_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut cameras: Query<(&Camera, &mut OrthographicProjection), With<MainCamera>>,
    mut game_state: ResMut<GameState>,
    mut player_controls: ResMut<PlayerControls>,
    mut ai_controls: ResMut<AIControls>,
    game_textures: Res<GameTextures>,
    mut stage_spawned: Query<(Entity, &mut Visibility, &GameStageSpawned)>,
    despawned_on_new_turn: Query<Entity, With<DespawnedOnNewTurn>>,
    mut archers_player: Query<&mut Transform, (With<ArcherPlayer>, Without<ArcherEnemy>)>,
    mut archers_enemy: Query<&mut Transform, (With<ArcherEnemy>, Without<ArcherPlayer>)>,
) {
    if game_state.wait_for > 0.0 {
        game_state.wait_for -= time.delta_seconds();
        return;
    }

    if !game_state.needs_refresh {
        return;
    }

    game_state.needs_refresh = false;
    let (_, mut camera_projection) = cameras.single_mut();

    let mut clear_scene = || {
        for (entity, mut visibility, _) in stage_spawned.iter_mut() {
            visibility.is_visible = false;
            commands.entity(entity).despawn_recursive();
        }
    };

    match &game_state.stage {
        GameStage::Menu => {
            clear_scene();
            game_state.waiting_for_hit = false;
            camera_projection.scaling_mode = ScalingMode::FixedVertical(CAMERA_SCALING_MENU);
            player_controls.set_enabled(true);
            player_controls.reset();
            game_state.player_health = 10;
            game_state.enemy_health = 10;

            commands
                .spawn(SpriteBundle {
                    texture: game_textures.menu_background.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(12.0, 6.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    ..default()
                })
                .insert(GameStageSpawned);

            let archer_player = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(-3.5, -3.0, 0.1)),
                    ..default()
                })
                .insert(Archer::new(false))
                .insert(ArcherPlayer)
                .insert(GameStageSpawned)
                .id();

            spawn_archer(&mut commands, &game_textures, archer_player, true);

            let start = commands
                .spawn(SpriteBundle {
                    texture: game_textures.start_button.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(3.0, 3.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(2.0, 1.9, 0.1)),
                    ..default()
                })
                .insert(MenuButton::Start)
                .insert(GameStageSpawned)
                .id();

            commands
                .entity(start)
                .insert(RectCollider::new(start.into(), Vec2::ZERO, 1.0, 1.0));

            let credits = commands
                .spawn(SpriteBundle {
                    texture: game_textures.credits_button.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(3.0, 3.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(4.0, 0.0, 0.1)),
                    ..default()
                })
                .insert(MenuButton::Credits)
                .insert(GameStageSpawned)
                .id();

            commands.entity(credits).insert(RectCollider::new(
                credits.into(),
                Vec2::ZERO,
                1.0,
                1.0,
            ));

            let quit = commands
                .spawn(SpriteBundle {
                    texture: game_textures.quit_button.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(3.0, 3.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(2.0, -1.9, 0.1)),
                    ..default()
                })
                .insert(MenuButton::Quit)
                .insert(GameStageSpawned)
                .id();

            commands
                .entity(quit)
                .insert(RectCollider::new(quit.into(), Vec2::ZERO, 1.0, 1.0));
        }
        GameStage::Credits => {
            clear_scene();
            camera_projection.scaling_mode = ScalingMode::FixedVertical(CAMERA_SCALING_MENU);
            player_controls.set_enabled(true);
            player_controls.reset();

            commands
                .spawn(SpriteBundle {
                    texture: game_textures.menu_background.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(12.0, 6.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    ..default()
                })
                .insert(GameStageSpawned);

            let archer_player = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(-3.5, -3.0, 0.1)),
                    ..default()
                })
                .insert(Archer::new(false))
                .insert(ArcherPlayer)
                .insert(GameStageSpawned)
                .id();

            spawn_archer(&mut commands, &game_textures, archer_player, true);

            commands
                .spawn(SpriteBundle {
                    texture: game_textures.credits.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(4.0, 4.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(2.0, 0.5, 0.1)),
                    ..default()
                })
                .insert(GameStageSpawned);

            let back = commands
                .spawn(SpriteBundle {
                    texture: game_textures.back_button.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(3.0, 3.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(2.0, -2.0, 0.1)),
                    ..default()
                })
                .insert(MenuButton::BackFromCredits)
                .insert(GameStageSpawned)
                .id();

            commands
                .entity(back)
                .insert(RectCollider::new(back.into(), Vec2::ZERO, 1.0, 1.0));
        }
        GameStage::StartGame => {
            clear_scene();
            camera_projection.scaling_mode = ScalingMode::FixedVertical(CAMERA_SCALING_GAME);

            commands
                .spawn(SpriteBundle {
                    texture: game_textures.game_background.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(32.0, 18.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    ..default()
                })
                .insert(GameStageSpawned);

            let archer_player = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(-12.0, 0.0, 0.2)),
                    ..default()
                })
                .insert(Archer::new(false))
                .insert(ArcherPlayer)
                .insert(GameStageSpawned)
                .id();

            let tower_player = commands
                .spawn(SpriteBundle {
                    texture: game_textures.tower.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(4.0, 16.0).into(),
                        anchor: Anchor::TopCenter,
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                    ..default()
                })
                .insert(GameStageSpawned)
                .id();

            commands.entity(archer_player).add_child(tower_player);

            let archer_enemy = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(12.0, 0.0, 0.2))
                        .with_scale(Vec3::new(-1.0, 1.0, 1.0)),
                    ..default()
                })
                .insert(Archer::new(true))
                .insert(ArcherEnemy)
                .insert(GameStageSpawned)
                .id();

            let tower_enemy = commands
                .spawn(SpriteBundle {
                    texture: game_textures.tower.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(4.0, 16.0).into(),
                        anchor: Anchor::TopCenter,
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                    ..default()
                })
                .insert(GameStageSpawned)
                .id();

            commands.entity(archer_enemy).add_child(tower_enemy);

            spawn_archer(&mut commands, &game_textures, archer_player, true);
            spawn_archer(&mut commands, &game_textures, archer_enemy, false);

            game_state.stage = GameStage::ChangeTurn;
            game_state.needs_refresh = true;
        }
        GameStage::Playing => {
            player_controls.reset();
            game_state.player_height = rand::thread_rng().gen_range(0.0..=1.0);
            game_state.enemy_height = rand::thread_rng().gen_range(0.0..=1.0);
            game_state.waiting_for_hit = false;
            player_controls.set_enabled(match game_state.turn {
                GameTurn::Player => true,
                GameTurn::Enemy => false,
            });
            ai_controls.set_enabled(true);

            let player_height = (game_state.player_height * 12.0) - (17.0 * 0.5) + 1.0;
            let enemy_height = (game_state.enemy_height * 12.0) - (17.0 * 0.5) + 1.0;

            for mut transform in archers_player.iter_mut() {
                transform.translation.y = player_height;
            }

            for mut transform in archers_enemy.iter_mut() {
                transform.translation.y = enemy_height;
            }
        }
        GameStage::ChangeTurn => {
            for entity in despawned_on_new_turn.iter() {
                commands.entity(entity).despawn_recursive();
            }

            game_state.turn = match &game_state.turn {
                GameTurn::Player => GameTurn::Enemy,
                GameTurn::Enemy => GameTurn::Player,
            };
            game_state.turn_count += 1;
            game_state.stage = GameStage::Playing;
            game_state.needs_refresh = true;
        }
        GameStage::Finished(winner) => {
            clear_scene();
            camera_projection.scaling_mode = ScalingMode::FixedVertical(CAMERA_SCALING_MENU);

            commands
                .spawn(SpriteBundle {
                    texture: game_textures.menu_background.clone(),
                    sprite: Sprite {
                        custom_size: Vec2::new(12.0, 6.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    ..default()
                })
                .insert(GameStageSpawned);

            let text_texture = match winner {
                GameTurn::Player => game_textures.victory.clone(),
                GameTurn::Enemy => game_textures.defeat.clone(),
            };

            commands
                .spawn(SpriteBundle {
                    texture: text_texture,
                    sprite: Sprite {
                        custom_size: Vec2::new(8.0, 4.0).into(),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                    ..default()
                })
                .insert(GameStageSpawned);
        }
    }
}

fn menu_buttons_update_system(
    mut game_state: ResMut<GameState>,
    mut exit: EventWriter<AppExit>,
    buttons: Query<(&MenuButton, &RectCollider)>,
    mut arrows: Query<(&mut Arrow, &mut RectCollider), Without<MenuButton>>,
) {
    if game_state.needs_refresh {
        return;
    }

    for (button, collider) in buttons.iter() {
        for (mut arrow, mut arrow_collider) in arrows.iter_mut() {
            if collider.aabb_collides_with(&arrow_collider) {
                match button {
                    MenuButton::Start => {
                        if game_state.stage == GameStage::Menu {
                            arrow_collider.disable();
                            arrow.set_moving(false);
                            game_state.stage = GameStage::StartGame;
                            game_state.needs_refresh = true;
                            game_state.wait_for = 0.2;
                        }
                    }
                    MenuButton::Credits => {
                        if game_state.stage == GameStage::Menu {
                            arrow_collider.disable();
                            arrow.set_moving(false);
                            game_state.stage = GameStage::Credits;
                            game_state.needs_refresh = true;
                            game_state.wait_for = 0.2;
                        }
                    }
                    MenuButton::BackFromCredits => {
                        if game_state.stage == GameStage::Credits {
                            arrow_collider.disable();
                            arrow.set_moving(false);
                            game_state.stage = GameStage::Menu;
                            game_state.needs_refresh = true;
                            game_state.wait_for = 0.2;
                        }
                    }
                    MenuButton::Quit => {
                        if game_state.stage == GameStage::Menu {
                            arrow_collider.disable();
                            arrow.set_moving(false);
                            exit.send(AppExit);
                        }
                    }
                }

                return;
            }
        }
    }
}

pub fn finished_game_update_system(
    keyboard: Res<Input<KeyCode>>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.needs_refresh {
        return;
    }

    if let GameStage::Finished(_) = game_state.stage {
        if keyboard.any_just_pressed([KeyCode::Space, KeyCode::Return]) {
            game_state.stage = GameStage::Menu;
            game_state.needs_refresh = true;
            game_state.wait_for = 0.2;
        }
    }
}

fn game_arrow_update_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut arrows: Query<(Entity, &mut Arrow, &mut RectCollider, &Transform)>,
    damage_receivers: Query<(&RectCollider, &DamageReceiver), Without<Arrow>>,
    archers_player: Query<&ArcherPlayer>,
    archers_enemy: Query<&ArcherEnemy>,
) {
    if !(game_state.stage == GameStage::Playing && game_state.waiting_for_hit)
        || game_state.needs_refresh
    {
        return;
    }

    let mut arrow_out_of_bounds = false;
    let mut hit_archer = false;
    let mut killed_archer = false;
    let mut winner = GameTurn::Player;
    for (arrow_entity, mut arrow, mut arrow_collider, arrow_transform) in arrows.iter_mut() {
        let arrow_pos = arrow_transform.translation.truncate();

        for (damage_collider, damage_receiver) in damage_receivers.iter() {
            if arrow_collider.owner == damage_collider.owner {
                continue;
            }

            if arrow_collider.aabb_collides_with(damage_collider) {
                if let Some(collider_owner) = damage_collider.owner {
                    if archers_player.get(collider_owner).is_ok() {
                        commands.entity(arrow_entity).insert(DespawnedOnNewTurn);
                        arrow_collider.disable();
                        arrow.set_moving(false);
                        hit_archer = true;

                        game_state.player_health -= damage_receiver.hitpoints;
                        if game_state.player_health <= 0 {
                            game_state.player_health = 0;

                            killed_archer = true;
                            winner = GameTurn::Enemy;
                        }
                    }

                    if archers_enemy.get(collider_owner).is_ok() {
                        commands.entity(arrow_entity).insert(DespawnedOnNewTurn);
                        arrow_collider.disable();
                        arrow.set_moving(false);
                        hit_archer = true;

                        game_state.enemy_health -= damage_receiver.hitpoints;
                        if game_state.enemy_health <= 0 {
                            game_state.enemy_health = 0;

                            killed_archer = true;
                            winner = GameTurn::Player;
                        }
                    }
                }
            }
        }

        if arrow_pos.x > 20.0 || arrow_pos.x < -20.0 || arrow_pos.y > 20.0 || arrow_pos.y < -20.0 {
            commands.entity(arrow_entity).despawn_recursive();
            arrow_collider.disable();

            arrow_out_of_bounds = true;
            break;
        }
    }

    if killed_archer {
        game_state.stage = GameStage::Finished(winner);
        game_state.needs_refresh = true;
        game_state.wait_for = 0.5;
        return;
    }

    if arrow_out_of_bounds || hit_archer {
        game_state.stage = GameStage::ChangeTurn;
        game_state.needs_refresh = true;
        game_state.wait_for = 0.5;
    }
}
