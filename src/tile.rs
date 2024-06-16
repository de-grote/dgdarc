use crate::game::GameWindow;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Serialize, Deserialize, Component, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Tile {
    #[default]
    Ground,
    Grass,
    Spike,
    Hole,
    Pole,
}

pub fn make_tile(tile: Tile, position: IVec2, commands: &mut Commands, asset_server: &AssetServer) {
    let texture: Handle<Image> = asset_server.load(match tile {
        Tile::Grass => "EvilGrass.png",
        Tile::Spike => "Spikes.png",
        Tile::Hole => "Pit.png",
        Tile::Pole => "WoodPole.png",
        _ => "test.png",
    });
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform {
                translation: Vec3::from((grid_to_world(position), 0.2)),
                scale: Vec3::splat(4.0),
                ..default()
            },
            ..default()
        },
        tile,
        GameWindow,
    ));
}
pub fn grid_tile(position: Vec2, grid: Vec<Vec<Tile>>) -> Option<Tile> {
    let position = world_to_grid(position);
    Some(grid[position.x as usize][position.y as usize])
}

pub fn world_to_grid(position: Vec2) -> IVec2 {
    let grid_pos = position / 64.0;
    IVec2 {
        x: grid_pos.x.round() as i32,
        y: grid_pos.y.round() as i32,
    }
}

pub fn grid_to_world(position: IVec2) -> Vec2 {
    let grid_pos = position * 64;
    Vec2 {
        x: grid_pos.x as f32,
        y: grid_pos.y as f32,
    }
}
