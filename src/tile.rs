use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Serialize, Deserialize, Component, Copy, Clone)]
pub enum Tile {
    #[default]
    Ground,
    Grass,
    Spike,
}

pub fn make_tile(
    tile: Tile,
    position: IVec2,
    commands: &mut Commands,
    asset_server: &mut AssetServer,
) {
    let texture: Handle<Image> = asset_server.load(match tile {
        Tile::Grass => "EvilGrass.png",
        Tile::Spike => "Spikes.png",
        _ => "test.png",
    });
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform {
                translation: Vec3::from((grid_to_world(position), 1.0)),
                ..default()
            },
            ..default()
        },
        tile,
    ));
}
pub fn grid_tile(position: Vec2, grid: Vec<Vec<Tile>>) -> Option<Tile> {
    let position = world_to_grid(position);
    Some(grid[position.x as usize][position.y as usize])
}

pub fn world_to_grid(position: Vec2) -> IVec2 {
    let grid_pos = position/16.0;
    IVec2 { x: grid_pos.x.round() as i32,
        y: grid_pos.y.round() as i32,
    }
}

pub fn grid_to_world(position: IVec2) -> Vec2 {
    let grid_pos = position * 16;
    Vec2 { x: grid_pos.x as f32,
        y: grid_pos.y as f32,
    }
}
