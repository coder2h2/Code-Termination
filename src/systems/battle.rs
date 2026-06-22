use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::constants::{GROUND_Y, GROUND_SIZE};

// Adjectives and nouns for weapon name generation
const ADJECTIVES: &[&str] = &[
    "KERNEL", "SYS", "NULL", "GLITCH", "TROJAN", 
    "OVERCLOCKED", "STACK", "BINARY", "BUFFER", "LOGICAL",
    "CYBER", "VECTOR", "COMPILER", "DECRYPTED", "HOST"
];

const NOUNS: &[&str] = &[
    "BLASTER", "LAUNCHER", "SHOTGUN", "BEAM", "CRUSHER", 
    "DESTROYER", "CLEANER", "COMPILER", "DELETER", "TERMINATOR",
    "POINTER", "DECRYPTOR", "EXPLODER", "RAIDER", "CO-PROCESSOR"
];

fn generate_random_name() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let adj_idx = (seed % ADJECTIVES.len() as u128) as usize;
    let noun_idx = ((seed / 3) % NOUNS.len() as u128) as usize;
    format!("{}_{}", ADJECTIVES[adj_idx], NOUNS[noun_idx])
}

fn get_bullet_color(color_idx: u32) -> Color {
    match color_idx {
        0 => Color::srgb(0.0, 1.0, 1.0),      // Neon Cyan
        1 => Color::srgb(1.0, 0.0, 1.0),      // Neon Pink
        2 => Color::srgb(1.0, 1.0, 0.0),      // Cyber Yellow
        3 => Color::srgb(0.0, 1.0, 0.0),      // Hacker Green
        _ => Color::srgb(1.0, 1.0, 1.0),      // White
    }
}

#[allow(dead_code)]
fn get_bullet_color_name(color_idx: u32) -> &'static str {
    match color_idx {
        0 => "CYAN",
        1 => "PINK",
        2 => "YELLOW",
        3 => "GREEN",
        _ => "WHITE",
    }
}

// Resource to track the state of the Battle Arena sandbox
#[derive(Resource)]
pub struct BattleArenaState {
    pub score: u32,
    pub spawn_timer: f32,
    pub survival_time: f32,
    pub wave: u32,
    pub health: u32,
    pub high_score: u32,
}

impl Default for BattleArenaState {
    fn default() -> Self {
        Self {
            score: 0,
            spawn_timer: 1.5,
            survival_time: 0.0,
            wave: 1,
            health: 10,
            high_score: crate::helpers::load_battle_high_score(),
        }
    }
}

// --- WEAPON DESIGNER SCREEN ---

pub fn setup_weapon_designer(
    mut commands: Commands,
    mut menu_selection: ResMut<MenuSelection>,
    mut custom_weapon: ResMut<CustomWeapon>,
) {
    menu_selection.selected_index = 0;
    
    // Ensure we start with a fresh random name if default
    if custom_weapon.name == "KERNEL_BLASTER" {
        custom_weapon.name = generate_random_name();
    }

    commands.spawn((
        WeaponDesignerUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.06, 0.1)),
    ))
    .with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("=== WEAPON COMPILER & DESIGNER ==="),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        // Subtitle
        parent.spawn((
            Text::new("Allocate CPU clock cycles to optimize your fire parameters:"),
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

        // Weapon Name Container
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(30.0)),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new("WEAPON PROTOCOL: "),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            row.spawn((
                WeaponNameText,
                Text::new(custom_weapon.name.clone()),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node { margin: UiRect::right(Val::Px(20.0)), ..default() },
            ));
            // Random Name button
            row.spawn((
                Button,
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(35.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                WeaponDesignerButtonAction::RandomName,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("RE-GEN"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
        });

        // Stats Box
        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            width: Val::Px(550.0),
            margin: UiRect::bottom(Val::Px(30.0)),
            ..default()
        })
        .with_children(|grid| {
            // Row 1: Damage
            grid.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("DAMAGE STAT (2x budget)"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node { width: Val::Px(200.0), ..default() },
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::DecDamage,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("-"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
                row.spawn((
                    DamageValText,
                    Text::new(custom_weapon.damage_stat.to_string()),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 1.0)),
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::IncDamage,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("+"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
            });

            // Row 2: Cooldown
            grid.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("COOLDOWN STAT (2x budget)"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node { width: Val::Px(200.0), ..default() },
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::DecCooldown,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("-"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
                row.spawn((
                    CooldownValText,
                    Text::new(custom_weapon.cooldown_stat.to_string()),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 1.0)),
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::IncCooldown,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("+"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
            });

            // Row 3: Speed
            grid.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("BULLET SPEED (1x budget)"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node { width: Val::Px(200.0), ..default() },
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::DecSpeed,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("-"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
                row.spawn((
                    SpeedValText,
                    Text::new(custom_weapon.speed_stat.to_string()),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 1.0)),
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::IncSpeed,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("+"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
            });

            // Row 4: Multishot
            grid.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("MULTISHOT STAT (2x budget)"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node { width: Val::Px(200.0), ..default() },
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::DecMultishot,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("-"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
                row.spawn((
                    MultishotValText,
                    Text::new(custom_weapon.multishot_stat.to_string()),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 1.0)),
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(35.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::IncMultishot,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("+"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
            });

            // Row 5: Color
            grid.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("BULLET COLOR"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    Node { width: Val::Px(200.0), ..default() },
                ));
                row.spawn((
                    ColorBoxIndicator,
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(25.0),
                        ..default()
                    },
                    BackgroundColor(get_bullet_color(custom_weapon.color_idx)),
                ));
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                    WeaponDesignerButtonAction::ChangeColor,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("NEXT COLOR"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
            });
        });

        // Budget display
        parent.spawn((
            BudgetText,
            Text::new("CPU CYCLE BUDGET: 0 / 20"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
        ));

        // Compile and Back buttons
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            // Compile and Test
            row.spawn((
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::right(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.0, 1.0, 1.0)),
                WeaponDesignerButtonAction::CompileAndTest,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("COMPILE & TEST"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 1.0)),
                ));
            });

            // Back
            row.spawn((
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
                WeaponDesignerButtonAction::Back,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("BACK"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            });
        });
    });
}

