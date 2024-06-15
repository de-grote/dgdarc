use bevy::prelude::*;

use crate::{despawn_screen, GameState, LevelScene};

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LevelSelect), setup)
            .add_systems(
                Update,
                button_pressed.run_if(in_state(GameState::LevelSelect)),
            )
            .add_systems(
                OnExit(GameState::LevelSelect),
                despawn_screen::<LevelSelectWindow>,
            );
    }
}

#[derive(Component)]
struct LevelSelectWindow;

#[derive(Component)]
struct Level(pub u8);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::LIME_GREEN),
                ..default()
            },
            ..default()
        },
        LevelSelectWindow,
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    top: Val::Px(50.0),
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
                    // grid_template_rows: RepeatedGridTrack::flex(4, 1.0),
                    row_gap: Val::Px(20.0),
                    column_gap: Val::Px(20.0),
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                ..default()
            },
            LevelSelectWindow,
        ))
        .with_children(|parent| {
            for i in 1..=NUMBER_OF_LEVELS {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                align_self: AlignSelf::Center,
                                ..default()
                            },
                            ..default()
                        },
                        Level(i),
                        LevelSelectWindow,
                    ))
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                format!("Level {i}"),
                                TextStyle {
                                    font_size: 50.0,
                                    color: Color::PURPLE,
                                    ..default()
                                },
                            )
                            .with_no_wrap(),
                        );
                    });
            }
        });
}

fn button_pressed(
    query: Query<(&Interaction, &Level)>,
    mut scene: ResMut<LevelScene>,
    mut state: ResMut<NextState<GameState>>,
) {
    for (interaction, level) in query.iter() {
        if *interaction == Interaction::Pressed {
            *scene = load_scene(level.0);
            state.set(GameState::Gaming);
        }
    }
}

fn load_scene(id: u8) -> LevelScene {
    let s = level(id);
    toml::from_str::<LevelScene>(s).unwrap()
}

const fn level(id: u8) -> &'static str {
    match id {
        1 => LEVEL1,
        _ => unimplemented!(),
    }
}

const NUMBER_OF_LEVELS: u8 = 1;
const LEVEL1: &str = include_str!("../levels/level1.toml");
