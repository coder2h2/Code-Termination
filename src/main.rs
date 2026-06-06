use bevy::prelude::*;
use bevy::window::WindowResolution;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const PLAYER_SIZE: Vec2 = Vec2::new(96.0, 96.0);
const GROUND_SIZE: Vec2 = Vec2::new(2200.0, 120.0);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.14)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "gidames".into(),
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.16, 0.18, 0.24), GROUND_SIZE),
        Transform::from_xyz(0.0, -220.0, 0.0),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.48, 0.86, 0.62), PLAYER_SIZE),
        Transform::from_xyz(0.0, -110.0, 1.0),
    ));
}
