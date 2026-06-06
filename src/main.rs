use bevy::prelude::*;
use bevy::window::WindowResolution;


const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const PLAYER_SIZE: Vec2 = Vec2::new(96.0, 96.0);
const GROUND_SIZE: Vec2 = Vec2::new(2200.0, 120.0);
const PLAYER_SPEED: f32 = 420.0;
const JUMP_SPEED: f32 = 720.0;
const GRAVITY: f32 = 1800.0;
const GROUND_Y: f32 = -110.0;

const DASH_SPEED: f32 = 1400.0;
const DASH_DURATION: f32 = 0.18;
const DOUBLE_TAP_TIMEOUT: f32 = 0.22;
const SMASHDOWN_SPEED: f32 = -2000.0;

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct JumpState {
    jumps_remaining: u32,
    max_jumps: u32,
    is_smashing: bool,
}

impl Default for JumpState {
    fn default() -> Self {
        Self {
            jumps_remaining: 2,
            max_jumps: 2,
            is_smashing: false,
        }
    }
}

#[derive(Component)]
struct DashState {
    last_a_press: f32,
    last_d_press: f32,
    dash_timer: f32,
    dash_dir: f32,
    air_dash_used: bool,
}

impl Default for DashState {
    fn default() -> Self {
        Self {
            last_a_press: -10.0,
            last_d_press: -10.0,
            dash_timer: 0.0,
            dash_dir: 0.0,
            air_dash_used: false,
        }
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum AppState {
    #[default]
    Game,
    Settings,
}

#[derive(Resource)]
struct GameSettings {
    player_speed: f32,
    jump_speed: f32,
    gravity: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            player_speed: PLAYER_SPEED,
            jump_speed: JUMP_SPEED,
            gravity: GRAVITY,
        }
    }
}

#[derive(Component)]
struct SettingsMenuUI;

#[derive(Component, Clone, Copy)]
enum SettingsButtonAction {
    DecSpeed,
    IncSpeed,
    DecJump,
    IncJump,
    DecGravity,
    IncGravity,
    Resume,
}

#[derive(Component, Clone, Copy)]
enum SettingsValueText {
    Speed,
    Jump,
    Gravity,
}

#[derive(Component)]
struct GameplayUI;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.14)))
        .insert_resource(GameSettings::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "gidames".into(),
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::Game), setup_gameplay_ui)
        .add_systems(OnExit(AppState::Game), cleanup_gameplay_ui)
        .add_systems(Update, toggle_settings_menu)
        .add_systems(Update, (
            move_player,
            jump_player,
            apply_velocity,
            gameplay_button_system,
        ).run_if(in_state(AppState::Game)))
        .add_systems(OnEnter(AppState::Settings), setup_settings_menu)
        .add_systems(OnExit(AppState::Settings), cleanup_settings_menu)
        .add_systems(Update, (
            settings_button_system,
            update_settings_ui,
        ).run_if(in_state(AppState::Settings)))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.16, 0.18, 0.24), GROUND_SIZE),
        Transform::from_xyz(0.0, -220.0, 0.0),
    ));

    commands.spawn((
        Player,
        Velocity::default(),
        JumpState::default(),
        DashState::default(),
        Sprite::from_color(Color::srgb(0.48, 0.86, 0.62), PLAYER_SIZE),
        Transform::from_xyz(0.0, GROUND_Y, 1.0),
    ));
}

fn setup_gameplay_ui(mut commands: Commands) {
    commands.spawn((
        GameplayUI,
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(20.0),
            width: Val::Px(100.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
        BorderColor::all(Color::srgb(0.48, 0.86, 0.62)),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Settings"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn cleanup_gameplay_ui(mut commands: Commands, query: Query<Entity, With<GameplayUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn gameplay_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<GameplayUI>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                next_state.set(AppState::Settings);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
            }
        }
    }
}

fn toggle_settings_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::Game => next_state.set(AppState::Settings),
            AppState::Settings => next_state.set(AppState::Game),
        }
    }
}

