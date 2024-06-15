use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Serialize, Deserialize, Component)]
pub enum Tile {
    #[default]
    Ground,
    Grass,
    Spike,
}

impl Tile {
    fn make_tile(
        tile: Tile,
        position: Vec3,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        let texture: Handle<Image> = asset_server.load(match tile {
            _ => "test.png",
        });
        commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform {
                    translation: position,
                    ..default()
                },
                ..default()
            },
            tile,
        ));
    }
}
