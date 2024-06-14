use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct Hero {
    pub sprite_bundle: SpriteBundle,
    pub target: Vec3,
    pub speed: f32,
    pub finished: bool,
}

pub fn move_heros(time: Res<Time>, mut heros: Query<&mut Hero>) {
    for mut hero in heros.iter_mut() {
        if hero.finished {
            // Skip
            continue;
        }
        let direction = hero.target - hero.sprite_bundle.transform.translation;
        if direction.length().abs() < 1.0 {
            hero.finished = true
        } else {
            let target = hero.target;
            hero.sprite_bundle.transform.translation +=
                direction.normalize() * time.delta_seconds() * target;
        }
    }
}
