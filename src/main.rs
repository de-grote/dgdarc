#![allow(clippy::too_many_arguments)]

pub mod game;
pub mod level_select;
pub mod main_menu;
pub mod tile;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use game::hero::Hero;
use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<GameState>()
        .init_resource::<LevelScene>()
        .add_plugins((
            main_menu::MenuPlugin,
            level_select::LevelSelectPlugin,
            game::GamePlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .run();
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, States)]
pub enum GameState {
    #[default]
    MainMenu,
    LevelSelect,
    Gaming,
}

#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct LevelScene {
    pub level_name: String,
    pub background_texture: String,
    pub heros: Vec<Hero>, 
    // pub grid: Vec<Vec<Tile>>,
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
