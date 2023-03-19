use bevy::prelude::*;

use super::ROT_AXIS_Z;

pub struct ArrowPlugin;

impl Plugin for ArrowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(arrow_update_system);
    }
}

#[derive(Component)]
pub struct Arrow {
    owner: Entity,
    is_moving: bool,
    start_pos: Vec2,
    velocity: f32,
    angle: f32,
    current_time: f32,
}

impl Arrow {
    pub fn new(owner: Entity, start_pos: Vec2, velocity: f32, angle: f32) -> Self {
        Self {
            owner,
            is_moving: true,
            start_pos,
            velocity,
            angle,
            current_time: 0.0,
        }
    }

    pub fn set_moving(&mut self, value: bool) {
        self.is_moving = value;
    }

    pub fn pos_at_time(&self, time: f32) -> Vec2 {
        let mut position = Arrow::get_trajectory(self.velocity, self.angle, time);
        position.x += self.start_pos.x;
        position.y += self.start_pos.y;
        position
    }

    pub fn get_trajectory(velocity: f32, angle: f32, t: f32) -> Vec2 {
        let mut x: f32 = velocity * t * f32::cos(angle);
        let mut y: f32 = velocity * t * f32::sin(angle);
        y -= 0.5 * 9.0 * t * t;
        x *= 4.0;
        y *= 4.0;
        Vec2::new(x, y)
    }
}

fn arrow_update_system(time: Res<Time>, mut arrows: Query<(&mut Arrow, &mut Transform)>) {
    for (mut arrow, mut transform) in arrows.iter_mut() {
        if !arrow.is_moving {
            continue;
        }

        arrow.current_time += time.delta_seconds() * 0.5;
        let new_pos = arrow.pos_at_time(arrow.current_time);
        let new_translation = Vec3::new(new_pos.x, new_pos.y, transform.translation.z);
        let diff = new_translation - transform.translation;
        let angle = f32::atan2(diff.y, diff.x);
        transform.rotation = Quat::from_axis_angle(ROT_AXIS_Z, angle);
        transform.translation = new_translation;
    }
}
