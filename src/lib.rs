pub mod controller;

use crate::controller::PlayerInfo;
use bevy::prelude::*;
use rand::random;
use std::collections::BTreeMap;
use std::fmt::Formatter;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SnakeComponent;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SnakeHead;
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SnakeSegment(pub i32);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Food;

pub struct Materials {
    pub colors: Vec<Color>,
    pub head_material: Vec<Handle<ColorMaterial>>,
    pub segment_material: Handle<ColorMaterial>,
    pub food_material: Handle<ColorMaterial>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PlayerId(pub i32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position(pub Vec2);
impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({},{})", self.0.x, self.0.y))
    }
}
impl Position {
    pub fn random(limit_x: f32, limit_y: f32) -> Self {
        Self(Vec2::new(
            (random::<f32>() - 0.5) * limit_x,
            (random::<f32>() - 0.5) * limit_y,
        ))
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Velocity(pub Vec2);
impl Velocity {
    pub fn random(abs: f32) -> Self {
        let angle = std::f32::consts::PI * 2.0 * random::<f32>();
        Self(Vec2::new(abs.clone() * angle.cos(), abs * angle.sin()))
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Radius(pub f32);

pub const CONST_SPEED: f32 = 5.0;
pub const GRID_SIZE: f32 = 10.0;
pub const TICK: f32 = 1.0 / 60.0;
pub const ARENA_WIDTH: f32 = 100.0;
pub const ARENA_HEIGHT: f32 = 100.0;

pub fn spawn_snake_head(
    commands: &mut Commands,
    player: PlayerId,
    pos: Position,
    vel: Velocity,
    materials: &Materials,
) -> Entity {
    let vel1 = vel.0.normalize();
    let rotation = Quat::from_rotation_arc(Vec3::X, Vec3::new(vel1.x.clone(), vel1.y.clone(), 0.0));
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.head_material[player.0.clone() as usize].clone(),
            sprite: Sprite::new(Vec2::new(GRID_SIZE, GRID_SIZE)),
            transform: Transform {
                translation: Vec3::new(pos.0.x.clone(), pos.0.y.clone(), 0.1),
                rotation,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(player)
        .insert(SnakeHead)
        .insert(SnakeComponent)
        .insert(vel)
        .insert(Radius(GRID_SIZE / 2.0))
        .id()
}
pub fn spawn_snake_segment(
    commands: &mut Commands,
    seg_num: i32,
    player: PlayerId,
    pos: Position,
    materials: &Materials,
) -> Entity {
    let transform = Transform::from_xyz(pos.0.x.clone(), pos.0.y.clone(), 0.0);
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.segment_material.clone(),
            sprite: Sprite::new(Vec2::new(GRID_SIZE, GRID_SIZE)),
            transform,
            ..Default::default()
        })
        .insert(SnakeComponent)
        .insert(SnakeSegment(seg_num))
        .insert(Radius(GRID_SIZE / 2.0))
        .insert(player)
        .id()
}
pub fn spawn_snake_with_nodes(
    commands: &mut Commands,
    player: PlayerId,
    pos: Position,
    vel: Velocity,
    nodes: i32,
    materials: &Materials,
) -> Entity {
    let head = spawn_snake_head(commands, player, pos, vel, materials);
    for i in 1..=nodes {
        spawn_snake_segment(commands, i, player, pos, materials);
    }
    head
}
pub fn spawn_food(commands: &mut Commands, pos: Position, materials: &Materials) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.food_material.clone(),
            sprite: Sprite::new(Vec2::new(GRID_SIZE / 3.0, GRID_SIZE / 3.0)),
            transform: Transform::from_xyz(pos.0.x.clone(), pos.0.y.clone(), 0.0),
            ..Default::default()
        })
        .insert(Food)
        .insert(Radius(GRID_SIZE / 6.0))
        .id()
}
#[derive(Default)]
pub struct SnakeNode<Trans> {
    pub seg_id: i32,
    pub trans: Trans,
    pub entity: Option<Entity>,
}

pub struct SnakeBody<T> {
    pub player_id: PlayerId,
    pub player_info: Option<PlayerInfo>,
    pub head_speed: Option<Velocity>,
    pub head_radius: Option<Radius>,
    pub body: BTreeMap<i32, SnakeNode<T>>,
}
impl<T> Default for SnakeBody<T> {
    fn default() -> Self {
        Self {
            player_id: PlayerId(-1),
            player_info: None,
            head_speed: None,
            head_radius: None,
            body: Default::default(),
        }
    }
}

pub struct FoodBody {
    pub pos: Position,
}
#[derive(Default)]
pub struct SnakeWorld<'a> {
    pub foods: Vec<FoodBody>,
    pub snakes: BTreeMap<PlayerId, SnakeBody<&'a Transform>>,
}
