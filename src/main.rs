use anyhow::Context;
use anyhow::Result;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::input::system::exit_on_esc_system;
#[allow(unused_imports)]
use bevy::log::*;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::path::Path;
use std::time::Duration;
use the_snakes::controller::{Controller, MovementCommand, PlayerInfo, StdioController};
use the_snakes::{
    spawn_food, spawn_snake_head, spawn_snake_segment, spawn_snake_with_nodes, Food, FoodBody,
    Materials, PlayerId, Position, Radius, SnakeBody, SnakeHead, SnakeNode, SnakeSegment,
    SnakeWorld, Velocity, ARENA_HEIGHT, ARENA_WIDTH, CONST_SPEED, TICK,
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
    let colors = vec![
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 255, 0),
        (0, 255, 255),
        (255, 0, 255),
        (192, 192, 192),
        (128, 128, 128),
        (128, 0, 0),
        (128, 128, 0),
        (0, 128, 0),
        (128, 0, 128),
        (0, 128, 128),
        (0, 0, 128),
    ];
    let colors = colors
        .into_iter()
        .map(|(r, g, b)| (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
        .map(|(r, g, b)| materials.add(Color::rgb(r, g, b).into()));
    commands.insert_resource(Materials {
        head_material: colors.collect(),
        segment_material: materials.add(Color::rgb(0.4, 0.4, 0.4).into()),
        food_material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
    });
}
#[derive(Default)]
struct AiManager {
    ais: HashMap<PlayerId, Box<dyn Controller>>,
}

impl AiManager {
    fn load_all_ai<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let dir = std::fs::read_dir(path.as_ref()).with_context(|| {
            format!(
                "Could not read {} directory",
                path.as_ref().to_str().unwrap()
            )
        })?;
        let mut player_id = 1;
        for f in dir {
            let entry = f?;
            let controller = StdioController::new(entry.path())?;
            self.ais
                .insert(PlayerId(player_id.clone()), Box::new(controller));
            player_id += 1;
        }
        Ok(())
    }
    fn initialize_all_ai(&mut self, command: &mut Commands, materials: &Materials) -> Result<()> {
        for (k, v) in self.ais.iter_mut() {
            let info: PlayerInfo = v.initialize(*k)?;
            assert_eq!(info.is_ai, true);
            let head = spawn_snake_with_nodes(
                command,
                *k,
                Position::random(ARENA_WIDTH, ARENA_HEIGHT),
                Velocity::random(CONST_SPEED),
                3,
                materials,
            );
            command.entity(head).insert(info);
        }
        Ok(())
    }
}
fn setup_game(
    mut commands: Commands,
    materials: Res<Materials>,
    mut controller: ResMut<AiManager>,
) {
    spawn_snake_with_nodes(
        &mut commands,
        PlayerId(0),
        Position::random(ARENA_WIDTH, ARENA_HEIGHT),
        Velocity::random(CONST_SPEED),
        3,
        &materials,
    );
    match controller.load_all_ai("bin/ai") {
        Ok(()) => {}
        Err(err) => {
            error!("Could not load ai: {:?}", err);
        }
    }
    match controller.initialize_all_ai(&mut commands, &materials) {
        Ok(()) => {}
        Err(err) => {
            error!("Could not initialize ai: {:?}", err);
        }
    }
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
        let mut snakes: HashMap<PlayerId, SnakeBody<Mut<Transform>>> = Default::default();
        for (trans, player, vel, head, segment) in snake_components.iter_mut() {
            let snake = snakes.entry(*player).or_default();
            if head.is_some() {
                snake.body.insert(SnakeNode {
                    seg_id: 0,
                    trans,
                    entity: None,
                });
                snake.head_speed = Some(vel.unwrap());
            } else if let Some(seg) = segment {
                snake.body.insert(SnakeNode {
                    seg_id: seg.0,
                    trans,
                    entity: None,
                });
            } else {
                unreachable!()
            }
        }

        for (_id, snake) in snakes {
            let head_vel = snake.head_speed.unwrap();
            let mut body: Vec<_> = snake.body.into_iter().collect();
            for i in (1..body.len()).rev() {
                let mut trans =
                    body[i].trans.translation * 0.9 + body[i - 1].trans.translation * 0.1;
                trans.z = 0.0;
                let mut from = body[i].trans.translation;
                from.z = 0.0;
                let diff = trans - from;
                body[i].trans.translation = trans;

                body[i].trans.rotation = Quat::from_rotation_arc(Vec3::X, diff.normalize());
            }
            body[0].trans.translation +=
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
        spawn_food(
            &mut commands,
            Position::random(ARENA_WIDTH, ARENA_HEIGHT),
            &materials,
        );
    }
}