pub fn cleanup_weapon_designer(
    mut commands: Commands,
    query: Query<Entity, With<WeaponDesignerUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn weapon_designer_button_system(
    mut button_query: Query<
        (&Interaction, &WeaponDesignerButtonAction, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut custom_weapon: ResMut<CustomWeapon>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let current_cost = custom_weapon.damage_stat * 2 
        + custom_weapon.cooldown_stat * 2 
        + custom_weapon.speed_stat 
        + custom_weapon.multishot_stat * 2;

    for (interaction, action, mut bg_color, mut border_color) in &mut button_query {
        // Disabled visual look for Compile if over budget
        if *action == WeaponDesignerButtonAction::CompileAndTest && current_cost > 20 {
            *bg_color = BackgroundColor(Color::srgb(0.08, 0.08, 0.08));
            *border_color = BorderColor::all(Color::srgb(0.3, 0.1, 0.1));
            continue;
        }

        if *interaction == Interaction::Hovered {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            match action {
                WeaponDesignerButtonAction::IncDamage => {
                    if custom_weapon.damage_stat < 10 {
                        custom_weapon.damage_stat += 1;
                    }
                }
                WeaponDesignerButtonAction::DecDamage => {
                    if custom_weapon.damage_stat > 1 {
                        custom_weapon.damage_stat -= 1;
                    }
                }
                WeaponDesignerButtonAction::IncCooldown => {
                    if custom_weapon.cooldown_stat < 10 {
                        custom_weapon.cooldown_stat += 1;
                    }
                }
                WeaponDesignerButtonAction::DecCooldown => {
                    if custom_weapon.cooldown_stat > 1 {
                        custom_weapon.cooldown_stat -= 1;
                    }
                }
                WeaponDesignerButtonAction::IncSpeed => {
                    if custom_weapon.speed_stat < 10 {
                        custom_weapon.speed_stat += 1;
                    }
                }
                WeaponDesignerButtonAction::DecSpeed => {
                    if custom_weapon.speed_stat > 1 {
                        custom_weapon.speed_stat -= 1;
                    }
                }
                WeaponDesignerButtonAction::IncMultishot => {
                    if custom_weapon.multishot_stat < 5 {
                        custom_weapon.multishot_stat += 1;
                    }
                }
                WeaponDesignerButtonAction::DecMultishot => {
                    if custom_weapon.multishot_stat > 1 {
                        custom_weapon.multishot_stat -= 1;
                    }
                }
                WeaponDesignerButtonAction::ChangeColor => {
                    custom_weapon.color_idx = (custom_weapon.color_idx + 1) % 4;
                }
                WeaponDesignerButtonAction::RandomName => {
                    custom_weapon.name = generate_random_name();
                }
                WeaponDesignerButtonAction::CompileAndTest => {
                    if current_cost <= 20 {
                        next_state.set(AppState::BattleArena);
                    }
                }
                WeaponDesignerButtonAction::Back => {
                    next_state.set(AppState::TitleScreen);
                }
            }
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
            if *action == WeaponDesignerButtonAction::CompileAndTest {
                *border_color = BorderColor::all(Color::srgb(0.0, 1.0, 1.0));
            } else if *action == WeaponDesignerButtonAction::Back {
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.5));
            } else {
                *border_color = BorderColor::all(Color::srgb(0.3, 0.3, 0.3));
            }
        }
    }
}

