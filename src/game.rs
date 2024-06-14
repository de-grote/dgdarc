use bevy::prelude::*;

use crate::{despawn_screen, GameState};

mod hero;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hero>()
            .add_systems(OnEnter(GameState::Gaming), setup)
            .add_systems(OnExit(GameState::Gaming), despawn_screen::<GameWindow>);
    }
}

/// Annotate everything specific to the game window with this component
#[derive(Component)]
struct GameWindow;

#[derive(Resource, Clone, Copy, Default)]
struct Hero {}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::Rgba {
                    red: 0.3,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 0.0,
                }),
                ..default()
            },
            ..default()
        },
        GameWindow,
    ));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("test.png"),
            transform: Transform {
                scale: Vec3::splat(4.0),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(Vec2::splat(1000.0)),
                ..default()
            },
            ..default()
        },
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y: true,
            stretch_value: 1.0,
        },
        GameWindow,
    ));
}
