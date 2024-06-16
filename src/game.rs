use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use bevy::input::mouse::MouseWheel;
use bevy::sprite::Anchor;
use bevy::{prelude::*, window::PrimaryWindow};

use crate::level_select::{LevelSelectWindow, LevelsWon, ReenterLevel};
use crate::tile::make_tile;
use crate::{despawn_screen, EndGameEvent, GameState, LevelScene, BGM};
use hero::*;

pub mod hero;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Spell>()
            .init_state::<GameRunning>()
            .add_systems(OnEnter(GameState::Gaming), (setup, create_hero))
            .add_systems(
                Update,
                (
                    select_spell_button.run_if(in_state(GameState::Gaming)),
                    select_spell_keybind.run_if(in_state(GameState::Gaming)),
                    highlight_selected_spell.run_if(in_state(GameState::Gaming)),
                    animate_and_despawn_fire.run_if(in_state(GameState::Gaming)),
                    animate_and_despawn_healing.run_if(in_state(GameState::Gaming)),
                    animate_and_despawn_gust.run_if(in_state(GameState::Gaming)),
                    cast_spell.run_if(
                        in_state(GameState::Gaming).and_then(in_state(GameRunning::Running)),
                    ),
                    move_heros.run_if(
                        in_state(GameState::Gaming).and_then(in_state(GameRunning::Running)),
                    ),
                    update_health_bars
                        .run_if(
                            in_state(GameState::Gaming).and_then(in_state(GameRunning::Running)),
                        )
                        .after(move_heros),
                    move_camera.run_if(
                        in_state(GameState::Gaming).and_then(in_state(GameRunning::Running)),
                    ),
                    wait_to_go_back.run_if(
                        in_state(GameState::Gaming).and_then(in_state(GameRunning::AfterEnd)),
                    ),
                ),
            )
            .add_systems(PostUpdate, register_win.run_if(in_state(GameState::Gaming)))
            .add_systems(OnExit(GameState::Gaming), despawn_screen::<GameWindow>);
    }
}

/// Annotate everything specific to the game window with this component
#[derive(Component)]
pub struct GameWindow;

#[derive(Component, Resource, Default, Debug, Clone, Copy, PartialEq)]
pub enum Spell {
    #[default]
    None,
    FireWall,
    HealthBoost,
    WindGust,
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum GameRunning {
    #[default]
    Running,
    AfterEnd,
}

#[derive(Component, Clone, PartialEq)]
pub struct FireWall {
    pub position: Vec2,
    pub ttl: Timer,
}

#[derive(Component, Clone, Debug, Default)]
pub struct HealingCircle {
    position: Vec2,
    timer: Timer,
}

#[derive(Component, Clone, Debug, Default)]
pub struct WindGust {
    position: Vec2,
    direction: Vec2,
    timer: Timer,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    mut running_state: ResMut<NextState<GameRunning>>,
    asset_server: Res<AssetServer>,
    scene: Res<LevelScene>,
    selected_spell: ResMut<Spell>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut bgm_query: Query<(&mut BGM, Entity)>,
) {
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

    let resolution = &window.single().resolution;

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(&scene.background_texture),
            transform: Transform {
                scale: Vec3::splat(4.0),
                translation: Vec3 {
                    x: 8.0,
                    y: 8.0,
                    z: 0.1,
                },
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(Vec2::new(resolution.width(), resolution.height())),
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

    if let Ok((mut bgm, entity)) = bgm_query.get_single_mut() {
        let music = format!("music/{}", scene.music);
        if bgm.0 != music {
            commands
                .entity(entity)
                .remove::<AudioSink>()
                .insert(asset_server.load::<AudioSource>(&music));
            bgm.0 = music;
        }
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    ..default()
                },
                ..default()
            },
            GameWindow,
        ))
        .with_children(|parent| {
            let spells = [Spell::FireWall, Spell::HealthBoost, Spell::WindGust];
            for spell in spells {
                parent.spawn((
                    ButtonBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
                            min_width: Val::Vw(6.0),
                            min_height: Val::Vw(6.0),
                            display: Display::Flex,
                            border: UiRect::all(Val::Px(5.0)),
                            margin: UiRect::all(Val::Px(3.0)),
                            ..default()
                        },
                        image: UiImage::new(asset_server.load(match spell {
                            Spell::FireWall => "FireSpell.png",
                            Spell::HealthBoost => "HealingSpell.png",
                            Spell::WindGust => "AirSpell.png",
                            Spell::None => unreachable!(),
                        })),
                        ..default()
                    },
                    spell,
                ));
            }
        });
    for (position, tile) in scene.points_of_interest.iter() {
        make_tile(*tile, *position, &mut commands, &asset_server)
    }

    *selected_spell.into_inner() = Spell::None;
    running_state.set(GameRunning::Running);
}

