use bevy::prelude::*;

use crate::{despawn_screen, GameState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup)
            .add_systems(Update, start_game.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), despawn_screen::<MenuWindow>);
    }
}

/// Annotate everything specific to the menu window with this component
#[derive(Component)]
struct MenuWindow;

#[derive(Component)]
struct StartButton;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::LIME_GREEN),
                ..default()
            },
            ..default()
        },
        MenuWindow,
    ));

    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "EPIC GAME",
                TextStyle {
                    color: Color::PURPLE,
                    font_size: 100.0,
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(20.0),
                justify_self: JustifySelf::Center,
                min_width: Val::Percent(40.0),
                min_height: Val::Px(20.0),
                ..default()
            },
            ..default()
        },
        MenuWindow,
    ));

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(40.0),
                    justify_self: JustifySelf::Center,
                    min_width: Val::Percent(40.0),
                    min_height: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::PINK),
                border_color: BorderColor(Color::PURPLE),
                ..default()
            },
            StartButton,
            MenuWindow,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Start Game",
                    TextStyle {
                        font: default(),
                        font_size: 60.0,
                        color: Color::WHITE,
                    },
                )
                .with_text_justify(JustifyText::Center),
                StartButton,
            ));
        });
}

fn start_game(
    text_selection: Query<&Interaction, With<StartButton>>,
    mut state: ResMut<NextState<GameState>>,
) {
    for selection in text_selection.iter() {
        if *selection == Interaction::Pressed {
            state.set(GameState::LevelSelect);
        }
    }
}
