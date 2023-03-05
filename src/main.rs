use crate::game::GamePlugin;
use bevy::prelude::*;

mod game;

const BACKGROUND_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Game".to_string(),
                width: 1280.0,
                height: 720.0,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(GamePlugin)
        .run();
}
