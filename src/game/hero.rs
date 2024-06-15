use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::LevelScene;

use super::AnimationTimer;

#[derive(Default, Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct Hero {
    pub target: Vec2,
    pub position: Vec2,
    pub speed: f32,
    pub hero_type: HeroType,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HeroType {
    #[default]
    JohnHeron,
    RerinGuard,
}



pub fn create_hero(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    scene: Res<LevelScene>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for &hero in scene.heros.iter() {
        // Texture
        let texture = asset_server.load(match hero.hero_type {
            HeroType::JohnHeron => "JohnHeron.png",
            HeroType::RerinGuard => "RerinGuard.png",
        });
        let layout_not_fr = TextureAtlasLayout::from_grid(Vec2::splat(16.0), 4, 1, None, None);
        let layout = texture_atlas_layouts.add(layout_not_fr);

        // Hero
        let timer = AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating));
        // Spawn hero
        commands.spawn((
            SpriteSheetBundle {
                texture,
                transform: Transform {
                    translation: hero.position.extend(1.0),
                    scale: Vec3::splat(4.0),
                    ..default()
                },
                atlas: TextureAtlas { layout, index: 0 },
                ..default()
            },
            hero,
            timer,
        ));
    }
}

pub fn move_heros(
    time: Res<Time>,
    mut query: Query<(
        &mut Hero,
        &mut Transform,
        &mut TextureAtlas,
        &mut Sprite,
        &mut AnimationTimer,
    )>,
) {
    for (mut hero, mut transform, mut atlas, mut sprite, mut timer) in query.iter_mut() {
        let direction = hero.target - hero.position;

        // Flip sprite if we go to the right
        sprite.flip_x = direction.x.is_sign_negative();
        
        // Animation
        const ANIMATION_SPEED: f32 = 0.01;
        timer.tick(time.delta().mul_f32(hero.speed * ANIMATION_SPEED));

        if timer.just_finished() {
            atlas.index = if atlas.index == 3 { 0 } else { atlas.index + 1 }
        }

        // Finish when close to target
        if direction.length() < hero.speed * time.delta_seconds() {
            hero.position = hero.target;
            hero.target = -hero.target;
        } else {
            // Movement
            let speed = hero.speed;
            hero.position += direction.normalize() * speed * time.delta_seconds();
        }
        transform.translation = hero.position.extend(1.0);
    }
}
