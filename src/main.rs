use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::system::exit_on_esc_system;
use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(basic_setup.system().label("basic_setup"))
        .add_startup_system(game_setup.system().after("basic_setup"))
        .add_system(template_animation.system())
        .add_system(exit_on_esc_system.system())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}

pub struct PlayerId(pub i32);
struct SnakeHead;
struct Materials {
    head_material: Handle<ColorMaterial>,
}

fn basic_setup(mut commands: Commands, asset_server: Res<AssetServer>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
    });
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(
            "Would you like to play a game?",
            TextStyle {
                font: asset_server.load("fonts/tiny.ttf"),
                font_size: 58.0,
                color: Color::WHITE,
            },
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        ),
        ..Default::default()
    });
}

fn spawn_node(mut commands: Commands, player: PlayerId, pos: Vec2, materials: &Materials) -> Entity {
    commands.spawn()
        .insert(player)
        .insert(SnakeHead)
        .insert(Transform::from_xyz(pos.x, pos.y, 0.0))
        .insert_bundle(SpriteBundle {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .id()
}

fn game_setup(mut commands: Commands, materials: Res<Materials>) {
    spawn_node(commands, PlayerId(0), Vec2::new(0.0, 0.0), &materials);

}
fn template_animation(time: Res<Time>, mut query: Query<&mut Transform, With<Text>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x = 100.0 * time.seconds_since_startup().sin() as f32;
        transform.translation.y = 100.0 * time.seconds_since_startup().cos() as f32;
    }
}
