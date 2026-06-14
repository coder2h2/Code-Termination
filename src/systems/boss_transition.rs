use bevy::prelude::*;
use crate::resources::*;
use crate::components::*;

#[derive(Component)]
pub struct BossTransitionUI;

#[derive(Component)]
pub struct BossTransitionText;

pub fn setup_boss_transition(
    mut commands: Commands,
    level_entity_query: Query<Entity, With<LevelEntity>>,
    hud_query: Query<Entity, With<GameHUD>>,
) {
    // Despawn old level entities and HUD
    for entity in &level_entity_query {
        commands.entity(entity).despawn();
    }
    for entity in &hud_query {
        commands.entity(entity).despawn();
    }

    // Spawn warning screen
    commands.spawn((
        BossTransitionUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.01, 0.01)), // Very dark red
    ))
    .with_children(|parent| {
        parent.spawn((
            BossTransitionText,
            Text::new("!!! WARNING: SYSTEM SECURITY CRITICAL !!!\n\nCOREDUMP PROCESS BREACH IN PROGRESS\nINITIALIZING SECTOR 04 CONTAINER...\n\nACCESS GRANTED IN: 3.0s"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.1, 0.1)), // Bright neon red
            TextLayout {
                justify: Justify::Center,
                ..default()
            },
        ));
    });
}

pub fn cleanup_boss_transition(
    mut commands: Commands,
    query: Query<Entity, With<BossTransitionUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn boss_transition_system(
    time: Res<Time>,
    mut timer: Local<f32>,
    mut initialized: Local<bool>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending_load: ResMut<PendingGameLoad>,
    mut text_query: Query<&mut Text, With<BossTransitionText>>,
    mut bg_query: Query<&mut BackgroundColor, With<BossTransitionUI>>,
) {
    if !*initialized {
        *timer = 3.0;
        *initialized = true;
    }

    *timer -= time.delta_secs();

    // Pulse red background in and out using a sine wave
    let elapsed = 3.0 - *timer;
    let pulse = (elapsed * std::f32::consts::PI * 2.0).sin().abs(); // 1Hz frequency
    let red_val = 0.05 + pulse * 0.35; // pulses between 0.05 (almost black) and 0.40 (dim red)
    if let Ok(mut bg) = bg_query.single_mut() {
        bg.0 = Color::srgb(red_val, 0.01, 0.01);
    }

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!(
            "!!! WARNING: SYSTEM SECURITY CRITICAL !!!\n\nCOREDUMP PROCESS BREACH IN PROGRESS\nINITIALIZING SECTOR 04 CONTAINER...\n\nACCESS GRANTED IN: {:.1}s",
            timer.max(0.0)
        );
    }

    if *timer <= 0.0 {
        *initialized = false; // Reset for next time
        pending_load.should_load = true;
        next_state.set(AppState::Game);
    }
}
