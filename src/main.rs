use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::system::exit_on_esc_system;
use bevy::log::*;
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::time::Duration;
use the_snakes::{
    spawn_food, spawn_snake, Food, Materials, PlayerId, Position, SnakeHead, SnakeSegment,
    Velocity, CONST_SPEED, TICK,
};
pub struct SnakeMoveTimer(pub Timer);
impl Default for SnakeMoveTimer {
    fn default() -> Self {
        Self(Timer::new(
            Duration::from_millis((TICK * 1000.0) as _),
            true,
        ))
    }
}

pub struct FoodSpawnTimer(pub Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}
fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)));
    commands.insert_resource(WindowDescriptor {
        title: "Snake!".to_string(),
        width: 640.0,
        height: 640.0,
        ..Default::default()
    });
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        food_material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
        segment_material: materials.add(Color::rgb(0.4, 0.4, 0.4).into()),
    });
}

fn setup_game(mut commands: Commands, materials: Res<Materials>) {
    spawn_snake(
        &mut commands,
        PlayerId(0),
        Position(Vec2::new(0.0, 0.0)),
        Velocity(Vec2::Y * CONST_SPEED),
        &materials,
    );
}

fn snake_move(
    mut snake_components: Query<(
        &mut Transform,
        &PlayerId,
        Option<&Velocity>,
        Option<&SnakeHead>,
        Option<&SnakeSegment>,
    )>,
    time: Res<Time>,
    mut timer: Local<SnakeMoveTimer>,
) {
    if timer.0.tick(time.delta()).finished() {
        #[derive(Default)]
        struct SnakeBody<'a> {
            head_speed: Option<&'a Velocity>,
            body: Vec<(i32, Mut<'a, Transform>)>,
        }
        let mut snakes: HashMap<PlayerId, SnakeBody> = Default::default();
        for (trans, player, vel, head, segment) in snake_components.iter_mut() {
            let snake = snakes.entry(*player).or_default();
            if head.is_some() {
                snake.body.push((-1, trans));
                snake.head_speed = Some(vel.unwrap());
            } else if let Some(seg) = segment {
                snake.body.push((seg.0, trans));
            } else {
                unreachable!()
            }
        }

        for (_id, mut snake) in snakes {
            let head_vel = snake.head_speed.unwrap();
            snake.body.sort_by_key(|x| x.0);
            for i in (1..snake.body.len()).rev() {
                let trans =
                    snake.body[i].1.translation * 0.9 + snake.body[i - 1].1.translation * 0.1;

                snake.body[i].1.translation = trans;
                let rot = snake.body[i].1.rotation * 0.9 + snake.body[i - 1].1.rotation * 0.1;
                snake.body[i].1.rotation = rot;
            }
            snake.body[0].1.translation +=
                CONST_SPEED * TICK * Vec3::new(head_vel.0.x.clone(), head_vel.0.y.clone(), 0.0);
        }
    }
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    if timer.0.tick(time.delta()).finished() {
        spawn_food(&mut commands, &materials)
    }
}

fn rotate((x, y): (f32, f32), theta: f32) -> Vec2 {
    let (x2, y2) = (theta.cos(), theta.sin());
    Vec2::new(
        x.clone() * x2.clone() - y.clone() * y2.clone(),
        x * y2 + y * x2,
    )
}
fn turn_from_keyboard(
    keys: Res<Input<KeyCode>>,
    mut q: Query<(&mut Velocity, &mut Transform, &PlayerId), With<SnakeHead>>,
) {
    const OMEGA: f32 = 2.0 * std::f32::consts::PI;
    const THETA: f32 = OMEGA * TICK;
    if keys.pressed(KeyCode::Left) {
        for (mut vel, mut trans, player_id) in q.iter_mut() {
            if player_id.0 == 0 {
                *vel = Velocity(rotate((vel.0.x, vel.0.y), THETA));
                trans.rotate(Quat::from_rotation_z(THETA));
            }
        }
    }
    if keys.pressed(KeyCode::Right) {
        for (mut vel, mut trans, player_id) in q.iter_mut() {
            if player_id.0 == 0 {
                *vel = Velocity(rotate((vel.0.x, vel.0.y), -THETA));
                trans.rotate(Quat::from_rotation_z(-THETA));
            }
        }
    }
}

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("setup_game", SystemStage::single(setup_game.system()))
        .add_system(exit_on_esc_system.system())
        .add_system(food_spawner.system())
        .add_system(snake_move.system())
        .add_system(turn_from_keyboard.system())
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .run();
}
