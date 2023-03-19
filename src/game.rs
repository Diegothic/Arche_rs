use bevy::app::AppExit;
use bevy::{prelude::*, render::camera::ScalingMode};
use rand::Rng;

use self::ai_controls::AIControlsPlugin;
use self::animation::AnimationPlugin;
use self::archer::{spawn_archer, Archer, ArcherEnemy, ArcherPlayer, ArcherPlugin};
use self::arrow::{Arrow, ArrowPlugin};
use self::collision::{CollisionPlugin, RectCollider};
use self::player_controls::{PlayerControls, PlayerControlsPlugin};

mod ai_controls;
mod animation;
mod archer;
mod arrow;
mod collision;
mod player_controls;

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
            .add_system(menu_buttons_update_system);
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
}

#[derive(Component)]
pub struct MainCamera;

#[derive(PartialEq, Eq)]
pub enum GameStage {
    Menu,
    Credits,
    Playing,
    ChangeTurn,
    Finished(GameTurn),
}

#[derive(Component)]
pub struct GameStageSpawned;

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
    pub waiting_for_hit: bool,
    pub turn_count: i32,
    pub player_height: f32,
    pub enemy_height: f32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            stage: GameStage::Menu,
            needs_refresh: true,
            turn: GameTurn::Enemy,
            waiting_for_hit: false,
            turn_count: -1,
            player_height: 0.5,
            enemy_height: 0.5,
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
    };

    commands.insert_resource(game_textures);
}

