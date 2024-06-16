use std::sync::OnceLock;

use bevy::prelude::*;

use crate::{despawn_screen, GameState, LevelScene, BGM};

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelsWon>()
            .add_systems(OnEnter(GameState::LevelSelect), (setup, reenter_level))
            .add_systems(
                Update,
                (
                    button_pressed.run_if(in_state(GameState::LevelSelect)),
                    back_button_pressed.run_if(in_state(GameState::LevelSelect)),
                ),
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
pub struct Level(pub usize);

#[derive(Resource, Debug, Default, DerefMut, Deref)]
pub struct LevelsWon(pub [bool; NUMBER_OF_LEVELS]);

#[derive(Component)]
pub struct ReenterLevel(pub usize);

#[derive(Component)]
struct BackToMainButton;

fn setup(
    mut commands: Commands,
    levels_won: Res<LevelsWon>,
    mut bgm_query: Query<(&mut BGM, Entity)>,
    asset_server: Res<AssetServer>,
) {
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

    if let Ok((mut bgm, entity)) = bgm_query.get_single_mut() {
        if bgm.0 != "music/Main_menu.wav" {
            commands
                .entity(entity)
                .remove::<AudioSink>()
                .insert(asset_server.load::<AudioSource>("music/Main_menu.wav"));
            bgm.0 = "music/Main_menu.wav".to_string();
        }
    }

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
                            background_color: if levels_won[i - 1] {
                                Color::GOLD
                            } else {
                                Color::WHITE
                            }
                            .into(),
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

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                background_color: Color::PINK.into(),
                ..default()
            },
            BackToMainButton,
            LevelSelectWindow,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "Back",
                    TextStyle {
                        font_size: 50.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_no_wrap(),
            );
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

fn back_button_pressed(
    query: Query<&Interaction, With<BackToMainButton>>,
    mut state: ResMut<NextState<GameState>>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            state.set(GameState::MainMenu);
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

fn load_scene(id: usize) -> LevelScene {
    if let Some(data) = LEVEL_DATA.get() {
        return data[id - 1].clone();
    }

    let res: Vec<LevelScene> = (1..=NUMBER_OF_LEVELS)
        .map(|id| {
            let s = level(id);

            let mut scene = toml::from_str::<LevelScene>(s).unwrap();
            for (position, tile) in scene.points_of_interest.iter() {
                scene.points_of_interest_map.insert(*position, *tile);
            }

            scene.level = id;
            scene
        })
        .collect();
    let out = res[id - 1].clone();
    LEVEL_DATA.set(res).unwrap();
    out
}

const fn level(id: usize) -> &'static str {
    match id {
        1 => LEVEL1,
        2 => LEVEL2,
        _ => unimplemented!(),
    }
}

static LEVEL_DATA: OnceLock<Vec<LevelScene>> = OnceLock::new();

const NUMBER_OF_LEVELS: usize = 2;
const LEVEL1: &str = include_str!("../levels/level1.toml");
const LEVEL2: &str = include_str!("../levels/level2.toml");
