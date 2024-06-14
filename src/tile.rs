use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Serialize, Deserialize)]
pub enum Tile {
    #[default]
    Ground,
    Grass,
    Spike,
}
