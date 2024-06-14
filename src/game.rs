use bevy::prelude::*;

use crate::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Gaming),
            setup,
        );
    }
}

fn setup() {
    
}