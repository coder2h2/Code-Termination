use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::helpers::save_game;
use crate::constants::GROUND_Y;

#[derive(Component)]
pub struct ModeSelectUI;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Component)]
pub enum ModeSelectButtonAction {
    Normal,
    Hacker,
    Back,
}

pub fn setup_mode_select(
    mut commands: Commands,
    hud_query: Query<Entity, With<GameHUD>>,
    mut menu_selection: ResMut<MenuSelection>,
) {
    menu_selection.selected_index = 0;
    for entity in &hud_query {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        ModeSelectUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.09, 0.14)), // dark blue/gray
    ))
    .with_children(|parent| {
        // Mode Select Title
        parent.spawn((
            Text::new("SELECT OPERATION MODE"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 1.0)), // Neon Cyan
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // Normal Mode Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(320.0),
                height: Val::Px(55.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 1.0)),
            ModeSelectButtonAction::Normal,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("NORMAL MODE"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 1.0)),
            ));
        });

        // Hacker Mode Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(320.0),
                height: Val::Px(55.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(1.0, 0.0, 0.0)),
            ModeSelectButtonAction::Hacker,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("H@CKER M0D3"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
            ));
        });

        // Back Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(320.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
            ModeSelectButtonAction::Back,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("BACK"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
    });
}

pub fn cleanup_mode_select(
    mut commands: Commands,
    query: Query<Entity, With<ModeSelectUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn mode_select_button_system(
    mut next_state: ResMut<NextState<AppState>>,
    mut hacker_mode: ResMut<HackerMode>,
    mut menu_selection: ResMut<MenuSelection>,
    mut pending_load: ResMut<PendingGameLoad>,
    gamepads: Query<&Gamepad>,
    mut stick_neutral: Local<bool>,
    mut button_query: Query<(Entity, &Interaction, &ModeSelectButtonAction, &mut BackgroundColor), With<Button>>,
) {
    let mut buttons = button_query.iter_mut().collect::<Vec<_>>();
    buttons.sort_by_key(|(_, _, action, _)| match action {
        ModeSelectButtonAction::Normal => 0,
        ModeSelectButtonAction::Hacker => 1,
        ModeSelectButtonAction::Back => 2,
    });
    let total_buttons = buttons.len();

    let mut up = false;
    let mut down = false;
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            up = true;
        }
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            down = true;
        }
    }

    let mut stick_up = false;
    let mut stick_down = false;
    let mut any_active = false;
    for gamepad in &gamepads {
        if let Some(y) = gamepad.get(GamepadAxis::LeftStickY) {
            if y > 0.5 {
                any_active = true;
                if *stick_neutral {
                    stick_up = true;
                }
            } else if y < -0.5 {
                any_active = true;
                if *stick_neutral {
                    stick_down = true;
                }
            }
        }
    }
    if any_active {
        *stick_neutral = false;
    } else {
        *stick_neutral = true;
    }

    if total_buttons > 0 {
        if up || stick_up {
            menu_selection.selected_index = if menu_selection.selected_index == 0 {
                total_buttons - 1
            } else {
                menu_selection.selected_index - 1
            };
        }
        if down || stick_down {
            menu_selection.selected_index = (menu_selection.selected_index + 1) % total_buttons;
        }
    }

    for (index, (_, interaction, _, _)) in buttons.iter().enumerate() {
        if *interaction == &Interaction::Hovered {
            menu_selection.selected_index = index;
        }
    }

    let gp_select = gamepads.iter().any(|g| g.just_pressed(GamepadButton::East));
    let mut trigger_action = None;
    for (index, (_, interaction, action, _)) in buttons.iter().enumerate() {
        if *interaction == &Interaction::Pressed || (gp_select && index == menu_selection.selected_index) {
            trigger_action = Some(*action);
        }
    }

    for (index, (_, _, action, mut bg_color)) in buttons.into_iter().enumerate() {
        let base_color = match action {
            ModeSelectButtonAction::Normal => Color::srgb(0.05, 0.05, 0.07),
            ModeSelectButtonAction::Hacker => Color::srgb(0.05, 0.05, 0.07),
            ModeSelectButtonAction::Back => Color::srgb(0.12, 0.12, 0.15),
        };
        let hover_color = match action {
            ModeSelectButtonAction::Normal => Color::srgb(0.0, 0.3, 0.3),
            ModeSelectButtonAction::Hacker => Color::srgb(0.4, 0.05, 0.05),
            ModeSelectButtonAction::Back => Color::srgb(0.2, 0.2, 0.25),
        };

        if index == menu_selection.selected_index {
            *bg_color = BackgroundColor(hover_color);
        } else {
            *bg_color = BackgroundColor(base_color);
        }
    }

    if let Some(action) = trigger_action {
        match action {
            ModeSelectButtonAction::Normal => {
                hacker_mode.active = false;
                pending_load.should_load = false;
                save_game(-350.0, GROUND_Y, 6, true, 1, false);
                next_state.set(AppState::Game);
            }
            ModeSelectButtonAction::Hacker => {
                hacker_mode.active = true;
                pending_load.should_load = false;
                save_game(-350.0, GROUND_Y, 6, true, 1, true);
                next_state.set(AppState::Game);
            }
            ModeSelectButtonAction::Back => {
                next_state.set(AppState::TitleScreen);
            }
        }
    }
}
