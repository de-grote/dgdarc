use bevy::{prelude::*, window::PrimaryWindow};

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
                    highligh_selected_spell.run_if(in_state(GameState::Gaming)),
                    move_heros.run_if(in_state(GameState::Gaming)),
                ),
            )
            .add_systems(OnExit(GameState::Gaming), despawn_screen::<GameWindow>);
    }
}

/// Annotate everything specific to the game window with this component
#[derive(Component)]
struct GameWindow;

#[derive(Component, Resource, Default, Debug, Clone, Copy, PartialEq)]
pub enum Spell {
    #[default]
    FireWall,
    HealthBoost,
}

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
                        })),
                        ..default()
                    },
                    spell,
                ));
            }
        });

    *selected_spell.into_inner() = Spell::FireWall;
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
        *selected_spell = Spell::FireWall;
    } else if input.any_just_pressed([KeyCode::Digit2, KeyCode::Numpad2]) {
        *selected_spell = Spell::HealthBoost;
    }
}

const BORDER_HIGHLIGHT: BorderColor = BorderColor(Color::ORANGE_RED);
const BORDER_NOT_HIGHLIGHT: BorderColor = BorderColor(Color::WHITE);

fn highligh_selected_spell(
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