pub fn update_weapon_designer_ui(
    custom_weapon: Res<CustomWeapon>,
    mut name_query: Query<&mut Text, (With<WeaponNameText>, Without<DamageValText>, Without<BudgetText>)>,
    mut budget_query: Query<(&mut Text, &mut TextColor), (With<BudgetText>, Without<DamageValText>, Without<WeaponNameText>)>,
    mut damage_query: Query<&mut Text, (With<DamageValText>, Without<WeaponNameText>, Without<BudgetText>)>,
    mut cooldown_query: Query<&mut Text, (With<CooldownValText>, Without<WeaponNameText>, Without<BudgetText>, Without<DamageValText>)>,
    mut speed_query: Query<&mut Text, (With<SpeedValText>, Without<WeaponNameText>, Without<BudgetText>, Without<DamageValText>, Without<CooldownValText>)>,
    mut multishot_query: Query<&mut Text, (With<MultishotValText>, Without<WeaponNameText>, Without<BudgetText>, Without<DamageValText>, Without<CooldownValText>, Without<SpeedValText>)>,
    mut color_indicator_query: Query<&mut BackgroundColor, (With<ColorBoxIndicator>, Without<Button>)>,
) {
    if let Ok(mut text) = name_query.single_mut() {
        text.0 = custom_weapon.name.clone();
    }
    
    let cost = custom_weapon.damage_stat * 2 
        + custom_weapon.cooldown_stat * 2 
        + custom_weapon.speed_stat 
        + custom_weapon.multishot_stat * 2;
        
    if let Ok((mut text, mut color)) = budget_query.single_mut() {
        text.0 = format!("CPU CYCLE BUDGET: {} / 20", cost);
        if cost > 20 {
            color.0 = Color::srgb(1.0, 0.2, 0.2); // Red if over budget
        } else {
            color.0 = Color::srgb(0.0, 1.0, 0.0); // Green if OK
        }
    }
    
    if let Ok(mut text) = damage_query.single_mut() {
        text.0 = custom_weapon.damage_stat.to_string();
    }
    if let Ok(mut text) = cooldown_query.single_mut() {
        text.0 = format!("{}  (Fire Delay: {:.2}s)", custom_weapon.cooldown_stat, 1.65 - (custom_weapon.cooldown_stat as f32) * 0.15);
    }
    if let Ok(mut text) = speed_query.single_mut() {
        text.0 = format!("{}  (Speed: {:.0}px/s)", custom_weapon.speed_stat, 200.0 + (custom_weapon.speed_stat as f32) * 100.0);
    }
    if let Ok(mut text) = multishot_query.single_mut() {
        text.0 = format!("{}  (Spread Bullets)", custom_weapon.multishot_stat);
    }
    
    if let Ok(mut bg_color) = color_indicator_query.single_mut() {
        *bg_color = BackgroundColor(get_bullet_color(custom_weapon.color_idx));
    }
}

// --- BATTLE ARENA SANDBOX ---

