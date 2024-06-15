use bevy::{prelude::*, sprite::Anchor};
use serde::{Deserialize, Serialize};
use std::{fmt, time::Duration};

use crate::LevelScene;

use super::{AnimationTimer, GameWindow};

#[derive(Default, Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct Hero {
    pub target: Vec2,
    pub position: Vec2,
    pub speed: f32,
    pub hero_type: HeroType,
    #[serde(flatten)]
    pub health_bar: HealthBar,
}

#[derive(Default, Debug, Clone, Copy, Serialize)]
pub struct HealthBar {
    pub max_health: f32,
    #[serde(skip_serializing)]
    pub current_health: f32,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HeroType {
    #[default]
    JohnHeron,
    RerinGuard,
}

#[derive(Component)]
pub struct HealthBarComponent;

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
        commands
            .spawn((
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
                GameWindow,
            ))
            .with_children(|parent| {
                // healthbar
                parent
                    .spawn((
                        SpriteBundle {
                            transform: Transform {
                                translation: Vec3::new(0.0, 10.0, 5.0),
                                scale: Vec3::new(15.0, 2.0, 1.0),
                                ..default()
                            },
                            sprite: Sprite {
                                color: Color::RED,
                                custom_size: Some(Vec2::ONE),
                                ..default()
                            },
                            ..default()
                        },
                        HealthBarComponent,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            SpriteBundle {
                                sprite: Sprite {
                                    color: Color::FUCHSIA,
                                    anchor: Anchor::CenterLeft,
                                    ..default()
                                },
                                transform: Transform {
                                    scale: Vec3::ONE,
                                    translation: Vec3::new(-0.5, 0.0, 0.0),
                                    ..default()
                                },
                                ..default()
                            },
                            HealthBarComponent,
                        ));
                    });
            });
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

pub fn update_health_bars(
    query: Query<(&Children, &Parent), With<HealthBarComponent>>,
    mut healthbars: Query<&mut Transform, With<HealthBarComponent>>,
    heros: Query<&Hero>,
) {
    for (hp, hero) in query.iter() {
        // idk if there is a better way to get the hero and hp at the same time
        let Some(&hp) = hp.iter().next() else {
            continue;
        };
        let Ok(mut hp) = healthbars.get_mut(hp) else {
            continue;
        };
        let Ok(hero) = heros.get(hero.get()) else {
            continue;
        };
        let hp_ratio = hero.health_bar.current_health / hero.health_bar.max_health;
        hp.scale.x = hp_ratio.max(0.0);
    }
}

// chat gpts fever dream to get deserializing working
impl<'de> Deserialize<'de> for HealthBar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            MaxHealth,
        }
        struct HealthBarVisitor;
        impl<'de> serde::de::Visitor<'de> for HealthBarVisitor {
            type Value = HealthBar;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct HealthBar")
            }
            fn visit_map<V>(self, mut map: V) -> Result<HealthBar, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut max_health = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::MaxHealth => {
                            if max_health.is_some() {
                                return Err(serde::de::Error::duplicate_field("max_health"));
                            }
                            max_health = Some(map.next_value()?);
                        }
                    }
                }
                let max_health =
                    max_health.ok_or_else(|| serde::de::Error::missing_field("max_health"))?;
                Ok(HealthBar {
                    max_health,
                    current_health: max_health,
                })
            }
        }
        const FIELDS: &[&str] = &["max_health"];
        deserializer.deserialize_struct("HealthBar", FIELDS, HealthBarVisitor)
    }
}