fn setup_game_stage_update_system(
    mut commands: Commands,
    mut cameras: Query<(&Camera, &mut OrthographicProjection), With<MainCamera>>,
    mut game_state: ResMut<GameState>,
    mut player_controls: ResMut<PlayerControls>,
    game_textures: Res<GameTextures>,
    mut stage_spawned: Query<(Entity, &mut Visibility, &GameStageSpawned)>,
) {
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
            camera_projection.scaling_mode = ScalingMode::FixedVertical(CAMERA_SCALING_MENU);
            player_controls.set_enabled(true);
            player_controls.reset();

            let archer_player = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(-3.5, -3.0, 0.0)),
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
                    transform: Transform::from_translation(Vec3::new(2.0, 1.9, 0.0)),
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
                    transform: Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
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
                    transform: Transform::from_translation(Vec3::new(2.0, -1.9, 0.0)),
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

            let archer_player = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(-3.5, -3.0, 0.0)),
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
                    transform: Transform::from_translation(Vec3::new(2.0, 0.5, 0.0)),
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
                    transform: Transform::from_translation(Vec3::new(2.0, -2.0, 0.0)),
                    ..default()
                })
                .insert(MenuButton::BackFromCredits)
                .insert(GameStageSpawned)
                .id();

            commands
                .entity(back)
                .insert(RectCollider::new(back.into(), Vec2::ZERO, 1.0, 1.0));
        }
        GameStage::Playing => {
            clear_scene();
            camera_projection.scaling_mode = ScalingMode::FixedVertical(CAMERA_SCALING_GAME);
            player_controls.reset();
            game_state.player_height = rand::thread_rng().gen_range(0.0..=1.0);
            game_state.enemy_height = rand::thread_rng().gen_range(0.0..=1.0);

            let player_height = (game_state.player_height * 14.0) - (17.0 * 0.5) + 1.0;
            let archer_player = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(-12.0, player_height, 0.0)),
                    ..default()
                })
                .insert(Archer::new(false))
                .insert(ArcherPlayer)
                .insert(GameStageSpawned)
                .id();

            let enemy_height = (game_state.enemy_height * 14.0) - (17.0 * 0.5) + 1.0;
            let archer_enemy = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(12.0, enemy_height, 0.0))
                        .with_scale(Vec3::new(-1.0, 1.0, 1.0)),
                    ..default()
                })
                .insert(Archer::new(true))
                .insert(ArcherEnemy)
                .insert(GameStageSpawned)
                .id();

            spawn_archer(&mut commands, &game_textures, archer_player, true);
            spawn_archer(&mut commands, &game_textures, archer_enemy, false);
        }
        GameStage::ChangeTurn => {
            game_state.turn = match &game_state.turn {
                GameTurn::Player => GameTurn::Enemy,
                GameTurn::Enemy => GameTurn::Player,
            };
            game_state.turn_count += 1;
            game_state.waiting_for_hit = false;
            game_state.stage = GameStage::Playing;
            player_controls.set_enabled(match game_state.turn {
                GameTurn::Player => true,
                GameTurn::Enemy => false,
            });
            game_state.needs_refresh = true;
        }
        GameStage::Finished(winner) => {
            clear_scene();
            camera_projection.scaling_mode = ScalingMode::FixedVertical(CAMERA_SCALING_MENU);
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
                            game_state.stage = GameStage::ChangeTurn;
                            game_state.needs_refresh = true;
                        }
                    }
                    MenuButton::Credits => {
                        if game_state.stage == GameStage::Menu {
                            arrow_collider.disable();
                            arrow.set_moving(false);
                            game_state.stage = GameStage::Credits;
                            game_state.needs_refresh = true;
                        }
                    }
                    MenuButton::BackFromCredits => {
                        if game_state.stage == GameStage::Credits {
                            arrow_collider.disable();
                            arrow.set_moving(false);
                            game_state.stage = GameStage::Menu;
                            game_state.needs_refresh = true;
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

pub fn eval_shot(power: f32, angle: f32, self_height: f32, enemy_height: f32) -> f32 {
    let self_height_real = (self_height * 14.0) - (17.0 * 0.5) + 1.0;
    let enemy_height_real = (enemy_height * 14.0) - (17.0 * 0.5) + 1.0;

    let shooting_vec = Vec2::new(f32::cos(angle), f32::sin(angle)) * 2.55;
    let mut shoot_pos = Vec2::new(-12.0, self_height_real);
    shoot_pos += Vec2::new(0.3, 2.1);
    shoot_pos += shooting_vec;

    let mut arrow_col = RectCollider::new(None, Vec2::ZERO, 0.3, 0.3);
    arrow_col.set_center(shoot_pos);

    let enemy_pos = Vec2::new(12.0, enemy_height_real);
    let mut enemy_legs_col = RectCollider::new(None, Vec2::ZERO, 0.8, 1.2);
    let enemy_legs_pos = enemy_pos + Vec2::new(0.0, 0.7);
    enemy_legs_col.set_center(enemy_legs_pos);

    let mut enemy_body_col = RectCollider::new(None, Vec2::ZERO, 0.8, 1.0);
    let enemy_body_pos = enemy_pos + Vec2::new(0.0, 1.9);
    enemy_body_col.set_center(enemy_body_pos);

    let mut enemy_head_col = RectCollider::new(None, Vec2::ZERO, 0.7, 0.7);
    let enemy_head_pos = enemy_pos + Vec2::new(0.0, 2.8);
    enemy_head_col.set_center(enemy_head_pos);

    let mut t = 0.0;
    loop {
        let arrow_pos = Arrow::get_trajectory(power * 10.0, angle, t);
        let arrow_col_pos = shoot_pos + arrow_pos;
        arrow_col.set_center(arrow_col_pos);
        if arrow_col.aabb_collides_with(&enemy_head_col) {
            return 1.0;
        }

        if arrow_col.aabb_collides_with(&enemy_body_col) {
            return 0.8;
        }

        if arrow_col.aabb_collides_with(&enemy_legs_col) {
            return 0.5;
        }

        if arrow_col_pos.x > 12.0 || arrow_col_pos.y > 12.0 || arrow_col_pos.y < -12.0 {
            let mut dist = arrow_col_pos.distance(enemy_head_pos);
            dist = dist.min(10.0);
            dist /= 10.0;
            dist = 1.0 - dist;
            dist *= 0.5;
            return dist;
        }

        t += 0.01;
    }
}
