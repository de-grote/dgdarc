use bevy::prelude::*;

use crate::{despawn_screen, GameState, BGM};

pub struct InfoPlugin;

impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InfoScreen), setup)
            .add_systems(
                Update,
                back_button_pressed.run_if(in_state(GameState::InfoScreen)),
            )
            .add_systems(OnExit(GameState::InfoScreen), despawn_screen::<InfoWindow>);
    }
}

#[derive(Component)]
pub struct InfoWindow;

#[derive(Component)]
struct BackToMainButton;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut bgm_query: Query<(&mut BGM, Entity)>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::LIME_GREEN),
                ..default()
            },
            ..default()
        },
        InfoWindow,
    ));

    if let Ok((mut bgm, entity)) = bgm_query.get_single_mut() {
        if bgm.0 != "music/Main_menu.ogg" {
            commands
                .entity(entity)
                .remove::<AudioSink>()
                .insert(asset_server.load::<AudioSource>("music/Main_menu.ogg"));
            bgm.0 = "music/Main_menu.ogg".to_string();
        }
    }

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
            InfoWindow,
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

    commands.spawn((
        TextBundle::from_section(
            include_str!("info.txt"),
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Percent(5.0)),
            ..default()
        }),
        InfoWindow,
    ));
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
