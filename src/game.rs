use bevy::{prelude::*, render::camera::ScalingMode};

use self::animation::AnimationPlugin;
use self::archer::ArcherPlugin;
use self::arrow::ArrowPlugin;
use self::collision::CollisionPlugin;
use self::player_controls::PlayerControlsPlugin;

mod animation;
mod archer;
mod arrow;
mod collision;
mod player_controls;

const ROT_AXIS_Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AnimationPlugin)
            .add_plugin(PlayerControlsPlugin)
            .add_plugin(ArcherPlugin)
            .add_plugin(ArrowPlugin)
            .add_plugin(CollisionPlugin)
            .add_startup_system_set_to_stage(
                StartupStage::PreStartup,
                SystemSet::new()
                    .with_system(setup_camera)
                    .with_system(setup_resources),
            );
    }
}

#[derive(Resource, Default)]
struct GameTextures {
    archer_blue_idle: Handle<TextureAtlas>,
    archer_blue_body: Handle<Image>,
    archer_blue_head: Handle<Image>,
    archer_blue_arm: Handle<Image>,
    archer_blue_arm_pull: Handle<TextureAtlas>,
    archer_bow: Handle<TextureAtlas>,
    archer_arrow: Handle<Image>,
}

#[derive(Component)]
struct MainCamera;

enum GameStage {
    Menu,
    Playing,
    Finished(GameTurn),
}

enum GameTurn {
    Player,
    Enemy,
}

#[derive(Resource)]
struct GameState {
    stage: GameStage,
    needs_refresh: bool,
    turn: GameTurn,
    waiting_for_hit: bool,
    turn_count: u32,
}

impl GameState {
    fn new() -> Self {
        Self {
            stage: GameStage::Menu,
            needs_refresh: true,
            turn: GameTurn::Player,
            waiting_for_hit: false,
            turn_count: 0,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(18.0),
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

    let game_textures = GameTextures {
        archer_blue_idle: archer_blue_idle_atlas_handle,
        archer_blue_body: archer_blue_body_texture,
        archer_blue_head: archer_blue_head_texture,
        archer_blue_arm: archer_blue_arm_texture,
        archer_blue_arm_pull: archer_blue_arm_pull_atlas_handle,
        archer_bow: archer_bow_atlas_handle,
        archer_arrow: archer_arrow_texture,
    };

    commands.insert_resource(game_textures);
}
