use std::time::Duration;

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::{prelude::*, window::PrimaryWindow};

use crate::tile::make_tile;
use crate::{despawn_screen, GameState, LevelScene};
use hero::*;

pub mod hero;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Spell>()
            .add_systems(OnEnter(GameState::Gaming), (setup, create_hero))
            .add_systems(
                Update,
                (
                    select_spell_button.run_if(in_state(GameState::Gaming)),
                    select_spell_keybind.run_if(in_state(GameState::Gaming)),
                    highlight_selected_spell.run_if(in_state(GameState::Gaming)),
                    animate_and_despawn_fire.run_if(in_state(GameState::Gaming)),
                    cast_spell.run_if(in_state(GameState::Gaming)),
                    move_heros.run_if(in_state(GameState::Gaming)),
                    update_health_bars.run_if(in_state(GameState::Gaming)),
                    move_camera.run_if(in_state(GameState::Gaming)),
                ),
            )
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
}

#[derive(Component, Clone, PartialEq)]
pub struct FireWall {
    pub position: Vec2,
    pub ttl: Timer,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    scene: Res<LevelScene>,
    selected_spell: ResMut<Spell>,
    window: Query<&Window, With<PrimaryWindow>>,
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
            let spells = [Spell::FireWall, Spell::HealthBoost];
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
}

fn select_spell_button(query: Query<(&Interaction, &Spell)>, mut selected_spell: ResMut<Spell>) {
    for (interaction, &spell) in query.iter() {
        if *interaction == Interaction::Pressed {
            *selected_spell = spell;
        }
    }
}

fn select_spell_keybind(input: Res<ButtonInput<KeyCode>>, mut selected_spell: ResMut<Spell>) {
    if input.any_just_pressed([KeyCode::Digit1, KeyCode::Numpad1]) {
        *selected_spell = if selected_spell.as_ref() == &Spell::FireWall {
            Spell::None
        } else {
            Spell::FireWall
        }
    } else if input.any_just_pressed([KeyCode::Digit2, KeyCode::Numpad2]) {
        *selected_spell = if selected_spell.as_ref() == &Spell::HealthBoost {
            Spell::None
        } else {
            Spell::HealthBoost
        }
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
    let pressed = || {
        input.pressed(MouseButton::Left)
            && !interaction_query
                .iter()
                .any(|interaction| *interaction == Interaction::Pressed)
    };
    match selected_spell.as_ref() {
        Spell::None => {}
        Spell::FireWall => {
            if pressed()
                && !fire_walls
                    .iter()
                    .any(|wall| wall.position.distance(ingame_position) < 40.0)
            {
                let layout_not_fr =
                    TextureAtlasLayout::from_grid(Vec2::splat(16.0), 6, 1, None, None);
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
        Spell::HealthBoost => {}
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
            atlas.index = if atlas.index == 5 { 0 } else { atlas.index + 1 }
        }
    }
}

fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut event_reader: EventReader<MouseMotion>,
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
            let zoom_factor = transform.scale.x * 0.6;
            transform.translation += Vec3::new(-event.delta.x, event.delta.y, 0.0) * zoom_factor;
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
