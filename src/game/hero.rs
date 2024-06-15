use std::time::Duration;
use bevy::prelude::*;

#[derive(Default, Clone, Component)]
pub struct Hero {
    pub target: Vec3,
    pub speed: f32,
    finished: bool,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

pub fn create_hero(
    file: &'static str,
    target: Vec3,
    speed: f32,
    start: Vec3,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Texture
    let texture = asset_server.load(file);
    let layout_not_fr = TextureAtlasLayout::from_grid(Vec2::splat(16.0), 4, 1, None, None);
    let layout = texture_atlas_layouts.add(layout_not_fr);
    //Hero
    let hero = Hero {
        target,
        speed,
        finished: false,
    };
    let timer = AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating));
    // Spawn hero
    commands.spawn((
        SpriteSheetBundle {
            texture,
            transform: Transform {
                translation: start,
                ..default()
            },
            atlas: TextureAtlas { layout, index: 1 },
            ..default()
        },
        hero,
        timer,
    ));
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
        if hero.finished {
            // Skip
            continue;
        }
        let direction = hero.target - transform.translation;

        // Flip sprite if we go to the right
        sprite.flip_x = direction.x < f32::EPSILON;

        // May need a const to not look weird
        timer.tick(time.delta().mul_f32(hero.speed * 0.016666667));
        // Animation
        if timer.just_finished() {
            atlas.index = if atlas.index == 4 { 0 } else { atlas.index + 1 }
        }

        // Finish when close to target
        if direction.length().abs() < 1.0 {
            hero.finished = true
        } else {
            // Movement
            transform.translation += direction.normalize() * time.delta_seconds() * hero.target;
        }
    }
}
