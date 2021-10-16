use bevy::prelude::*;
use rand::random;
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SnakeComponent;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SnakeHead;
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SnakeSegment(pub i32);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Food;

pub struct Materials {
    pub head_material: Handle<ColorMaterial>,
    pub food_material: Handle<ColorMaterial>,
    pub segment_material: Handle<ColorMaterial>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PlayerId(pub i32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position(pub Vec2);
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Velocity(pub Vec2);

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
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(GRID_SIZE, GRID_SIZE)),
            transform: Transform::from_xyz(pos.0.x.clone(), pos.0.y.clone(), 0.1),
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

pub fn spawn_food(commands: &mut Commands, materials: &Materials) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.food_material.clone(),
            sprite: Sprite::new(Vec2::new(GRID_SIZE / 3.0, GRID_SIZE / 3.0)),
            transform: Transform::from_xyz(
                (random::<f32>() - 0.5) * ARENA_WIDTH,
                (random::<f32>() - 0.5) * ARENA_HEIGHT,
                0.0,
            ),
            ..Default::default()
        })
        .insert(Food)
        .insert(Radius(GRID_SIZE / 6.0))
        .id()
}
