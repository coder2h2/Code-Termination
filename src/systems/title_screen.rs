use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

pub fn setup_title_screen(
    mut commands: Commands,
    hud_query: Query<Entity, With<GameHUD>>,
    mut menu_selection: ResMut<MenuSelection>,
) {
    menu_selection.selected_index = 0;
    for entity in &hud_query {
        commands.entity(entity).despawn();
    }

    // Spawn the black background container overlay
    commands.spawn((
        TitleScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
    ))
    .with_children(|parent| {
        // Title Text
        parent.spawn((
            Text::new("Code-Termination"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // Play Button (Continue)
        parent.spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            TitleButtonAction::Play,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("PLAY"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // New Game Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            TitleButtonAction::NewGame,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("NEW GAME"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // Achievements Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            TitleButtonAction::Achievements,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("ACHIEVEMENTS"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // DLC Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            TitleButtonAction::Dlc,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("DLC"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // Quit Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            TitleButtonAction::Quit,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("QUIT"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });
    });
}

pub fn cleanup_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreenUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn title_button_system(
    mut button_query: Query<
        (Entity, &Interaction, &TitleButtonAction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_events: MessageWriter<AppExit>,
    mut pending_load: ResMut<PendingGameLoad>,
    mut menu_selection: ResMut<MenuSelection>,
    gamepads: Query<&Gamepad>,
    mut stick_neutral: Local<bool>,
) {
    let mut buttons = button_query.iter_mut().collect::<Vec<_>>();
    buttons.sort_by_key(|(_, _, action, _)| match action {
        TitleButtonAction::Play => 0,
        TitleButtonAction::NewGame => 1,
        TitleButtonAction::Achievements => 2,
        TitleButtonAction::Dlc => 3,
        TitleButtonAction::Quit => 4,
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

    for (index, (_, interaction, _, mut bg_color)) in buttons.into_iter().enumerate() {
        if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
        } else if index == menu_selection.selected_index {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }

    if let Some(action) = trigger_action {
        match action {
            TitleButtonAction::Play => {
                pending_load.should_load = true;
                next_state.set(AppState::Game);
            }
            TitleButtonAction::NewGame => {
                pending_load.should_load = false;
                next_state.set(AppState::Game);
            }
            TitleButtonAction::Achievements => {
                next_state.set(AppState::Achievements);
            }
            TitleButtonAction::Dlc => {
                next_state.set(AppState::DlcMenu);
            }
            TitleButtonAction::Quit => {
                app_exit_events.write(AppExit::Success);
            }
        }
    }
}
