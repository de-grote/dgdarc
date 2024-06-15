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
                    move_heros.run_if(in_state(GameState::Gaming)),
                ),
            )
            .add_systems(OnExit(GameState::Gaming), despawn_screen::<GameWindow>);
    }
}

/// Annotate everything specific to the game window with this component
#[derive(Component)]
struct GameWindow;

#[derive(Component, Resource, Default, Debug, Clone, Copy)]
pub enum Spell {
    #[default]
    FireWall,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    scene: Res<LevelScene>,
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
                background_color: Color::LIME_GREEN.into(),
                ..default()
            },
            GameWindow,
        ))
        .with_children(|parent| {
            let spells = [Spell::FireWall];
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
                        })),
                        border_color: Color::RED.into(),
                        ..default()
                    },
                    spell,
                ));
            }
        });
}

fn select_spell_button(
    query: Query<(&Interaction, &Spell)>,
    mut selected_spell: ResMut<Spell>,
) {
    for (interaction, &spell) in query.iter() {
        if *interaction == Interaction::Pressed {
            *selected_spell = spell;
        }
    }
}

fn select_spell_keybind(
    input: Res<ButtonInput<KeyCode>>,
    mut selected_spell: ResMut<Spell>,
) {
    if input.any_just_pressed([KeyCode::Digit0, KeyCode::Numpad0]) {
        *selected_spell = Spell::FireWall;
    }
}