pub fn setup_battle_arena(
    mut commands: Commands,
    mut custom_weapon: ResMut<CustomWeapon>,
    mut player_query: Query<(
        &mut Transform,
        &mut Velocity,
        &mut JumpState,
        &mut DashState,
        &mut GlitchState,
        &mut RamState,
        &mut Sprite,
    ), With<Player>>,
) {
    commands.insert_resource(BattleArenaState::default());
    custom_weapon.shoot_cooldown = 0.0;

    // Reset the existing persistent player specifically for the sandbox
    for (
        mut transform,
        mut velocity,
        mut jump_state,
        mut dash_state,
        mut glitch_state,
        mut ram_state,
        mut sprite,
    ) in &mut player_query {
        transform.translation = Vec3::new(0.0, GROUND_Y, 1.0);
        velocity.0 = Vec2::ZERO;
        *jump_state = JumpState::default();
        *dash_state = DashState::default();
        *glitch_state = GlitchState::default();
        *ram_state = RamState::default();
        sprite.color = Color::srgb(0.48, 0.86, 0.62);
    }

    // Arena Background
    commands.spawn((
        BattleArenaUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
    ))
    .with_children(|parent| {
        // Top HUD bar
        parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|hud| {
            hud.spawn((
                BattleScoreText,
                Text::new("GLITCH SCORE: 0"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));

            hud.spawn((
                BattleTimerText,
                Text::new("ELAPSED TIME: 0.00s"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            hud.spawn((
                BattleHealthText,
                Text::new("SYSTEM INTEGRITY: 10/10"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
            ));
        });

        // Bottom status instruction bar
        parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|bar| {
            bar.spawn((
                BattleWeaponNameText,
                Text::new("ATTACK PROTOCOL: DASH & SMASHDOWN"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.0, 1.0, 1.0)),
            ));

            bar.spawn((
                Text::new("ATTACK: DASH (Double-tap A/D, Shift, or RMB/Trigger) | SMASHDOWN (S/Arrow-Down in air) | BACK: ESC"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
    });

    // Floor platform (centered in the view)
    commands.spawn((
        LevelEntity,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(10.0),
            right: Val::Percent(10.0),
            bottom: Val::Px(0.0),
            height: Val::Px(GROUND_SIZE.y),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.25)), // Cyberpunk grid texture style floor
    ));

    // Spawn Pillars (Walls that block horizontal movement)
    commands.spawn((
        LevelEntity,
        Wall,
        Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::new(40.0, 160.0)),
        Transform::from_xyz(-250.0, GROUND_Y + 80.0, 1.0),
    ));

    commands.spawn((
        LevelEntity,
        Wall,
        Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::new(40.0, 160.0)),
        Transform::from_xyz(250.0, GROUND_Y + 80.0, 1.0),
    ));

    // Spawn Hazard Laser
    commands.spawn((
        LevelEntity,
        BattleHazardLaser {
            active_timer: 3.0,
            is_active: false,
        },
        Sprite::from_color(Color::srgba(1.0, 0.1, 0.1, 0.15), Vec2::new(450.0, 6.0)),
        Transform::from_xyz(0.0, GROUND_Y + 120.0, 1.0),
    ));

    commands.insert_resource(PlayerPowerUpState::default());
}

pub fn cleanup_battle_arena(
    mut commands: Commands,
    ui_query: Query<Entity, With<BattleArenaUI>>,
    entity_query: Query<Entity, With<LevelEntity>>,
    bullet_query: Query<Entity, With<CustomBullet>>,
    enemy_query: Query<Entity, With<BattleEnemy>>,
) {
    commands.remove_resource::<BattleArenaState>();

    for entity in &ui_query {
        commands.entity(entity).despawn();
    }
    for entity in &entity_query {
        commands.entity(entity).despawn();
    }
    for entity in &bullet_query {
        commands.entity(entity).despawn();
    }
    for entity in &enemy_query {
        commands.entity(entity).despawn();
    }
}

// Shooting system
#[allow(dead_code)]
pub fn battle_arena_shooting_system(
    mut commands: Commands,
    time: Res<Time>,
    mut custom_weapon: ResMut<CustomWeapon>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<&Transform, With<Player>>,
) {
    if custom_weapon.shoot_cooldown > 0.0 {
        custom_weapon.shoot_cooldown -= time.delta_secs();
        if custom_weapon.shoot_cooldown < 0.0 {
            custom_weapon.shoot_cooldown = 0.0;
        }
    }

    let Ok(player_transform) = player_query.single() else { return; };
    let click = mouse.pressed(MouseButton::Left) 
        || keyboard.pressed(KeyCode::KeyF) 
        || keyboard.pressed(KeyCode::KeyJ);

    if click && custom_weapon.shoot_cooldown == 0.0 {
        // Calculate shooting direction towards mouse cursor in world space
        let mut shoot_direction = Vec2::new(1.0, 0.0); // Fallback rightwards
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_query.single() {
                    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                        let player_pos = player_transform.translation.truncate();
                        let delta = world_pos - player_pos;
                        if delta.length_squared() > 1.0 {
                            shoot_direction = delta.normalize();
                        }
                    }
                }
            }
        }

        // Apply weapon parameters
        let bullets_count = custom_weapon.multishot_stat;
        let speed = 200.0 + (custom_weapon.speed_stat as f32) * 100.0;
        let damage = custom_weapon.damage_stat;
        let bullet_color = get_bullet_color(custom_weapon.color_idx);
        let start_pos = player_transform.translation + Vec3::new(0.0, 5.0, 0.5);

        // Spawn spread bullets
        for i in 0..bullets_count {
            // Spread angle calculation: center is shoot_direction
            let angle_offset = if bullets_count > 1 {
                let step = 0.25; // approx 15 degrees in radians
                let start = -(bullets_count as f32 - 1.0) * step / 2.0;
                start + (i as f32) * step
            } else {
                0.0
            };

            // Rotate vector by angle_offset
            let cos_a = angle_offset.cos();
            let sin_a = angle_offset.sin();
            let bullet_dir = Vec2::new(
                shoot_direction.x * cos_a - shoot_direction.y * sin_a,
                shoot_direction.x * sin_a + shoot_direction.y * cos_a
            ).normalize();

            // Spawn bullet entity
            commands.spawn((
                CustomBullet {
                    damage,
                    velocity: bullet_dir * speed,
                },
                Sprite::from_color(bullet_color, Vec2::new(15.0, 8.0)), // Custom Bullet sprite
                Transform::from_translation(start_pos)
                    .with_rotation(Quat::from_rotation_z(bullet_dir.y.atan2(bullet_dir.x))),
            ));
        }

        // Set cooldown: higher cooldown_stat = lower delay
        custom_weapon.shoot_cooldown = 1.65 - (custom_weapon.cooldown_stat as f32) * 0.15;
    }
}

