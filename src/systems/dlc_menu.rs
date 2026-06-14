use bevy::prelude::*;
use crate::resources::*;
use crate::helpers::has_dlc;

#[derive(Component)]
pub struct DlcMenuUI;

pub fn setup_dlc_screen(
    mut commands: Commands,
) {
    commands.spawn((
        DlcMenuUI,
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
        // Title
        parent.spawn((
            Text::new("=== SYSTEM DOWNLOADABLE CONTENT (DLC) ==="),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // List Container
        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            margin: UiRect::bottom(Val::Px(40.0)),
            ..default()
        })
        .with_children(|list| {
            let active = has_dlc();
            let status_text = if active {
                "[INSTALLED & ACTIVE]"
            } else {
                "[NOT INSTALLED]"
            };
            let color = if active {
                Color::srgb(0.0, 1.0, 0.0) // Green
            } else {
                Color::srgb(1.0, 0.2, 0.2) // Red
            };

            // DLC Title and Status
            list.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("• H@CKER M0D3  "),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                ));
                row.spawn((
                    Text::new(status_text),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(color),
                ));
            });

            // DLC Description
            list.spawn((
                Text::new("  Description: Makes the game significantly harder. Enemies run faster,\n               you take double damage, and the boss has double health."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));

            if !active {
                // Instruction on how to install
                list.spawn((
                    Text::new("  How to install: Switch to the private DLC branch and run the publish workflow\n                  with the passcode to activate and download H@CKER M0D3."),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            }
        });

        // Back Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("BACK"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });
    });
}

pub fn cleanup_dlc_screen(mut commands: Commands, query: Query<Entity, With<DlcMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn dlc_screen_system(
    mut button_query: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
) {
    let mut back_pressed = keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Backspace);
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::East) || gamepad.just_pressed(GamepadButton::Start) {
            back_pressed = true;
        }
    }

    for (_, interaction, mut bg_color) in &mut button_query {
        if *interaction == Interaction::Hovered {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            back_pressed = true;
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }

    if back_pressed {
        next_state.set(AppState::TitleScreen);
    }
}
