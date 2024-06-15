use std::sync::OnceLock;

use bevy::prelude::*;

use crate::{despawn_screen, GameState, LevelScene};

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LevelSelect), (setup, reenter_level))
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
pub struct LevelSelectWindow;

#[derive(Component)]
pub struct Level(pub u8);

#[derive(Component)]
pub struct WonLevel(pub u8);

#[derive(Component)]
pub struct ReenterLevel(pub u8);

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

fn reenter_level(
    query: Query<&ReenterLevel>,
    mut scene: ResMut<LevelScene>,
    mut state: ResMut<NextState<GameState>>,
) {
    for q in query.iter() {
        *scene = load_scene(q.0);
        state.set(GameState::Gaming);
    }
}

fn load_scene(id: u8) -> LevelScene {
    if let Some(data) = LEVEL_DATA.get() {
        return data[id as usize - 1].clone();
    }

    let res: Vec<LevelScene> = (1..=NUMBER_OF_LEVELS)
        .map(|id| {
            let s = level(id);

            let mut scene = toml::from_str::<LevelScene>(s).unwrap();
            scene.level = id;
            scene
        })
        .collect();
    let out = res[id as usize - 1].clone();
    LEVEL_DATA.set(res).unwrap();
    out
}

const fn level(id: u8) -> &'static str {
    match id {
        1 => LEVEL1,
        _ => unimplemented!(),
    }
}

static LEVEL_DATA: OnceLock<Vec<LevelScene>> = OnceLock::new();

const NUMBER_OF_LEVELS: u8 = 1;
const LEVEL1: &str = include_str!("../levels/level1.toml");