// Bullet movement
#[allow(dead_code)]
pub fn bullet_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut bullet_query: Query<(Entity, &mut Transform, &CustomBullet)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, bullet) in &mut bullet_query {
        transform.translation.x += bullet.velocity.x * delta;
        transform.translation.y += bullet.velocity.y * delta;

        // Despawn bullet if it flies off-screen
        if transform.translation.x.abs() > 800.0 || transform.translation.y.abs() > 800.0 {
            commands.entity(entity).despawn();
        }
    }
}

// Enemy Spawner waves
pub fn battle_arena_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<BattleArenaState>,
) {
    state.survival_time += time.delta_secs();
    
    // Wave updates every 20 seconds
    state.wave = 1 + (state.survival_time / 20.0) as u32;

    // Spawn timer scaling down with waves
    let spawn_rate = (1.5 - (state.wave as f32 - 1.0) * 0.2).max(0.4);
    
    state.spawn_timer -= time.delta_secs();
    if state.spawn_timer <= 0.0 {
        state.spawn_timer = spawn_rate;

        // Spawn a glitch enemy on a random side
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let left_side = (seed % 2) == 0;
        let x_pos = if left_side { -450.0 } else { 450.0 };
        
        let enemy_rand = (seed / 100) % 100;
        let (enemy_type, health, speed, color) = if enemy_rand < 60 {
            // Walker
            (EnemyType::Walker, 1 + (state.wave as i32) / 2, 80.0 + (state.wave as f32) * 15.0, Color::srgb(1.0, 0.1, 0.4))
        } else if enemy_rand < 85 {
            // Shooter
            (EnemyType::Shooter, 1 + (state.wave as i32) / 3, 60.0 + (state.wave as f32) * 10.0, Color::srgb(0.0, 0.8, 1.0))
        } else {
            // Charger
            (EnemyType::Charger, 2 + (state.wave as i32) / 2, 90.0 + (state.wave as f32) * 12.0, Color::srgb(1.0, 0.8, 0.0))
        };

        commands.spawn((
            BattleEnemy {
                health,
                speed,
                enemy_type,
                state_timer: 1.5,
                charge_dir: 0.0,
                is_charging: false,
            },
            Sprite::from_color(color, Vec2::new(24.0, 24.0)),
            Transform::from_xyz(x_pos, GROUND_Y, 1.0),
        ));
    }
}

// Enemy movement towards player
pub fn battle_arena_enemy_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<BattleEnemy>, Without<EnemyProjectile>)>,
    mut enemy_query: Query<(&mut Transform, &mut BattleEnemy, &mut Sprite)>,
) {
    let Ok(player_transform) = player_query.single() else { return; };
    let delta = time.delta_secs();
    let player_x = player_transform.translation.x;

    for (mut transform, mut enemy, mut sprite) in &mut enemy_query {
        match enemy.enemy_type {
            EnemyType::Walker => {
                let diff_x = player_x - transform.translation.x;
                let dir = diff_x.signum();
                transform.translation.x += dir * enemy.speed * delta;
            }
            EnemyType::Shooter => {
                let diff_x = player_x - transform.translation.x;
                let dist_x = diff_x.abs();
                
                // Move towards player until within shooting range
                if dist_x > 260.0 {
                    let dir = diff_x.signum();
                    transform.translation.x += dir * enemy.speed * delta;
                } else if dist_x < 180.0 {
                    // Retract if player is too close
                    let dir = -diff_x.signum();
                    transform.translation.x += dir * enemy.speed * delta;
                }
                
                // Handle shooting
                enemy.state_timer -= delta;
                if enemy.state_timer <= 0.0 {
                    enemy.state_timer = 2.0; // Shoot every 2 seconds
                    
                    // Spawn a projectile package
                    let direction = diff_x.signum();
                    let velocity = Vec2::new(direction * 180.0, 0.0);
                    
                    commands.spawn((
                        LevelEntity,
                        EnemyProjectile { velocity },
                        Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::new(12.0, 12.0)),
                        Transform::from_translation(transform.translation + Vec3::new(0.0, 0.0, 0.5)),
                    ));
                }
            }
            EnemyType::Charger => {
                if enemy.is_charging {
                    // Move very fast in the charge direction
                    transform.translation.x += enemy.charge_dir * enemy.speed * 2.8 * delta;
                    enemy.state_timer -= delta;
                    if enemy.state_timer <= 0.0 {
                        enemy.is_charging = false;
                        enemy.state_timer = 2.5; // Cooldown before next charge
                        sprite.color = Color::srgb(1.0, 0.8, 0.0); // Reset to Yellow
                    }
                } else {
                    // Move normally towards player
                    let diff_x = player_x - transform.translation.x;
                    let dir = diff_x.signum();
                    transform.translation.x += dir * enemy.speed * 0.8 * delta;
                    
                    enemy.state_timer -= delta;
                    if enemy.state_timer <= 0.0 {
                        // Telegraph charge! Stop and turn bright orange/red
                        enemy.is_charging = true;
                        enemy.charge_dir = dir;
                        enemy.state_timer = 1.2; // Charge for 1.2 seconds
                        sprite.color = Color::srgb(1.0, 0.2, 0.0); // Warning Red-Orange
                    }
                }
            }
        }
    }
}