fn setup_settings_menu(mut commands: Commands, settings: Res<GameSettings>) {
    // Spawn the dark background container overlay
    commands.spawn((
        SettingsMenuUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
    ))
    .with_children(|parent| {
        // Main panel card
        parent.spawn((
            Node {
                width: Val::Px(500.0),
                height: Val::Px(420.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(3.0)),
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.09, 0.11, 0.16)),
            BorderColor::all(Color::srgb(0.2, 0.25, 0.35)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SETTINGS"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.48, 0.86, 0.62)),
            ));

            // Row for Player Speed
            spawn_settings_row(parent, "Player Speed", format!("{:.0}", settings.player_speed), SettingsButtonAction::DecSpeed, SettingsButtonAction::IncSpeed, SettingsValueText::Speed);

            // Row for Jump Speed
            spawn_settings_row(parent, "Jump Speed", format!("{:.0}", settings.jump_speed), SettingsButtonAction::DecJump, SettingsButtonAction::IncJump, SettingsValueText::Jump);

            // Row for Gravity
            spawn_settings_row(parent, "Gravity", format!("{:.0}", settings.gravity), SettingsButtonAction::DecGravity, SettingsButtonAction::IncGravity, SettingsValueText::Gravity);

            // Resume/Close button
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(45.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.48, 0.86, 0.62)),
                SettingsButtonAction::Resume,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Resume"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

fn spawn_settings_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    val_str: String,
    dec_action: SettingsButtonAction,
    inc_action: SettingsButtonAction,
    val_marker: SettingsValueText,
) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(50.0),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Row,
        padding: UiRect::horizontal(Val::Px(10.0)),
        ..default()
    })
    .with_children(|row| {
        // Label
        row.spawn((
            Text::new(label.to_string()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                width: Val::Px(150.0),
                ..default()
            },
        ));

        // Controls container
        row.spawn(Node {
            width: Val::Px(250.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|ctrl| {
            // Decrement Button
            ctrl.spawn((
                Button,
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(35.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.3, 0.35, 0.45)),
                dec_action,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("-"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // Value text
            ctrl.spawn((
                Text::new(val_str),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                val_marker,
            ));

            // Increment Button
            ctrl.spawn((
                Button,
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(35.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.3, 0.35, 0.45)),
                inc_action,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("+"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

fn cleanup_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn settings_button_system(
    mut interaction_query: Query<
        (&Interaction, &SettingsButtonAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                match action {
                    SettingsButtonAction::DecSpeed => {
                        settings.player_speed = (settings.player_speed - 20.0).max(100.0);
                    }
                    SettingsButtonAction::IncSpeed => {
                        settings.player_speed = (settings.player_speed + 20.0).min(1200.0);
                    }
                    SettingsButtonAction::DecJump => {
                        settings.jump_speed = (settings.jump_speed - 50.0).max(200.0);
                    }
                    SettingsButtonAction::IncJump => {
                        settings.jump_speed = (settings.jump_speed + 50.0).min(1500.0);
                    }
                    SettingsButtonAction::DecGravity => {
                        settings.gravity = (settings.gravity - 100.0).max(300.0);
                    }
                    SettingsButtonAction::IncGravity => {
                        settings.gravity = (settings.gravity + 100.0).min(4000.0);
                    }
                    SettingsButtonAction::Resume => {
                        next_state.set(AppState::Game);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
            }
        }
    }
}

fn update_settings_ui(
    settings: Res<GameSettings>,
    mut text_query: Query<(&mut Text, &SettingsValueText)>,
) {
    if !settings.is_changed() {
        return;
    }
    for (mut text, value_type) in &mut text_query {
        match value_type {
            SettingsValueText::Speed => {
                *text = Text::new(format!("{:.0}", settings.player_speed));
            }
            SettingsValueText::Jump => {
                *text = Text::new(format!("{:.0}", settings.jump_speed));
            }
            SettingsValueText::Gravity => {
                *text = Text::new(format!("{:.0}", settings.gravity));
            }
        }
    }
}

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut player_query: Query<(&mut Transform, &mut DashState, &mut Sprite, &JumpState), With<Player>>,
) {
    let now = time.elapsed_secs();
    let delta = time.delta_secs();

    for (mut transform, mut dash_state, mut sprite, jump_state) in &mut player_query {
        if jump_state.is_smashing {
            // Lock movement and set smashdown color (fiery orange/red)
            sprite.color = Color::srgb(1.0, 0.25, 0.0);
            dash_state.dash_timer = 0.0; // Cancel any active dash
            continue;
        }

        let is_in_air = transform.translation.y > GROUND_Y;

        // Detect double tap for A or ArrowLeft (left)
        if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
            if now - dash_state.last_a_press < DOUBLE_TAP_TIMEOUT && dash_state.dash_timer <= 0.0 {
                if !is_in_air || !dash_state.air_dash_used {
                    dash_state.dash_timer = DASH_DURATION;
                    dash_state.dash_dir = -1.0;
                    if is_in_air {
                        dash_state.air_dash_used = true;
                    }
                }
            }
            dash_state.last_a_press = now;
        }

        // Detect double tap for D or ArrowRight (right)
        if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
            if now - dash_state.last_d_press < DOUBLE_TAP_TIMEOUT && dash_state.dash_timer <= 0.0 {
                if !is_in_air || !dash_state.air_dash_used {
                    dash_state.dash_timer = DASH_DURATION;
                    dash_state.dash_dir = 1.0;
                    if is_in_air {
                        dash_state.air_dash_used = true;
                    }
                }
            }
            dash_state.last_d_press = now;
        }

        // Apply movement and change color based on dash status
        if dash_state.dash_timer > 0.0 {
            // Player is dashing
            transform.translation.x += dash_state.dash_dir * DASH_SPEED * delta;
            dash_state.dash_timer -= delta;
            
            // Set dash color (magenta/pink for air dash, cyan for ground dash)
            if dash_state.air_dash_used {
                sprite.color = Color::srgb(1.0, 0.3, 0.8);
            } else {
                sprite.color = Color::srgb(0.2, 0.9, 1.0);
            }
            
            if dash_state.dash_timer < 0.0 {
                dash_state.dash_timer = 0.0;
            }
        } else {
            // Normal color (pastel green)
            sprite.color = Color::srgb(0.48, 0.86, 0.62);
            
            // Normal movement
            let mut direction = 0.0;
            if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
                direction -= 1.0;
            }
            if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
                direction += 1.0;
            }
            if direction != 0.0 {
                transform.translation.x += direction * settings.player_speed * delta;
            }
        }
    }
}

fn jump_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    settings: Res<GameSettings>,
    mut player_query: Query<(&Transform, &mut Velocity, &mut JumpState), With<Player>>,
) {
    let down_pressed = keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::ArrowDown);
    let up_pressed = keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::ArrowUp);

    for (transform, mut velocity, mut jump_state) in &mut player_query {
        let is_in_air = transform.translation.y > GROUND_Y;

        if down_pressed && is_in_air {
            velocity.0.y = SMASHDOWN_SPEED;
            jump_state.is_smashing = true;
        } else if up_pressed && jump_state.jumps_remaining > 0 && !jump_state.is_smashing {
            velocity.0.y = settings.jump_speed;
            jump_state.jumps_remaining -= 1;
        }
    }
}

fn apply_velocity(
    time: Res<Time>,
    settings: Res<GameSettings>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut JumpState, &mut DashState), With<Player>>,
) {
    for (mut transform, mut velocity, mut jump_state, mut dash_state) in &mut player_query {
        if dash_state.dash_timer > 0.0 {
            // Freeze vertical velocity during dash
            velocity.0.y = 0.0;
        } else {
            velocity.0.y -= settings.gravity * time.delta_secs();
        }
        
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        if transform.translation.y < GROUND_Y {
            transform.translation.y = GROUND_Y;
            velocity.0.y = 0.0;
            jump_state.jumps_remaining = jump_state.max_jumps;
            jump_state.is_smashing = false;
            dash_state.air_dash_used = false;
        }
    }
}