fn rotate((x, y): (f32, f32), theta: f32) -> Vec2 {
    let (x2, y2) = (theta.cos(), theta.sin());
    Vec2::new(
        x.clone() * x2.clone() - y.clone() * y2.clone(),
        x * y2 + y * x2,
    )
}
fn process_keyboard_input(keys: Res<Input<KeyCode>>, mut event: EventWriter<MovementEvent>) {
    if keys.pressed(KeyCode::Left) && keys.pressed(KeyCode::Right) {
        return;
    }
    if keys.pressed(KeyCode::Left) {
        event.send(MovementEvent {
            player_id: PlayerId(0),
            command: MovementCommand::TurnLeft,
        });
    } else if keys.pressed(KeyCode::Right) {
        event.send(MovementEvent {
            player_id: PlayerId(0),
            command: MovementCommand::TurnRight,
        });
    }
}
struct MovementEvent {
    player_id: PlayerId,
    command: MovementCommand,
}
fn process_movement(
    mut events: EventReader<MovementEvent>,
    mut q: Query<(&mut Velocity, &mut Transform, &PlayerId), With<SnakeHead>>,
) {
    const OMEGA: f32 = 2.0 * std::f32::consts::PI;
    const THETA: f32 = OMEGA * TICK;
    let mut movements = HashMap::default();
    for event in events.iter() {
        let command: &MovementCommand = &event.command;
        let angle = match command {
            MovementCommand::TurnLeft => THETA,
            MovementCommand::TurnRight => -THETA,
            MovementCommand::NoOps => 0.0,
        };
        movements.insert(event.player_id, angle);
    }
    for (mut vel, mut trans, player_id) in q.iter_mut() {
        if let Some(angle) = movements.get(&player_id) {
            *vel = Velocity(rotate((vel.0.x, vel.0.y), angle.clone()));
            trans.rotate(Quat::from_rotation_z(angle.clone()));
        }
    }
}
fn drive_all_ai(
    mut ai_manager: ResMut<AiManager>,
    mut snake_components: Query<(
        &Transform,
        &PlayerId,
        Option<&SnakeHead>,
        Option<&SnakeSegment>,
    )>,
    foods: Query<&Transform, With<Food>>,
    mut events: EventWriter<MovementEvent>,
) {
    let mut world = SnakeWorld::default();
    for trans in foods.iter() {
        world.foods.push(FoodBody {
            pos: Position(trans.translation.xy()),
        })
    }
    for (trans, player, head, segment) in snake_components.iter_mut() {
        let snake = world.snakes.entry(*player).or_default();
        if head.is_some() {
            snake.body.insert(SnakeNode {
                seg_id: 0,
                trans,
                entity: None,
            });
        } else if let Some(seg) = segment {
            snake.body.insert(SnakeNode {
                seg_id: seg.0,
                trans,
                entity: None,
            });
        } else {
            unreachable!()
        }
    }

    for (id, ai) in ai_manager.ais.iter_mut() {
        ai.feed_input(&world).unwrap();
        let output = ai.get_output().unwrap();
        events.send(MovementEvent {
            player_id: *id,
            command: output,
        })
    }
}
fn locate_food(
    pos: Vec3,
    radius: Radius,
    foods: &Query<(&Transform, &Radius, Entity), With<Food>>,
) -> Option<(Entity, Vec3)> {
    for p in foods.iter() {
        let (trans, radius1, entity): (&Transform, &Radius, Entity) = p;
        if trans.translation.distance(pos).abs() < radius.0.clone() + radius1.0.clone() {
            return Some((entity, trans.translation));
        }
    }
    None
}
fn eat_food_and_extend(
    mut commands: Commands,
    mut snake_components: Query<(
        &Transform,
        &PlayerId,
        &Radius,
        Option<&SnakeHead>,
        Option<&SnakeSegment>,
    )>,
    foods: Query<(&Transform, &Radius, Entity), With<Food>>,
    materials: Res<Materials>,
) {
    let mut snakes: HashMap<PlayerId, SnakeBody<&Transform>> = Default::default();
    for (trans, player, radius, head, segment) in snake_components.iter_mut() {
        let snake = snakes.entry(*player).or_default();
        if head.is_some() {
            snake.body.insert(SnakeNode {
                seg_id: 0,
                trans,
                entity: None,
            });
            snake.head_radius = Some(*radius);
        } else if let Some(seg) = segment {
            snake.body.insert(SnakeNode {
                seg_id: seg.0,
                trans,
                entity: None,
            });
        } else {
            unreachable!()
        }
    }
    for (player, snake) in snakes {
        let first = snake.body.iter().next().unwrap();
        if let Some((food, food_pos)) =
            locate_food(first.trans.translation, snake.head_radius.unwrap(), &foods)
        {
            commands.entity(food).despawn();
            spawn_snake_segment(
                &mut commands,
                snake.body.len() as _,
                player,
                Position(food_pos.xy()),
                &materials,
            );
        }
    }
}
fn death_detection(
    mut commands: Commands,
    mut snake_components: Query<(
        &Transform,
        &PlayerId,
        &Radius,
        Option<&SnakeHead>,
        Option<&SnakeSegment>,
        Entity,
    )>,
    materials: Res<Materials>,
) {
    let mut snakes: HashMap<PlayerId, SnakeBody<&Transform>> = Default::default();
    for (trans, player, radius, head, segment, entity) in snake_components.iter_mut() {
        let snake = snakes.entry(*player).or_default();
        if head.is_some() {
            snake.body.insert(SnakeNode {
                seg_id: 0,
                trans,
                entity: Some(entity),
            });
            snake.head_radius = Some(*radius);
        } else if let Some(seg) = segment {
            snake.body.insert(SnakeNode {
                seg_id: seg.0,
                trans,
                entity: Some(entity),
            });
        } else {
            unreachable!()
        }
    }
    for (player, snake) in &snakes {
        for (player2, snake2) in &snakes {
            if player == player2 {
                continue;
            }
            let mut collision = false;
            for node in &snake2.body {
                if snake
                    .body
                    .iter()
                    .next()
                    .unwrap()
                    .trans
                    .translation
                    .distance(node.trans.translation)
                    < snake.head_radius.unwrap().0 + snake2.head_radius.unwrap().0
                {
                    collision = true;
                }
            }
            if collision {
                for n in &snake.body {
                    commands.entity(n.entity.unwrap()).despawn();
                }
                spawn_snake_head(
                    &mut commands,
                    *player,
                    Position::random(ARENA_WIDTH, ARENA_HEIGHT),
                    Velocity::random(CONST_SPEED),
                    &materials,
                );
            }
        }
    }
}
fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Snakes!".to_string(),
            width: 640.0,
            height: 640.0,
            ..Default::default()
        })
        .insert_resource(AiManager::default())
        .add_event::<MovementEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("setup_game", SystemStage::single(setup_game.system()))
        .add_system(exit_on_esc_system.system())
        .add_system(food_spawner.system())
        .add_system(snake_move.system())
        .add_system(process_keyboard_input.system())
        .add_system(eat_food_and_extend.system())
        .add_system(death_detection.system())
        .add_system(process_movement.system())
        .add_system(drive_all_ai.system())
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .run();
}