// Hit Collisions detection
pub fn battle_arena_collision_system(
    mut commands: Commands,
    mut state: ResMut<BattleArenaState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut enemy_query: Query<(Entity, &Transform, &mut BattleEnemy)>,
    mut player_query: Query<(&Transform, &mut RamState, &DashState, &JumpState), With<Player>>,
    proj_query: Query<(Entity, &Transform), With<EnemyProjectile>>,
    mut shake: ResMut<ScreenShake>,
    player_powerup_state: Res<PlayerPowerUpState>,
) {
    let Ok((p_trans, mut ram_state, dash_state, jump_state)) = player_query.single_mut() else { return; };
    let is_dashing = dash_state.dash_timer > 0.0;
    let is_smashing = jump_state.is_smashing;
    let is_attacking = is_dashing || is_smashing;
    let has_shield = player_powerup_state.shield_timer > 0.0;
    let is_invulnerable = is_attacking || has_shield || ram_state.invulnerability_timer > 0.0;

    let p_pos = p_trans.translation;

    // Enemy melee hits
    for (e_entity, e_trans, mut enemy) in &mut enemy_query {
        let dist = p_pos.distance(e_trans.translation);
        if dist < 30.0 {
            if is_attacking {
                // Inflict damage based on attack mode
                let damage = if is_smashing { 3 } else { 1 };
                enemy.health -= damage;

                if enemy.health <= 0 {
                    commands.entity(e_entity).despawn();
                    state.score += 10 * state.wave;
                    
                    // Spawn particles
                    spawn_glitch_particles(&mut commands, e_trans.translation, 12);
                    
                    // Shake screen
                    shake.intensity = 5.0;
                    shake.duration = 0.2;
                    
                    // Drop PowerUp (25% chance)
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
                    if (seed % 100) < 25 {
                        let powerup_type = match (seed / 7) % 3 {
                            0 => PowerUpType::HealthRecovery,
                            1 => PowerUpType::OverclockBoost,
                            _ => PowerUpType::GlitchShield,
                        };
                        
                        let color = match powerup_type {
                            PowerUpType::HealthRecovery => Color::srgb(1.0, 0.0, 0.0), // Red
                            PowerUpType::OverclockBoost => Color::srgb(1.0, 0.8, 0.0), // Gold
                            PowerUpType::GlitchShield => Color::srgb(0.0, 0.5, 1.0),    // Blue
                        };
                        
                        commands.spawn((
                            LevelEntity,
                            PowerUp {
                                power_up_type: powerup_type,
                                duration: 8.0,
                            },
                            Sprite::from_color(color, Vec2::new(16.0, 16.0)),
                            Transform::from_translation(e_trans.translation + Vec3::new(0.0, 10.0, 0.5)),
                        ));
                    }
                }
            } else if !has_shield && ram_state.invulnerability_timer <= 0.0 {
                // Hurt the player, despawn enemy
                commands.entity(e_entity).despawn();
                if state.health > 0 {
                    state.health -= 1;
                }
                ram_state.current = ram_state.current.saturating_sub(1);
                ram_state.invulnerability_timer = 1.0;
                
                // Shake screen
                shake.intensity = 10.0;
                shake.duration = 0.3;
                
                if state.health == 0 {
                    next_state.set(AppState::TitleScreen);
                }
            }
        }
    }

    // Enemy projectile hits
    for (proj_entity, proj_trans) in &proj_query {
        let dist = p_pos.distance(proj_trans.translation);
        if dist < 22.0 {
            commands.entity(proj_entity).despawn();
            if !is_invulnerable {
                if state.health > 0 {
                    state.health -= 1;
                }
                ram_state.current = ram_state.current.saturating_sub(1);
                ram_state.invulnerability_timer = 1.0;
                
                shake.intensity = 10.0;
                shake.duration = 0.3;
                
                if state.health == 0 {
                    next_state.set(AppState::TitleScreen);
                }
            }
        }
    }

    // High Score updates
    if state.score > state.high_score {
        state.high_score = state.score;
        crate::helpers::save_battle_high_score(state.high_score);
    }
}

