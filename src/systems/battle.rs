use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::constants::{PLAYER_SIZE, GROUND_Y, GROUND_SIZE};

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
}

impl Default for BattleArenaState {
    fn default() -> Self {
        Self {
            score: 0,
            spawn_timer: 1.5,
            survival_time: 0.0,
            wave: 1,
            health: 10,
        }
    }
}

#[derive(Component)]
pub struct BattleEnemy {
    pub health: i32,
    pub speed: f32,
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
) {
    commands.insert_resource(BattleArenaState::default());
    custom_weapon.shoot_cooldown = 0.0;

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
                Text::new(format!("WEAPON: {} (Color: {})", custom_weapon.name, get_bullet_color_name(custom_weapon.color_idx))),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
            ));

            bar.spawn((
                Text::new("AIM: MOUSE CURSOR | SHOOT: LEFT-CLICK or F/J | JUMP: SPACE | BACK: ESC"),
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

    // Spawn the player specifically for the sandbox (will auto cleanup on exit)
    commands.spawn((
        Player,
        LevelEntity,
        Velocity(Vec2::ZERO),
        JumpState::default(),
        DashState::default(),
        GlitchState::default(),
        RamState::default(),
        Sprite::from_color(Color::srgb(0.48, 0.86, 0.62), PLAYER_SIZE),
        Transform::from_xyz(0.0, GROUND_Y, 1.0),
    ));
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
        let speed = 80.0 + (state.wave as f32) * 15.0;
        let health = 1 + (state.wave as i32) / 2; // Glitch health scales with waves

        commands.spawn((
            BattleEnemy { health, speed },
            // Bright red/magenta glitch cube enemy representation
            Sprite::from_color(Color::srgb(1.0, 0.1, 0.4), Vec2::new(24.0, 24.0)),
            Transform::from_xyz(x_pos, GROUND_Y, 1.0),
        ));
    }
}

// Enemy movement towards player
pub fn battle_arena_enemy_movement_system(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<BattleEnemy>)>,
    mut enemy_query: Query<(&mut Transform, &BattleEnemy)>,
) {
    let Ok(player_transform) = player_query.single() else { return; };
    let delta = time.delta_secs();

    for (mut transform, enemy) in &mut enemy_query {
        let player_x = player_transform.translation.x;
        let diff_x = player_x - transform.translation.x;
        let dir = diff_x.signum();
        transform.translation.x += dir * enemy.speed * delta;
    }
}

// Hit Collisions detection
pub fn battle_arena_collision_system(
    mut commands: Commands,
    mut state: ResMut<BattleArenaState>,
    mut next_state: ResMut<NextState<AppState>>,
    bullet_query: Query<(Entity, &Transform, &CustomBullet)>,
    mut enemy_query: Query<(Entity, &Transform, &mut BattleEnemy)>,
    mut player_query: Query<(&Transform, &mut RamState), With<Player>>,
) {
    // Bullet hitting enemies
    for (b_entity, b_trans, bullet) in &bullet_query {
        for (e_entity, e_trans, mut enemy) in &mut enemy_query {
            let dist = b_trans.translation.distance(e_trans.translation);
            if dist < 20.0 {
                // Inflict damage based on designed weapon
                enemy.health -= bullet.damage as i32;
                commands.entity(b_entity).despawn();

                if enemy.health <= 0 {
                    commands.entity(e_entity).despawn();
                    state.score += 10 * state.wave;
                }
            }
        }
    }

    // Enemies hitting player
    if let Ok((p_trans, mut ram_state)) = player_query.single_mut() {
        for (e_entity, e_trans, _) in &enemy_query {
            let dist = p_trans.translation.distance(e_trans.translation);
            if dist < 25.0 {
                // Hurt the player, despawn enemy
                commands.entity(e_entity).despawn();
                if state.health > 0 {
                    state.health -= 1;
                }
                ram_state.current = ram_state.current.saturating_sub(1);
                
                if state.health == 0 {
                    // System Crash! Battle Arena over, go back to designer
                    next_state.set(AppState::WeaponDesigner);
                }
            }
        }
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
        next_state.set(AppState::WeaponDesigner);
        return;
    }

    if let Ok(mut text) = score_query.single_mut() {
        text.0 = format!("GLITCH SCORE: {}  (WAVE {})", state.score, state.wave);
    }
    if let Ok(mut text) = timer_query.single_mut() {
        text.0 = format!("ELAPSED TIME: {:.2}s", state.survival_time);
    }
    if let Ok(mut text) = health_query.single_mut() {
        text.0 = format!("SYSTEM INTEGRITY: {}/10", state.health);
    }
}