fn select_spell_button(query: Query<(&Interaction, &Spell)>, mut selected_spell: ResMut<Spell>) {
    for (interaction, &spell) in query.iter() {
        if *interaction == Interaction::Pressed {
            *selected_spell = spell;
        }
    }
}

fn select_spell_keybind(input: Res<ButtonInput<KeyCode>>, mut selected_spell: ResMut<Spell>) {
    let mut select_spell = |spell| {
        *selected_spell = if selected_spell.as_ref() == &spell {
            Spell::None
        } else {
            spell
        }
    };
    if input.any_just_pressed([KeyCode::Digit1, KeyCode::Numpad1]) {
        select_spell(Spell::FireWall);
    } else if input.any_just_pressed([KeyCode::Digit2, KeyCode::Numpad2]) {
        select_spell(Spell::HealthBoost);
    } else if input.any_just_pressed([KeyCode::Digit3, KeyCode::Numpad3]) {
        select_spell(Spell::WindGust);
    }
}

const BORDER_HIGHLIGHT: BorderColor = BorderColor(Color::ORANGE_RED);
const BORDER_NOT_HIGHLIGHT: BorderColor = BorderColor(Color::WHITE);

fn highlight_selected_spell(
    selected_spell: Res<Spell>,
    mut query: Query<(&Spell, &mut BorderColor)>,
) {
    if selected_spell.is_changed() {
        for (spell, border) in query.iter_mut() {
            *border.into_inner() = if spell == selected_spell.as_ref() {
                BORDER_HIGHLIGHT
            } else {
                BORDER_NOT_HIGHLIGHT
            }
        }
    }
}