// HUD update
pub fn battle_arena_ui_update_system(
    state: Res<BattleArenaState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut score_query: Query<&mut Text, (With<BattleScoreText>, Without<BattleTimerText>, Without<BattleHealthText>)>,
    mut timer_query: Query<&mut Text, (With<BattleTimerText>, Without<BattleScoreText>, Without<BattleHealthText>)>,
    mut health_query: Query<&mut Text, (With<BattleHealthText>, Without<BattleScoreText>, Without<BattleTimerText>)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::TitleScreen);
        return;
    }

    if let Ok(mut text) = score_query.single_mut() {
        text.0 = format!("SCORE: {} (HI: {}) | WAVE {}", state.score, state.high_score, state.wave);
    }
    if let Ok(mut text) = timer_query.single_mut() {
        text.0 = format!("ELAPSED TIME: {:.2}s", state.survival_time);
    }
    if let Ok(mut text) = health_query.single_mut() {
        text.0 = format!("SYSTEM INTEGRITY: {}/10", state.health);
    }
}

// Glitch Particle Emitter helper
pub fn spawn_glitch_particles(
    commands: &mut Commands,
    pos: Vec3,
    count: usize,
) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    
    for _ in 0..count {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let angle = (seed % 360) as f32 * std::f32::consts::PI / 180.0;
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let speed = 100.0 + (seed % 150) as f32;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
        
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let lifetime = 0.3 + (seed % 4) as f32 * 0.1;
        
        let color_base = match seed % 4 {
            0 => (0.0, 1.0, 1.0),      // Cyan
            1 => (1.0, 0.0, 1.0),      // Pink
            2 => (0.0, 1.0, 0.0),      // Lime
            _ => (1.0, 1.0, 0.0),      // Yellow
        };
        
        commands.spawn((
            LevelEntity,
            GlitchParticle {
                velocity,
                timer: lifetime,
                initial_timer: lifetime,
                color_base,
            },
            Sprite::from_color(
                Color::srgba(color_base.0, color_base.1, color_base.2, 1.0),
                Vec2::new(10.0, 10.0)
            ),
            Transform::from_translation(pos),
        ));
    }
}

// Particle updates
pub fn particle_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut GlitchParticle, &mut Sprite)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut particle, mut sprite) in &mut query {
        particle.timer -= delta;
        if particle.timer <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity.extend(0.0) * delta;
            let alpha = (particle.timer / particle.initial_timer).clamp(0.0, 1.0);
            sprite.color = Color::srgba(
                particle.color_base.0,
                particle.color_base.1,
                particle.color_base.2,
                alpha
            );
        }
    }
}

// Camera Screenshake updater
pub fn camera_shake_system(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let delta = time.delta_secs();
    if shake.duration > 0.0 {
        shake.duration -= delta;
        if shake.duration <= 0.0 {
            shake.duration = 0.0;
            shake.intensity = 0.0;
            for mut trans in &mut camera_query {
                trans.translation.x = 0.0;
                trans.translation.y = 0.0;
            }
        } else {
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
            let angle = (seed % 360) as f32 * std::f32::consts::PI / 180.0;
            let offset_x = angle.cos() * shake.intensity;
            let offset_y = angle.sin() * shake.intensity;
            for mut trans in &mut camera_query {
                trans.translation.x = offset_x;
                trans.translation.y = offset_y;
            }
        }
    }
}

// Enemy projectile physical moves
pub fn enemy_projectile_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &EnemyProjectile)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, projectile) in &mut query {
        transform.translation.x += projectile.velocity.x * delta;
        if transform.translation.x.abs() > 800.0 {
            commands.entity(entity).despawn();
        }
    }
}

