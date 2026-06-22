use bevy::prelude::*;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use crate::components::*;
use crate::resources::*;
use crate::helpers::save_user_profile;

pub fn setup_user_register(
    mut commands: Commands,
    profile: Res<UserProfile>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !profile.username.is_empty() {
        next_state.set(AppState::MultiplayerMenu);
        return;
    }

    commands.spawn((
        UserRegisterUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.09, 0.14)),
    ))
    .with_children(|parent| {
        // Title Text
        parent.spawn((
            Text::new("ESTABLISH PLAYER IDENTITY"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        // Info text
        parent.spawn((
            Text::new("A unique username is required for identification"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Input label
        parent.spawn((
            Text::new("ENTER USERNAME:"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // Input field box
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 1.0)),
        ))
        .with_children(|field| {
            field.spawn((
                UserRegisterInputText,
                Text::new("_"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // Status instructions
        parent.spawn((
            UserRegisterStatusText,
            Text::new("Type username using alphanumeric characters and press Enter"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Buttons container
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Px(400.0),
                ..default()
            },
        ))
        .with_children(|btn_container| {
            // Register Button
            btn_container.spawn((
                Button,
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
                UserRegisterButtonAction::Register,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("REGISTER"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                ));
            });

            // Cancel Button
            btn_container.spawn((
                Button,
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                UserRegisterButtonAction::Quit,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("CANCEL"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            });
        });
    });
}

pub fn cleanup_user_register(
    mut commands: Commands,
    query: Query<Entity, With<UserRegisterUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn user_register_button_system(
    mut button_query: Query<
        (&Interaction, &UserRegisterButtonAction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    username_text_query: Query<&Text, With<UserRegisterInputText>>,
    mut profile: ResMut<UserProfile>,
) {
    let mut username = String::new();
    if let Ok(text) = username_text_query.single() {
        let t = text.0.as_str();
        if t.ends_with('_') {
            username = t[..t.len() - 1].to_string();
        } else {
            username = t.to_string();
        }
    }

    for (interaction, action, mut bg_color) in &mut button_query {
        if *interaction == Interaction::Hovered {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            match action {
                UserRegisterButtonAction::Register => {
                    let trimmed = username.trim();
                    if !trimmed.is_empty() {
                        let uid = trimmed.chars().rev().collect::<String>();
                        save_user_profile(trimmed, &uid);
                        profile.username = trimmed.to_string();
                        profile.uid = uid;
                        next_state.set(AppState::MultiplayerMenu);
                    }
                }
                UserRegisterButtonAction::Quit => {
                    next_state.set(AppState::MultiplayerMenu);
                }
            }
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }
}

pub fn user_register_keyboard_system(
    mut char_evr: MessageReader<KeyboardInput>,
    mut username_input: Local<String>,
    mut query_text: Query<&mut Text, (With<UserRegisterInputText>, Without<UserRegisterStatusText>)>,
    mut query_status: Query<&mut Text, (With<UserRegisterStatusText>, Without<UserRegisterInputText>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut profile: ResMut<UserProfile>,
) {
    let mut updated = false;
    for ev in char_evr.read() {
        if ev.state == ButtonState::Released {
            continue;
        }

        match &ev.logical_key {
            Key::Enter => {
                let trimmed = username_input.trim();
                if !trimmed.is_empty() {
                    let uid = trimmed.chars().rev().collect::<String>();
                    save_user_profile(trimmed, &uid);
                    profile.username = trimmed.to_string();
                    profile.uid = uid;
                    next_state.set(AppState::MultiplayerMenu);
                    return;
                } else {
                    if let Ok(mut status) = query_status.single_mut() {
                        status.0 = "Username cannot be empty!".to_string();
                    }
                }
            }
            Key::Escape => {
                next_state.set(AppState::MultiplayerMenu);
                return;
            }
            Key::Backspace => {
                username_input.pop();
                updated = true;
            }
            Key::Character(input) => {
                if username_input.len() < 16 {
                    for c in input.chars() {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                            username_input.push(c);
                            updated = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if updated {
        if let Ok(mut text) = query_text.single_mut() {
            text.0 = format!("{}_", *username_input);
        }
        if let Ok(mut status) = query_status.single_mut() {
            status.0 = "Type username using alphanumeric characters and press Enter".to_string();
        }
    }
}

pub fn setup_user_name_change(
    mut commands: Commands,
    profile: Res<UserProfile>,
) {
    commands.spawn((
        UserNameChangeUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.09, 0.14)),
    ))
    .with_children(|parent| {
        // Title Text
        parent.spawn((
            Text::new("CHANGE PLAYER IDENTITY"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
        ));

        // Subtitle showing current details
        parent.spawn((
            Text::new(format!("Current Username: {}", profile.username)),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Input label
        parent.spawn((
            Text::new("ENTER NEW USERNAME:"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // Input field box
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 1.0)),
        ))
        .with_children(|field| {
            field.spawn((
                UserNameChangeInputText,
                Text::new(format!("{}_", profile.username)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // Status instructions
        parent.spawn((
            UserNameChangeStatusText,
            Text::new("Type new username and press Enter to save"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Buttons container
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Px(400.0),
                ..default()
            },
        ))
        .with_children(|btn_container| {
            // Save Button
            btn_container.spawn((
                Button,
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
                UserNameChangeButtonAction::Save,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("SAVE CHANGES"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                ));
            });

            // Back Button
            btn_container.spawn((
                Button,
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                UserNameChangeButtonAction::Back,
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
    });
}

pub fn cleanup_user_name_change(
    mut commands: Commands,
    query: Query<Entity, With<UserNameChangeUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn user_name_change_button_system(
    mut button_query: Query<
        (&Interaction, &UserNameChangeButtonAction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    username_text_query: Query<&Text, With<UserNameChangeInputText>>,
    mut profile: ResMut<UserProfile>,
) {
    let mut username = String::new();
    if let Ok(text) = username_text_query.single() {
        let t = text.0.as_str();
        if t.ends_with('_') {
            username = t[..t.len() - 1].to_string();
        } else {
            username = t.to_string();
        }
    }

    for (interaction, action, mut bg_color) in &mut button_query {
        if *interaction == Interaction::Hovered {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            match action {
                UserNameChangeButtonAction::Save => {
                    let trimmed = username.trim();
                    if !trimmed.is_empty() {
                        let uid = trimmed.chars().rev().collect::<String>();
                        save_user_profile(trimmed, &uid);
                        profile.username = trimmed.to_string();
                        profile.uid = uid;
                        next_state.set(AppState::TitleScreen);
                    }
                }
                UserNameChangeButtonAction::Back => {
                    next_state.set(AppState::TitleScreen);
                }
            }
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }
}

pub fn user_name_change_keyboard_system(
    mut char_evr: MessageReader<KeyboardInput>,
    mut username_input: Local<String>,
    mut query_text: Query<&mut Text, (With<UserNameChangeInputText>, Without<UserNameChangeStatusText>)>,
    mut query_status: Query<&mut Text, (With<UserNameChangeStatusText>, Without<UserNameChangeInputText>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut profile: ResMut<UserProfile>,
) {
    if username_input.is_empty() && !profile.username.is_empty() {
        *username_input = profile.username.clone();
    }

    let mut updated = false;
    for ev in char_evr.read() {
        if ev.state == ButtonState::Released {
            continue;
        }

        match &ev.logical_key {
            Key::Enter => {
                let trimmed = username_input.trim();
                if !trimmed.is_empty() {
                    let uid = trimmed.chars().rev().collect::<String>();
                    save_user_profile(trimmed, &uid);
                    profile.username = trimmed.to_string();
                    profile.uid = uid;
                    next_state.set(AppState::TitleScreen);
                    username_input.clear();
                    return;
                } else {
                    if let Ok(mut status) = query_status.single_mut() {
                        status.0 = "Username cannot be empty!".to_string();
                    }
                }
            }
            Key::Escape => {
                next_state.set(AppState::TitleScreen);
                username_input.clear();
                return;
            }
            Key::Backspace => {
                username_input.pop();
                updated = true;
            }
            Key::Character(input) => {
                if username_input.len() < 16 {
                    for c in input.chars() {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                            username_input.push(c);
                            updated = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if updated {
        if let Ok(mut text) = query_text.single_mut() {
            text.0 = format!("{}_", *username_input);
        }
        if let Ok(mut status) = query_status.single_mut() {
            status.0 = "Type new username and press Enter to save".to_string();
        }
    }
}