fn cast_spell(
    mut commands: Commands,
    selected_spell: Res<Spell>,
    window: Query<&Window, With<PrimaryWindow>>,
    fire_walls: Query<&FireWall>,
    input: Res<ButtonInput<MouseButton>>,
    asset_server: Res<AssetServer>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    interaction_query: Query<&Interaction>,
    healing_spell: Query<&HealingCircle>,
    mut last_mouse_down: Local<Vec2>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let Some(mouse_position) = window.single().cursor_position() else {
        return;
    };
    let (camera, global_transform) = camera_query.single();
    let Some(ingame_position) = camera.viewport_to_world_2d(global_transform, mouse_position)
    else {
        return;
    };
    let mouse_on_game = interaction_query
        .iter()
        .all(|interaction| *interaction == Interaction::None);
    let pressed = input.pressed(MouseButton::Left) && mouse_on_game;
    if input.just_pressed(MouseButton::Left) && mouse_on_game {
        *last_mouse_down = ingame_position
    };
    match selected_spell.as_ref() {
        Spell::None => {}
        Spell::FireWall => {
            if pressed
                && !fire_walls
                    .iter()
                    .any(|wall| wall.position.distance(ingame_position) < 40.0)
            {
                let layout_not_fr =
                    TextureAtlasLayout::from_grid(Vec2::new(16.0, 32.0), 10, 1, None, None);
                let layout = texture_atlas_layouts.add(layout_not_fr);
                commands.spawn((
                    SpriteSheetBundle {
                        transform: Transform {
                            translation: ingame_position.extend(2.0),
                            scale: Vec3::splat(4.0),
                            ..default()
                        },
                        texture: asset_server.load("FireWall.png"),
                        atlas: TextureAtlas { layout, index: 0 },
                        ..default()
                    },
                    FireWall {
                        position: ingame_position,
                        ttl: Timer::from_seconds(5.0, TimerMode::Once),
                    },
                    AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
                    GameWindow,
                ));
            }
        }
        Spell::HealthBoost => {
            if healing_spell.is_empty() && pressed {
                let layout_not_fr =
                    TextureAtlasLayout::from_grid(Vec2::new(32.0, 32.0), 14, 1, None, None);
                let layout = texture_atlas_layouts.add(layout_not_fr);
                let healing_duration = Duration::from_secs(4);

                commands.spawn((
                    SpriteSheetBundle {
                        transform: Transform {
                            translation: ingame_position.extend(0.6),
                            scale: Vec3::new(4.0, 4.0, 1.0),
                            ..default()
                        },
                        texture: asset_server.load("HealingCircle.png"),
                        atlas: TextureAtlas { layout, index: 0 },
                        ..default()
                    },
                    HealingCircle {
                        position: ingame_position,
                        timer: Timer::new(healing_duration, TimerMode::Once),
                    },
                    AnimationTimer(Timer::new(
                        healing_duration.mul_f32(1.0 / 14.0),
                        TimerMode::Repeating,
                    )),
                    GameWindow,
                ));
            }
        }
        Spell::WindGust => {
            if input.just_released(MouseButton::Left) && mouse_on_game {
                let layout_not_fr =
                    TextureAtlasLayout::from_grid(Vec2::new(16.0, 32.0), 21, 1, None, None);
                let layout = texture_atlas_layouts.add(layout_not_fr);
                let gust_duration = Duration::from_secs(2);

                let direction = *last_mouse_down - ingame_position;

                commands.spawn((
                    SpriteSheetBundle {
                        transform: Transform {
                            translation: last_mouse_down.extend(3.0),
                            scale: Vec3::new(4.0, 4.0, 1.0),
                            rotation: Quat::from_rotation_z(direction.y.atan2(direction.x) + FRAC_PI_2),
                        },
                        sprite: Sprite { anchor: Anchor::BottomCenter, ..default() },
                        texture: asset_server.load("Gust.png"),
                        atlas: TextureAtlas { layout, index: 0 },
                        ..default()
                    },
                    WindGust {
                        position: *last_mouse_down,
                        direction,
                        timer: Timer::new(gust_duration, TimerMode::Once),
                    },
                    AnimationTimer(Timer::new(
                        gust_duration.mul_f32(1.0 / 21.0),
                        TimerMode::Repeating,
                    )),
                    GameWindow,
                ));
            }
        }
    };
}

fn animate_and_despawn_fire(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut FireWall,
        &mut TextureAtlas,
        &mut AnimationTimer,
    )>,
    time: Res<Time>,
) {
    for (entity, mut firewall, mut atlas, mut animation) in query.iter_mut() {
        firewall.ttl.tick(time.delta());
        if firewall.ttl.finished() {
            commands.entity(entity).despawn_recursive();
        }

        const ANIMATION_SPEED: f32 = 1.0;
        animation.tick(time.delta().mul_f32(ANIMATION_SPEED));
        if animation.just_finished() {
            atlas.index = if atlas.index == 9 { 4 } else { atlas.index + 1 }
        }
    }
}

fn animate_and_despawn_healing(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut HealingCircle,
        &mut TextureAtlas,
        &mut AnimationTimer,
    )>,
    time: Res<Time>,
) {
    for (entity, mut healing_circle, mut atlas, mut animation) in query.iter_mut() {
        healing_circle.timer.tick(time.delta());
        if healing_circle.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }

        animation.tick(time.delta());
        if animation.just_finished() {
            atlas.index += 1;
        }
    }
}

