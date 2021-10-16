use anyhow::Context;
use anyhow::Result;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::ecs::system::EntityCommands;
use bevy::input::system::exit_on_esc_system;
#[allow(unused_imports)]
use bevy::log::*;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;
use the_snakes::controller::{Controller, MovementCommand, PlayerInfo, StdioController};
use the_snakes::{
    spawn_food, spawn_snake_segment, spawn_snake_with_nodes, Food, FoodBody, Materials, PlayerId,
    Position, Radius, SnakeBody, SnakeHead, SnakeNode, SnakeSegment, SnakeWorld, Velocity,
    ARENA_HEIGHT, ARENA_WIDTH, CONST_SPEED, TICK,
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
    let colors: Vec<Color> = colors
        .into_iter()
        .map(|(r, g, b)| (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
        .map(|(r, g, b)| Color::rgb(r, g, b))
        .collect();
    commands.insert_resource(Materials {
        colors: colors.clone(),
        head_material: colors
            .into_iter()
            .map(|x| materials.add(x.into()))
            .collect(),
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
    fn initialize_all_ai(
        &mut self,
        command: &mut Commands,
        materials: &Materials,
        registry: &mut PlayerInfoRegistry,
    ) -> Result<()> {
        for (k, v) in self.ais.iter_mut() {
            let info: PlayerInfo = v.initialize(*k)?;
            assert_eq!(info.is_ai, true);
            spawn_snake_with_nodes(
                command,
                *k,
                Position::random(ARENA_WIDTH, ARENA_HEIGHT),
                Velocity::random(CONST_SPEED),
                3,
                materials,
            );
            registry.player_infos.insert(*k, info);
        }
        Ok(())
    }
}
#[derive(Default)]
struct PlayerInfoRegistry {
    pub player_infos: BTreeMap<PlayerId, PlayerInfo>,
}
fn setup_game(
    mut commands: Commands,
    materials: Res<Materials>,
    mut controller: ResMut<AiManager>,
    mut registry: ResMut<PlayerInfoRegistry>,
) {
    spawn_snake_with_nodes(
        &mut commands,
        PlayerId(0),
        Position::random(ARENA_WIDTH, ARENA_HEIGHT),
        Velocity::random(CONST_SPEED),
        3,
        &materials,
    );
    registry.player_infos.insert(
        PlayerId(0),
        PlayerInfo {
            username: "player".to_string(),
            is_ai: false,
        },
    );
    match controller.load_all_ai("bin/ai") {
        Ok(()) => {}
        Err(err) => {
            error!("Could not load ai: {:?}", err);
        }
    }
    match controller.initialize_all_ai(&mut commands, &materials, &mut registry) {
        Ok(()) => {}
        Err(err) => {
            error!("Could not initialize ai: {:?}", err);
        }
    }
}
type CollectSnakeQuery<'a, 'b> = Query<
    'a,
    (
        &'b Transform,
        &'b PlayerId,
        Entity,
        Option<&'b Radius>,
        Option<&'b SnakeHead>,
        Option<&'b SnakeSegment>,
    ),
>;
fn collect_snakes<'a>(
    snake_components: &'a CollectSnakeQuery,
    registry: &PlayerInfoRegistry,
) -> BTreeMap<PlayerId, SnakeBody<&'a Transform>> {
    let mut world = SnakeWorld::default();
    for (trans, player, entity, radius, head, segment) in snake_components.iter() {
        let snake = world.snakes.entry(*player).or_default();
        snake.player_id = *player;
        if head.is_some() {
            snake.body.insert(
                0,
                SnakeNode {
                    seg_id: 0,
                    trans,
                    entity: Some(entity),
                },
            );
            snake.head_radius = radius.map(|x| *x);
            snake.player_info = registry.player_infos.get(player).map(|x| x.clone());
        } else if let Some(seg) = segment {
            snake.body.insert(
                seg.0,
                SnakeNode {
                    seg_id: seg.0,
                    trans,
                    entity: Some(entity),
                },
            );
        } else {
            unreachable!()
        }
    }
    world.snakes
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
                snake.body.insert(
                    0,
                    SnakeNode {
                        seg_id: 0,
                        trans,
                        entity: None,
                    },
                );
                snake.head_speed = Some(vel.cloned().unwrap());
            } else if let Some(seg) = segment {
                snake.body.insert(
                    seg.0,
                    SnakeNode {
                        seg_id: seg.0,
                        trans,
                        entity: None,
                    },
                );
            } else {
                unreachable!()
            }
        }

        for snake in snakes.values_mut() {
            let head_vel = snake.head_speed.unwrap();
            let mut body: Vec<_> = snake.body.values_mut().collect();
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
    snake_components: CollectSnakeQuery,
    foods: Query<&Transform, With<Food>>,
    mut events: EventWriter<MovementEvent>,
    registry: Res<PlayerInfoRegistry>,
) {
    let mut world = SnakeWorld::default();
    for trans in foods.iter() {
        world.foods.push(FoodBody {
            pos: Position(trans.translation.xy()),
        })
    }
    world.snakes = collect_snakes(&snake_components, &registry);

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
    snake_components: CollectSnakeQuery,
    foods: Query<(&Transform, &Radius, Entity), With<Food>>,
    materials: Res<Materials>,
    registry: Res<PlayerInfoRegistry>,
) {
    let snakes = collect_snakes(&snake_components, &registry);
    for (player, snake) in snakes {
        let first = snake.body.values().next().unwrap();
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
    snake_components: CollectSnakeQuery,
    materials: Res<Materials>,
    registry: Res<PlayerInfoRegistry>,
) {
    let snakes = collect_snakes(&snake_components, &registry);
    for (player, snake) in &snakes {
        for (player2, snake2) in &snakes {
            if player == player2 {
                continue;
            }
            let mut collision = false;
            for node in snake2.body.values() {
                if snake
                    .body
                    .values()
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
                for n in snake.body.values() {
                    commands.entity(n.entity.unwrap()).despawn();
                }
                spawn_snake_with_nodes(
                    &mut commands,
                    *player,
                    Position::random(ARENA_WIDTH, ARENA_HEIGHT),
                    Velocity::random(CONST_SPEED),
                    3,
                    &materials,
                );
            }
        }
    }
}
struct LeaderBoard;

fn draw_text<'a, 'b>(
    commands: &'b mut Commands<'a>,
    text: impl Into<String>,
    font_size: f32,
    color: Color,
    pos: Vec2,
    font: Handle<Font>,
) -> EntityCommands<'a, 'b> {
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            text,
            TextStyle {
                font,
                font_size,
                color,
            },
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Left,
            },
        ),
        transform: Transform::from_xyz(pos.x.clone(), pos.y.clone(), 100.0),
        ..Default::default()
    })
}
fn draw_leaderboard(
    mut commands: Commands,
    last: Query<Entity, With<LeaderBoard>>,
    asset_server: Res<AssetServer>,
    snakes: CollectSnakeQuery,
    materials: Res<Materials>,
    registry: Res<PlayerInfoRegistry>,
) {
    last.for_each(|x| commands.entity(x).despawn());
    let font: Handle<Font> = asset_server.load("fonts/Arial.ttf");
    let snakes = collect_snakes(&snakes, &registry);
    let pos_x = 300.0;
    let mut pos_y = 0.0;
    for snake in snakes.values() {
        let color = materials.colors[snake.player_id.0.clone() as usize];
        // | player_name | 0 score(s) |
        draw_text(
            &mut commands,
            format!(
                "{}.{}: score(s)",
                snake.player_id.0,
                snake
                    .player_info
                    .as_ref()
                    .map(|x| x.username.as_str())
                    .unwrap_or("unnamed")
            ),
            24.0,
            color,
            Vec2::new(pos_x.clone(), pos_y.clone()),
            font.clone(),
        )
        .insert(LeaderBoard);
        pos_y -= 20.0;
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
        .insert_resource(PlayerInfoRegistry::default())
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
        .add_system(draw_leaderboard.system())
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .run();
}
