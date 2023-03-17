use bevy::{prelude::*, sprite::Anchor};

use crate::game::MainCamera;

use super::ROT_AXIS_Z;

pub struct PlayerControlsPlugin;

impl Plugin for PlayerControlsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseWorldPos(Vec3::ZERO))
            .insert_resource(PlayerControls::new(true, 5.0))
            .add_startup_system(setup_player_controls_startup_system)
            .add_system(mouse_world_pos_update_system)
            .add_system(player_controls_update_system)
            .add_system(player_controls_indicator_update_system);
    }
}

#[derive(Resource, Default)]
pub struct PlayerControls {
    is_enabled: bool,
    is_aiming: bool,
    hook_pos: Vec3,
    current_pos: Vec3,
    distance: f32,
    max_distance: f32,
    can_shot_arrow: bool,
    has_shot_arrow: bool,
    indicator_color: Color,
}

impl PlayerControls {
    pub fn new(is_enabled: bool, max_distance: f32) -> Self {
        Self {
            is_enabled,
            max_distance,
            indicator_color: Color::WHITE,
            ..default()
        }
    }

    pub fn enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn aiming(&self) -> bool {
        self.is_aiming
    }

    pub fn percent(&self) -> f32 {
        self.distance / self.max_distance
    }

    pub fn angle(&self) -> f32 {
        let diff = self.hook_pos - self.current_pos;
        f32::atan2(diff.y, diff.x)
    }

    pub fn should_shoot_arrow(&self) -> bool {
        !self.can_shot_arrow && self.has_shot_arrow
    }

    pub fn reset_shooting(&mut self) {
        self.can_shot_arrow = true;
        self.has_shot_arrow = false;
    }

    pub fn set_indicator_color(&mut self, new_color: Color) {
        self.indicator_color = new_color;
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct MouseWorldPos(Vec3);

#[derive(Component)]
struct PlayerControlsIndicator;

fn setup_player_controls_startup_system(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::CenterRight,
                ..default()
            },
            ..default()
        })
        .insert(PlayerControlsIndicator);
}

fn mouse_world_pos_update_system(
    mut mouse_world_pos: ResMut<MouseWorldPos>,
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if let Some(window) = windows.get_primary() {
        let (camera, camera_transform) = cameras.single();
        if let Some(world_pos) = window
            .cursor_position()
            .and_then(|cursor_pos| camera.viewport_to_world(camera_transform, cursor_pos))
            .map(|ray| ray.origin)
        {
            mouse_world_pos.0 = world_pos;
        }
    }
}

fn player_controls_update_system(
    mut player_controls: ResMut<PlayerControls>,
    mouse_world_pos: Res<MouseWorldPos>,
    mouse_buttons: Res<Input<MouseButton>>,
) {
    if !player_controls.is_enabled {
        return;
    }

    if !player_controls.is_aiming {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            player_controls.is_aiming = true;
            player_controls.hook_pos = mouse_world_pos.0;
            player_controls.current_pos = mouse_world_pos.0;
            player_controls.reset_shooting();
        }
        return;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if player_controls.can_shot_arrow {
            player_controls.has_shot_arrow = true;
            player_controls.can_shot_arrow = false;
        }

        player_controls.is_aiming = false;
        player_controls.hook_pos = mouse_world_pos.0;
        player_controls.current_pos = mouse_world_pos.0;
        player_controls.distance = 0.0;
        return;
    }

    player_controls.current_pos = mouse_world_pos.0;
    player_controls.distance = f32::min(
        player_controls
            .hook_pos
            .distance(player_controls.current_pos)
            .abs(),
        player_controls.max_distance,
    );
}

fn player_controls_indicator_update_system(
    player_controls: Res<PlayerControls>,
    mut indicators: Query<(&mut Transform, &mut Sprite), With<PlayerControlsIndicator>>,
) {
    for (mut transform, mut sprite) in indicators.iter_mut() {
        if !player_controls.is_enabled || !player_controls.is_aiming {
            sprite.color.set_a(0.0);
            return;
        }

        transform.translation = player_controls.hook_pos;
        transform.translation.z = 2.0;
        transform.rotation = Quat::from_axis_angle(ROT_AXIS_Z, player_controls.angle());
        sprite.custom_size = Vec2::new(player_controls.distance, 0.05).into();
        sprite.color = player_controls.indicator_color;
        sprite.color.set_a(1.0);
    }
}
