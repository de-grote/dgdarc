#![allow(clippy::too_many_arguments)]

pub mod main_menu;
pub mod game;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_plugins((
            main_menu::MenuPlugin,
            game::GamePlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .run();
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, States)]
pub enum GameState {
    #[default]
    MainMenu,
    Gaming,
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