// Hazard lasers logic
pub fn battle_hazard_laser_system(
    time: Res<Time>,
    mut query: Query<(&mut BattleHazardLaser, &mut Sprite, &Transform)>,
    mut player_query: Query<(&Transform, &mut RamState, &DashState, &JumpState), (With<Player>, Without<BattleHazardLaser>)>,
    mut state: ResMut<BattleArenaState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut shake: ResMut<ScreenShake>,
) {
    let delta = time.delta_secs();
    let mut active_laser_y = None;
    let mut active_laser_left = 0.0;
    let mut active_laser_right = 0.0;
    
    for (mut laser, mut sprite, transform) in &mut query {
        laser.active_timer -= delta;
        if laser.active_timer <= 0.0 {
            laser.is_active = !laser.is_active;
            laser.active_timer = if laser.is_active { 2.5 } else { 3.5 };
        }
        
        if laser.is_active {
            let pulse = (time.elapsed_secs() * 25.0).sin().abs() * 0.4 + 0.6;
            sprite.color = Color::srgba(1.0, 0.0, 0.0, pulse);
            
            let x = transform.translation.x;
            let len = 450.0;
            active_laser_y = Some(transform.translation.y);
            active_laser_left = x - len / 2.0;
            active_laser_right = x + len / 2.0;
        } else {
            let blink = (time.elapsed_secs() * 3.0).sin().abs() * 0.1 + 0.05;
            sprite.color = Color::srgba(1.0, 0.0, 0.0, blink);
        }
    }
    
    if let Some(laser_y) = active_laser_y {
        if let Ok((p_trans, mut ram_state, dash_state, jump_state)) = player_query.single_mut() {
            let is_invulnerable = dash_state.dash_timer > 0.0 || jump_state.is_smashing || ram_state.invulnerability_timer > 0.0;
            if !is_invulnerable {
                let px = p_trans.translation.x;
                let py = p_trans.translation.y;
                let p_bottom = py - 48.0;
                let p_top = py + 48.0;
                
                if px >= active_laser_left && px <= active_laser_right 
                    && laser_y >= p_bottom && laser_y <= p_top 
                {
                    if state.health > 0 {
                        state.health -= 1;
                    }
                    ram_state.current = ram_state.current.saturating_sub(1);
                    ram_state.invulnerability_timer = 1.0;
                    shake.intensity = 10.0;
                    shake.duration = 0.3;
                    
                    if state.health == 0 {
                        next_state.set(AppState::TitleScreen);
                    }
                }
            }
        }
    }
}

// Powerup pickups & durations
pub fn powerup_pickup_system(
    mut commands: Commands,
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut RamState, &mut Sprite, &mut DashState), (With<Player>, Without<PowerUp>)>,
    mut powerup_query: Query<(Entity, &Transform, &mut PowerUp)>,
    mut state: ResMut<BattleArenaState>,
    mut player_powerup_state: ResMut<PlayerPowerUpState>,
) {
    let delta = time.delta_secs();
    let Ok((p_trans, mut ram_state, mut sprite, mut dash_state)) = player_query.single_mut() else { return; };
    
    if player_powerup_state.overclock_timer > 0.0 {
        player_powerup_state.overclock_timer -= delta;
        if player_powerup_state.overclock_timer <= 0.0 {
            player_powerup_state.overclock_timer = 0.0;
            sprite.color = Color::srgb(0.48, 0.86, 0.62);
        } else {
            let pulse = (time.elapsed_secs() * 15.0).sin().abs() * 0.4 + 0.6;
            sprite.color = Color::srgb(1.0 * pulse, 0.8 * pulse, 0.0);
            dash_state.dash_timer = 0.0;
            dash_state.air_dash_used = false;
        }
    }
    
    if player_powerup_state.shield_timer > 0.0 {
        player_powerup_state.shield_timer -= delta;
        if player_powerup_state.shield_timer <= 0.0 {
            player_powerup_state.shield_timer = 0.0;
            if player_powerup_state.overclock_timer == 0.0 {
                sprite.color = Color::srgb(0.48, 0.86, 0.62);
            }
        } else {
            if player_powerup_state.overclock_timer == 0.0 {
                let pulse = (time.elapsed_secs() * 15.0).sin().abs() * 0.4 + 0.6;
                sprite.color = Color::srgb(0.0, 0.5 * pulse, 1.0 * pulse);
            }
        }
    }

    for (entity, trans, mut powerup) in &mut powerup_query {
        powerup.duration -= delta;
        if powerup.duration <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        
        let dist = p_trans.translation.distance(trans.translation);
        if dist < 30.0 {
            commands.entity(entity).despawn();
            match powerup.power_up_type {
                PowerUpType::HealthRecovery => {
                    state.health = (state.health + 2).min(10);
                    ram_state.current = (ram_state.current + 2).min(ram_state.max);
                }
                PowerUpType::OverclockBoost => {
                    player_powerup_state.overclock_timer = 5.0;
                }
                PowerUpType::GlitchShield => {
                    player_powerup_state.shield_timer = 5.0;
                }
            }
        }
    }
}
