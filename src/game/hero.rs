use std::collections::HashSet;
use std::f32::consts::FRAC_PI_2;
use std::{fmt, time::Duration};

use bevy::{prelude::*, sprite::Anchor};
use rand::prelude::random;
use serde::{Deserialize, Serialize};

use super::{AnimationTimer, FireWall, GameWindow};
use crate::tile::{world_to_grid, Tile};
use crate::{EndGameEvent, LevelScene};

#[derive(Default, Debug, Clone, Component, Serialize, Deserialize)]
pub struct Hero {
    pub targets: Vec<Vec2>,
    pub position: Vec2,
    pub speed: f32,
    pub hero_type: HeroType,
    #[serde(skip)]
    pub current_target: usize,
    #[serde(flatten)]
    pub health_bar: HealthBar,
    #[serde(skip)]
    pub rand: u8,
    #[serde(skip)]
    pub seen_poi: HashSet<IVec2>,
}

impl Hero {
    pub fn target(&self) -> Vec2 {
        self.targets[self.current_target]
    }
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
    mut scene: ResMut<LevelScene>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for hero in scene.heros.iter_mut() {
        // Texture
        let texture = asset_server.load(match hero.hero_type {
            HeroType::JohnHeron => "JohnHeron.png",
            HeroType::RerinGuard => "RerinGuard.png",
        });
        let layout_not_fr = TextureAtlasLayout::from_grid(Vec2::splat(16.0), 4, 1, None, None);
        let layout = texture_atlas_layouts.add(layout_not_fr);
        hero.rand = random();

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
                hero.clone(),
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
                                    translation: Vec3::new(-0.5, 0.0, 0.1),
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
    fires: Query<&FireWall>,
    scene: Res<LevelScene>,
    mut event_writer: EventWriter<EndGameEvent>,
) {
    for (mut hero, mut transform, mut atlas, mut sprite, mut timer) in query.iter_mut() {
        let direction = hero.target() - hero.position;

        // Animation
        const ANIMATION_SPEED: f32 = 0.01;
        timer.tick(time.delta().mul_f32(hero.speed * ANIMATION_SPEED));

        if timer.just_finished() {
            atlas.index = if atlas.index == 3 { 0 } else { atlas.index + 1 }
        }

        let clostest_fire = fires.iter().min_by(|x, y| {
            x.position
                .distance(hero.position)
                .total_cmp(&y.position.distance(hero.position))
        });
        let direction = match clostest_fire {
            Some(fire) => {
                let distance = fire.position - hero.position;
                let distance_len = distance.length();
                if distance_len <= 55.0 {
                    hero.health_bar.current_health -=
                        (55.0 - distance.length()) * time.delta_seconds();
                }
                if distance_len <= 60.0 {
                    -distance
                } else if distance_len <= 70.0 {
                    let new_dir_a = Vec2 {
                        x: distance.y,
                        y: -distance.x,
                    };
                    let new_dir_b = Vec2 {
                        x: -distance.y,
                        y: distance.x,
                    };
                    let angle_a = new_dir_a.angle_between(direction).abs();
                    let angle_b = new_dir_b.angle_between(direction).abs();
                    if angle_a < FRAC_PI_2 || angle_b < FRAC_PI_2 {
                        if hero.rand <= 128 {
                            new_dir_a.normalize() * direction.length()
                        } else {
                            new_dir_b.normalize() * direction.length()
                        }
                    } else {
                        direction
                    }
                } else {
                    if distance_len >= 100.0 {
                        hero.rand = random()
                    }
                    direction
                }
            }
            None => direction,
        };

        // POI
        let grid_pos = world_to_grid(hero.position);
        if !hero.seen_poi.contains(&grid_pos) {
            if let Some(tile) = scene.points_of_interest_map.get(&grid_pos) {
                match *tile {
                    Tile::Ground => {}
                    Tile::Grass => {}
                    Tile::Spike => hero.health_bar.current_health -= 40.0,
                };
                hero.seen_poi.insert(grid_pos);
            }
        }

        // Flip sprite if we go to the right
        sprite.flip_x = direction.x.is_sign_negative();

        // Finish when close to target
        if direction.length() < hero.speed * time.delta_seconds() {
            hero.position = hero.target();
            hero.current_target += 1;
            if hero.current_target == hero.targets.len() {
                event_writer.send(EndGameEvent::Win);
            }
        } else {
            // Movement
            let speed = hero.speed;
            hero.position += direction.normalize() * speed * time.delta_seconds();
        }
        transform.translation = hero.position.extend(1.0);

        if hero.health_bar.current_health <= 0.0 {
            event_writer.send(EndGameEvent::Loss);
        }
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