fn animate_and_despawn_gust(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut WindGust,
        &mut TextureAtlas,
        &mut AnimationTimer,
    )>,
    time: Res<Time>,
) {
    for (entity, mut gust, mut atlas, mut animation) in query.iter_mut() {
        gust.timer.tick(time.delta());
        if gust.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }

        animation.tick(time.delta());
        if animation.just_finished() {
            atlas.index += 1;
        }
    }
}

fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut event_reader: EventReader<CursorMoved>,
    mut scroll_event: EventReader<MouseWheel>,
    interaction_query: Query<&Interaction>,
    input: Res<ButtonInput<MouseButton>>,
) {
    for event in event_reader.read() {
        if !interaction_query
            .iter()
            .all(|interaction| *interaction == Interaction::None)
        {
            break;
        }
        if input.pressed(MouseButton::Right) {
            let mut transform = camera_query.single_mut();
            // the constant is just a random value that felt right
            let zoom_factor = transform.scale.x;
            let delta = event.delta.unwrap_or_default();
            transform.translation += Vec3::new(-delta.x, delta.y, 0.0) * zoom_factor;
        }
    }
    for event in scroll_event.read() {
        if !interaction_query
            .iter()
            .all(|interaction| *interaction == Interaction::None)
        {
            break;
        }
        let mut transform = camera_query.single_mut();
        const SCROLL_SPEED: f32 = 0.1;
        transform.scale = (transform.scale.xy() - (event.x + event.y) * SCROLL_SPEED)
            .max(Vec2::splat(0.3))
            .extend(1.0);
    }
}

#[derive(Component)]
struct BackToMenuButton;
#[derive(Component)]
struct RetryButton;

fn register_win(
    mut commands: Commands,
    mut event_reader: EventReader<EndGameEvent>,
    level: Res<LevelScene>,
    mut levels_won: ResMut<LevelsWon>,
    mut state: ResMut<NextState<GameRunning>>,
) {
    for event in event_reader.read() {
        let style = Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            justify_self: JustifySelf::Center,
            ..default()
        };
        if let EndGameEvent::Win = event {
            levels_won[level.level - 1] = true;
            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        "You Win!",
                        TextStyle {
                            font_size: 100.0,
                            color: Color::GREEN,
                            ..default()
                        },
                    ),
                    style,
                    ..default()
                },
                GameWindow,
            ));
        } else {
            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        "You Lose!",
                        TextStyle {
                            font_size: 100.0,
                            color: Color::RED,
                            ..default()
                        },
                    )
                    .with_justify(JustifyText::Center),
                    style,
                    ..default()
                },
                GameWindow,
            ));
        }
        commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Percent(60.0),
                        justify_self: JustifySelf::Center,
                        ..default()
                    },
                    background_color: Color::FUCHSIA.into(),
                    ..default()
                },
                BackToMenuButton,
                GameWindow,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        "Go Back To Menu",
                        TextStyle {
                            font_size: 60.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_justify(JustifyText::Center),
                    ..default()
                });
            });
        commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Percent(80.0),
                        justify_self: JustifySelf::Center,
                        ..default()
                    },
                    background_color: Color::FUCHSIA.into(),
                    ..default()
                },
                RetryButton,
                GameWindow,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        "Retry",
                        TextStyle {
                            font_size: 60.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_justify(JustifyText::Center),
                    ..default()
                });
            });
        state.set(GameRunning::AfterEnd);
    }
}

fn wait_to_go_back(
    mut commands: Commands,
    menu: Query<&Interaction, With<BackToMenuButton>>,
    retry: Query<&Interaction, With<RetryButton>>,
    level: Res<LevelScene>,
    mut state: ResMut<NextState<GameState>>,
) {
    for &interaction in menu.iter() {
        if interaction == Interaction::Pressed {
            state.set(GameState::LevelSelect);
        }
    }
    for &interaction in retry.iter() {
        if interaction == Interaction::Pressed {
            // janky but it works
            commands.spawn((ReenterLevel(level.level), LevelSelectWindow));
            state.set(GameState::LevelSelect);
        }
    }
}
