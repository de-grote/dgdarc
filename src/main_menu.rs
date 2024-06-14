use bevy::prelude::*;

use crate::{despawn_screen, GameState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup)
            .add_systems(OnExit(GameState::MainMenu), despawn_screen::<MenuWindow>);
    }
}

/// Annotate everything specific to the menu window with this component
#[derive(Component)]
struct MenuWindow;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::Rgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                }),
                ..default()
            },
            ..default()
        },
        MenuWindow,
    ));
}
