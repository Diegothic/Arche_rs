use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(sprite_animation_system);
    }
}

pub enum AnimationMode {
    Manual,
    Automatic,
}

#[derive(Component)]
pub struct Animation {
    first_index: usize,
    last_index: usize,
    frame_time: f32,
    current_frame: usize,
    current_time: f32,
    mode: AnimationMode,
}

impl Animation {
    pub fn new(
        first_index: usize,
        frame_count: usize,
        frame_time: f32,
        mode: AnimationMode,
    ) -> Self {
        Self {
            first_index,
            last_index: first_index + frame_count - 1,
            frame_time,
            current_frame: first_index,
            current_time: 0.0,
            mode,
        }
    }

    pub fn advance(&mut self, elapsed_time: f32) {
        self.current_time += elapsed_time;
        if self.current_time >= self.frame_time {
            let times = (self.current_time / self.frame_time).floor();
            let frame_count = self.last_index - self.first_index + 1;
            let frame = (self.current_frame + times as usize) % frame_count;
            self.current_time -= self.frame_time * times;
            self.current_frame = frame;
        }
    }

    pub fn set_progress(&mut self, mut progress: f32) {
        progress = progress.clamp(0.0, 1.0);
    }
}

fn sprite_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut Animation, &mut TextureAtlasSprite)>,
) {
    for (mut animation, mut sprite) in query.iter_mut() {
        match animation.mode {
            AnimationMode::Automatic => {
                animation.advance(time.delta_seconds());
                sprite.index = animation.current_frame;
            }
            AnimationMode::Manual => (),
        }
    }
}
