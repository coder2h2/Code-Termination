use bevy::prelude::*;
use crate::constants::*;
use crate::components::*;
use crate::resources::*;

pub fn save_achievements(achievements: &Achievements) {
    let content = format!(
        "{},{},{},{},{},{},{}",
        achievements.phase_shift,
        achievements.turbo_charged,
        achievements.airborne,
        achievements.system_crash,
        achievements.speed_daemon,
        achievements.better_call_tech_support,
        achievements.death_count
    );
    if let Err(e) = std::fs::write("achievements.txt", content) {
        eprintln!("Failed to save achievements: {:?}", e);
    }
}

pub fn load_achievements() -> Achievements {
    if let Ok(content) = std::fs::read_to_string("achievements.txt") {
        let parts: Vec<&str> = content.trim().split(',').collect();
        if parts.len() >= 5 {
            let phase_shift = parts[0].parse::<bool>().unwrap_or(false);
            let turbo_charged = parts[1].parse::<bool>().unwrap_or(false);
            let airborne = parts[2].parse::<bool>().unwrap_or(false);
            let system_crash = parts[3].parse::<bool>().unwrap_or(false);
            let speed_daemon = parts[4].parse::<bool>().unwrap_or(false);
            
            let better_call_tech_support = if parts.len() >= 7 {
                parts[5].parse::<bool>().unwrap_or(false)
            } else {
                false
            };
            
            let death_count = if parts.len() >= 7 {
                parts[6].parse::<u32>().unwrap_or(0)
            } else {
                0
            };

            return Achievements {
                phase_shift,
                turbo_charged,
                airborne,
                system_crash,
                speed_daemon,
                better_call_tech_support,
                death_count,
            };
        }
    }
    Achievements::default()
}

pub fn spawn_achievement_toast(
    commands: &mut Commands,
    toast_container: Entity,
    title: &str,
    desc: &str,
) {
    commands.entity(toast_container).with_children(|parent| {
        parent.spawn((
            AchievementToast { timer: 3.0 },
            Node {
                width: Val::Px(320.0),
                height: Val::Px(80.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.09, 0.14, 0.95)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
        ))
        .with_children(|toast| {
            toast.spawn((
                Text::new("▲ ACHIEVEMENT UNLOCKED ▲"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));
            toast.spawn((
                Text::new(title),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ));
            toast.spawn((
                Text::new(desc),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
    });
}

pub fn unlock_achievement(
    commands: &mut Commands,
    achievements: &mut ResMut<Achievements>,
    toast_container_query: &Query<Entity, With<ToastContainer>>,
    achievement_type: &str,
) {
    let mut unlocked = false;
    let mut title = "";
    let mut desc = "";

    match achievement_type {
        "phase_shift" => {
            if !achievements.phase_shift {
                achievements.phase_shift = true;
                unlocked = true;
                title = "Phase Shift";
                desc = "Glitch through a wall or spike.";
            }
        }
        "turbo_charged" => {
            if !achievements.turbo_charged {
                achievements.turbo_charged = true;
                unlocked = true;
                title = "Turbo Charged";
                desc = "Activate CPU Overclock.";
            }
        }
        "airborne" => {
            if !achievements.airborne {
                achievements.airborne = true;
                unlocked = true;
                title = "Airborne";
                desc = "Perform a double jump.";
            }
        }
        "system_crash" => {
            if !achievements.system_crash {
                achievements.system_crash = true;
                unlocked = true;
                title = "System Crash";
                desc = "Deplete memory to 0GB.";
            }
        }
        "speed_daemon" => {
            if !achievements.speed_daemon {
                achievements.speed_daemon = true;
                unlocked = true;
                title = "Speed Daemon";
                desc = "Perform a horizontal dash.";
            }
        }
        "better_call_tech_support" => {
            if !achievements.better_call_tech_support {
                achievements.better_call_tech_support = true;
                unlocked = true;
                title = "Better Call Tech Support!";
                desc = "Die ten times.";
            }
        }
        _ => {}
    }

    if unlocked {
        save_achievements(achievements);
        if let Ok(container) = toast_container_query.single() {
            spawn_achievement_toast(commands, container, title, desc);
        }
    }
}

#[cfg(unix)]
unsafe extern "C" {
    fn isatty(fd: i32) -> i32;
}

pub fn stdin_is_tty() -> bool {
    #[cfg(unix)]
    unsafe { isatty(0) != 0 }
    #[cfg(not(unix))]
    true
}

pub fn run_auto_update() {
    let repo = "coder2h2/Code-Termination";
    let current_commit = env!("GIT_HASH");
    
    if std::env::var("NO_AUTO_UPDATE").is_ok() {
        return;
    }

    // Only prompt if stdin is a terminal/TTY
    if !stdin_is_tty() {
        return;
    }

    // If we have a local DLC token (e.g. GITHUB_TOKEN or DLC_PAT env var), check and download dlc_signal.txt from the private repo
    if let Ok(token) = std::env::var("DLC_PAT").or_else(|_| std::env::var("GITHUB_TOKEN")) {
        let dlc_url = "https://raw.githubusercontent.com/coder2h2/Code-Termination-DLC/main/dlc_signal.txt";
        let dlc_output = std::process::Command::new("curl")
            .arg("-s")
            .arg("-m")
            .arg("2")
            .arg("-H")
            .arg(format!("Authorization: token {}", token))
            .arg(dlc_url)
            .output();
        if let Ok(output) = dlc_output {
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                if !content.trim().is_empty() && !content.contains("Not Found") {
                    if let Err(e) = std::fs::write("dlc_signal.txt", content.trim()) {
                        eprintln!("[Auto-Updater] Failed to write dlc_signal.txt: {:?}", e);
                    } else {
                        println!("[Auto-Updater] DLC signal file downloaded/updated successfully.");
                    }
                }
            }
        }
    }

    // Fetch the update signal first
    let signal_url = format!("https://raw.githubusercontent.com/{}/main/update_signal.txt", repo);
    let signal_output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-m")
        .arg("2")
        .arg(&signal_url)
        .output();

    let Ok(signal_output) = signal_output else { return; };
    if !signal_output.status.success() { return; }
    let signal_str = String::from_utf8_lossy(&signal_output.stdout);
    if !signal_str.contains("IDN:226531") {
        // Signal not found or incorrect, ignore update check
        return;
    }

    println!("[Auto-Updater] Active update signal detected. Checking for new updates...");
    
    // Fetch latest commit SHA from main branch
    let url = format!("https://api.github.com/repos/{}/commits/main", repo);
    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-m")
        .arg("2") // 2 seconds timeout
        .arg("-H")
        .arg("User-Agent: Code-Termination-Updater")
        .arg(&url)
        .output();

    let Ok(output) = output else { return; };
    if !output.status.success() { return; }
    
    let response = String::from_utf8_lossy(&output.stdout);
    let sha_keyword = "\"sha\":";
    let Some(sha_index) = response.find(sha_keyword) else { return; };
    let remaining = &response[sha_index + sha_keyword.len()..];
    let start_quote = remaining.find('"');
    let Some(start_quote) = start_quote else { return; };
    let end_quote = remaining[start_quote + 1..].find('"');
    let Some(end_quote) = end_quote else { return; };
    let latest_commit = &remaining[start_quote + 1..start_quote + 1 + end_quote];
    let latest_short = if latest_commit.len() >= 7 { &latest_commit[..7] } else { latest_commit };

    if latest_short != current_commit && current_commit != "unknown" && !latest_short.is_empty() {
        print!("[Auto-Updater] A new update (commit {}) is available. Would you like to update? [y/N]: ", latest_short);
        use std::io::Write;
        let _ = std::io::stdout().flush();
        
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let choice = input.trim().to_lowercase();
            if choice == "y" || choice == "yes" {
                println!("[Auto-Updater] Updating from source repository...");
                
                let repo_dir = env!("CARGO_MANIFEST_DIR");
                let is_git_clone = std::path::Path::new(repo_dir).join(".git").exists();
                
                let success = if is_git_clone {
                    println!("[Auto-Updater] Local git repository found at {}. Pulling latest changes...", repo_dir);
                    let pull_status = std::process::Command::new("git")
                        .arg("-C")
                        .arg(repo_dir)
                        .arg("pull")
                        .status();
                        
                    let pull_ok = pull_status.map(|s| s.success()).unwrap_or(false);
                    if !pull_ok {
                        eprintln!("[Auto-Updater] Failed to pull latest code.");
                        false
                    } else {
                        println!("[Auto-Updater] Compiling the new code in release mode...");
                        let build_status = std::process::Command::new("cargo")
                            .arg("build")
                            .arg("--release")
                            .current_dir(repo_dir)
                            .status();
                            
                        let build_ok = build_status.map(|s| s.success()).unwrap_or(false);
                        if build_ok {
                            let new_bin = std::path::Path::new(repo_dir).join("target/release/Code-Termination");
                            if let Ok(current_path) = std::env::current_exe() {
                                if let Err(e) = std::fs::copy(&new_bin, &current_path) {
                                    eprintln!("[Auto-Updater] Failed to copy new binary to path: {:?}", e);
                                    false
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        } else {
                            eprintln!("[Auto-Updater] Compilation failed.");
                            false
                        }
                    }
                } else {
                    println!("[Auto-Updater] Standalone binary run. Cloning repository to compile...");
                    let temp_dir = std::env::temp_dir().join("code-termination-build");
                    let _ = std::fs::remove_dir_all(&temp_dir);
                    
                    let clone_url = format!("https://github.com/{}.git", repo);
                    let clone_status = std::process::Command::new("git")
                        .arg("clone")
                        .arg(&clone_url)
                        .arg(&temp_dir)
                        .status();
                        
                    let clone_ok = clone_status.map(|s| s.success()).unwrap_or(false);
                    if !clone_ok {
                        eprintln!("[Auto-Updater] Failed to clone repository.");
                        false
                    } else {
                        println!("[Auto-Updater] Compiling source code...");
                        let build_status = std::process::Command::new("cargo")
                            .arg("build")
                            .arg("--release")
                            .current_dir(&temp_dir)
                            .status();
                            
                        let build_ok = build_status.map(|s| s.success()).unwrap_or(false);
                        if build_ok {
                            let new_bin = temp_dir.join("target/release/Code-Termination");
                            if let Ok(current_path) = std::env::current_exe() {
                                if let Err(e) = std::fs::copy(&new_bin, &current_path) {
                                    eprintln!("[Auto-Updater] Failed to copy binary: {:?}", e);
                                    false
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        } else {
                            eprintln!("[Auto-Updater] Compilation failed.");
                            false
                        }
                    }
                };
                
                if success {
                    println!("[Auto-Updater] Update applied successfully! Please restart the game.");
                    std::process::exit(0);
                } else {
                    println!("[Auto-Updater] Update failed. Starting current version of the game...");
                }
            }
        }
    }
}

pub fn has_dlc() -> bool {
    let paths = [
        std::path::PathBuf::from("dlc_signal.txt"),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("dlc_signal.txt"),
    ];
    for path in &paths {
        if path.exists() {
            return true;
        }
    }
    false
}

pub fn save_game(x: f32, y: f32, ram: u32, tutorial_visible: bool, level: u32, hacker_mode: bool) {
    let content = format!("{},{},{},{},{},{}", x, y, ram, tutorial_visible, level, hacker_mode);
    if let Err(e) = std::fs::write("savegame.txt", content) {
        eprintln!("Failed to save game: {:?}", e);
    }
}

pub fn load_game() -> Option<(f32, f32, u32, bool, u32, bool)> {
    if let Ok(content) = std::fs::read_to_string("savegame.txt") {
        let parts: Vec<&str> = content.trim().split(',').collect();
        if parts.len() >= 5 {
            let x = parts[0].parse::<f32>().unwrap_or(-350.0);
            let y = parts[1].parse::<f32>().unwrap_or(GROUND_Y);
            let ram = parts[2].parse::<u32>().unwrap_or(6);
            let tutorial_visible = parts[3].parse::<bool>().unwrap_or(true);
            let level = parts[4].parse::<u32>().unwrap_or(1);
            let hacker_mode = if parts.len() >= 6 {
                parts[5].parse::<bool>().unwrap_or(false)
            } else {
                false
            };
            return Some((x, y, ram, tutorial_visible, level, hacker_mode));
        }
        // Fallback for older saves
        if parts.len() == 4 {
            let x = parts[0].parse::<f32>().unwrap_or(-350.0);
            let y = parts[1].parse::<f32>().unwrap_or(GROUND_Y);
            let ram = parts[2].parse::<u32>().unwrap_or(6);
            let tutorial_visible = parts[3].parse::<bool>().unwrap_or(true);
            return Some((x, y, ram, tutorial_visible, 1, false));
        }
    }
    None
}

pub fn setup_game_hud(
    commands: &mut Commands,
    tutorial_visible: bool,
    current_level: u32,
    hacker_mode_active: bool,
) {
    if current_level == 4 {
        // Spawn Boss Health Bar container in the top-center
        commands.spawn((
            GameHUD,
            BossHealthBar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(30.0),
                right: Val::Percent(30.0),
                top: Val::Px(20.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|bar_parent| {
            // Boss Name Label
            bar_parent.spawn((
                Text::new("CORE BREACH PROCESS (LEVEL 4 SECURITY)"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));
            // Bar outer frame
            bar_parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(16.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.05, 0.0, 0.0)),
                BorderColor::all(Color::srgb(1.0, 0.0, 0.0)),
            ))
            .with_children(|frame_parent| {
                // Bar Fill
                frame_parent.spawn((
                    BossHealthBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.0, 0.3)), // Cyberpunk Hot Pink/Red
                ));
            });
        });
    }

    commands.spawn((
        GameHUD,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ))
    .with_children(|parent| {
        // Level Indicator
        parent.spawn((
            LevelHUD,
            Text::new(format!("ZONE: TUTORIAL [SECTOR 0{}]", current_level)),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 1.0)), // cyan for levels
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        if hacker_mode_active {
            parent.spawn((
                Text::new("H@CKER M0D3 ACTIVE"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));
        }

        if tutorial_visible {
            parent.spawn((
                TutorialHUD,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|tut_parent| {
                // Tutorial Title
                tut_parent.spawn((
                    Text::new("=== SYSTEM TUTORIAL ==="),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));

                let tutorial_lines = match current_level {
                    1 => vec![
                        "• MOVE: Use A/D or Arrow Keys",
                        "• JUMP: Space or Up Arrow (Double jump in mid-air)",
                        "• GLITCH: Press SHIFT or F to phase through the wall (UNLOCKED!)",
                        "• OVERCLOCK & UNDERCLOCK: LOCKED (Unlock at Level 2)",
                        "• DASH: LOCKED (Unlock at Level 3)",
                        "• OBJECTIVE: Glitch through the firewall wall and touch the cyan console",
                    ],
                    2 => vec![
                        "• MOVE: Use A/D or Arrow Keys",
                        "• JUMP: Space or Up Arrow",
                        "• GLITCH: Press SHIFT or F to phase through walls",
                        "• UNDERCLOCK: Press Z or E to slow down enemies (UNLOCKED!)",
                        "• OVERCLOCK: Press C or Q to speed up CPU clock (UNLOCKED!)",
                        "• DASH: LOCKED (Unlock at Level 3)",
                        "• OBJECTIVE: Underclock the fast enemy to slow them down, jump over them, and reach the console",
                    ],
                    3 => vec![
                        "• MOVE: Use A/D or Arrow Keys",
                        "• JUMP: Space or Up Arrow",
                        "• GLITCH: Press SHIFT or F to phase through walls",
                        "• UNDERCLOCK: Press Z or E to slow down enemies",
                        "• OVERCLOCK: Press C or Q to speed up CPU clock",
                        "• DASH: Double-tap A/D to dash horizontally and destroy enemies (UNLOCKED!)",
                        "• OBJECTIVE: Dash-kill the enemy, air-dash over the red spikes to the console",
                    ],
                    4 => vec![
                        "• FINAL CHALLENGE: All skills are fully unlocked!",
                        "• GLITCH: Phase through the vertical firewall wall at x = -150",
                        "• UNDERCLOCK: Press Z/E to slow down the fast patrolling process at x = 100",
                        "• OVERCLOCK: Press C/Q to speed up CPU clock and run/jump faster",
                        "• DASH: Double-tap to air-dash over the spike pit from x = 250 to 350",
                        "• OBJECTIVE: Combine all breach protocols to reach the final console at x = 500",
                    ],
                    _ => vec![],
                };

                for line in tutorial_lines {
                    tut_parent.spawn((
                        Text::new(line),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.0, 1.0, 0.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                }
            });
        }

        // System Memory Status (Health)
        parent.spawn((
            RamHUD,
            Text::new("SYSTEM MEMORY: 6GB / 6GB"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::top(Val::Px(15.0)),
                ..default()
            },
        ));

        // CPU Overclock Status HUD
        parent.spawn((
            OverclockHUD,
            Text::new("CPU CLOCK: 4.2GHz (READY - C: Overclock / Z: Underclock)"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ));
    });

    // Spawn Toast Container on the top-right
    commands.spawn((
        GameHUD,
        ToastContainer,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(10.0),
            ..default()
        },
    ));
}

pub fn load_level(
    level: u32,
    mut commands: Commands,
    level_entity_query: &Query<Entity, With<LevelEntity>>,
    player_query: &mut Query<(&mut Transform, &mut Velocity, &mut JumpState, &mut DashState, &mut GlitchState, &mut RamState), (With<Player>, Without<GatewayConsole>)>,
    hud_query: &Query<Entity, With<GameHUD>>,
    tutorial_state: &TutorialState,
    hacker_mode_active: bool,
) {
    // 1. Despawn old level entities
    for entity in level_entity_query {
        commands.entity(entity).despawn();
    }
    // 2. Despawn old HUD
    for entity in hud_query {
        commands.entity(entity).despawn();
    }
    // 3. Spawn new HUD
    setup_game_hud(&mut commands, tutorial_state.visible, level, hacker_mode_active);

    // 4. Reset player position and status
    for (mut transform, mut velocity, mut jump_state, mut dash_state, mut glitch_state, mut ram_state) in player_query {
        transform.translation = Vec3::new(-350.0, GROUND_Y, 1.0);
        velocity.0 = Vec2::ZERO;
        *jump_state = JumpState::default();
        *dash_state = DashState::default();
        *glitch_state = GlitchState::default();
        ram_state.current = ram_state.max; // Full RAM health
    }

    // Spawn the Floor platform
    commands.spawn((
        LevelEntity,
        Sprite::from_color(Color::srgb(0.12, 0.14, 0.19), Vec2::new(FLOOR_WIDTH, 40.0)),
        Transform::from_xyz(0.0, GROUND_Y - PLAYER_SIZE.y / 2.0 - 20.0, 0.5),
    ));


    // 5. Spawn new level entities
    match level {
        1 => {
            // Level 1: Wall at x = 0.0, Console at x = 350.0
            commands.spawn((
                LevelEntity,
                Wall,
                Sprite::from_color(Color::srgb(0.25, 0.28, 0.38), WALL_SIZE),
                Transform::from_xyz(0.0, -10.0, 1.0),
            ));

            commands.spawn((
                LevelEntity,
                GatewayConsole,
                Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::new(40.0, 60.0)),
                Transform::from_xyz(350.0, GROUND_Y, 1.0),
            ));
        }
        2 => {
            // Level 2: Wall at x = -100.0, Fast Enemy at x = 100.0, Console at x = 350.0
            commands.spawn((
                LevelEntity,
                Wall,
                Sprite::from_color(Color::srgb(0.25, 0.28, 0.38), WALL_SIZE),
                Transform::from_xyz(-100.0, -10.0, 1.0),
            ));

            commands.spawn((
                LevelEntity,
                Enemy {
                    patrol_left: 50.0,
                    patrol_right: 200.0,
                    speed: 250.0,
                    direction: 1.0,
                    is_destroyed: false,
                },
                Sprite::from_color(Color::srgb(1.0, 0.0, 0.5), Vec2::new(64.0, 64.0)),
                Transform::from_xyz(100.0, -128.0, 1.0),
            ));

            commands.spawn((
                LevelEntity,
                GatewayConsole,
                Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::new(40.0, 60.0)),
                Transform::from_xyz(350.0, GROUND_Y, 1.0),
            ));
        }
        3 => {
            // Level 3: Spikes at x = -48, 0, 48, Enemy at x = 200, Console at x = 380
            for sx in [-48.0, 0.0, 48.0] {
                commands.spawn((
                    LevelEntity,
                    Spike,
                    Sprite::from_color(Color::srgb(0.9, 0.1, 0.2), Vec2::new(48.0, 48.0)),
                    Transform::from_xyz(sx, -136.0, 1.0),
                ));
            }

            commands.spawn((
                LevelEntity,
                Enemy {
                    patrol_left: 150.0,
                    patrol_right: 300.0,
                    speed: 150.0,
                    direction: 1.0,
                    is_destroyed: false,
                },
                Sprite::from_color(Color::srgb(1.0, 0.0, 0.5), Vec2::new(64.0, 64.0)),
                Transform::from_xyz(200.0, -128.0, 1.0),
            ));

            commands.spawn((
                LevelEntity,
                GatewayConsole,
                Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::new(40.0, 60.0)),
                Transform::from_xyz(380.0, GROUND_Y, 1.0),
            ));
        }
        4 => {
            // Level 4: Final Challenge - Boss Fight Arena!
            // Spawning the BOSS (patrolling, double size, 3 HP, arena from -250 to 250)
            commands.spawn((
                LevelEntity,
                Enemy {
                    patrol_left: -250.0,
                    patrol_right: 250.0,
                    speed: 200.0,
                    direction: 1.0,
                    is_destroyed: false,
                },
                Boss {
                    health: if hacker_mode_active { 6 } else { 3 },
                    invulnerable_timer: 0.0,
                    state: BossAttackState::Intro,
                    state_timer: 2.0,
                    attack_cooldown_timer: 3.0,
                    ram_direction: 0.0,
                    sweep_target_x: 0.0,
                },
                Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::new(128.0, 128.0)),
                Transform::from_xyz(0.0, 400.0, 1.0),
            ))
            .with_children(|boss_parent| {
                boss_parent.spawn((
                    BossSpeechText,
                    Text::new("ACCESS DENIED"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.1, 0.1)),
                    Transform::from_xyz(0.0, 90.0, 2.0),
                ));
            });
        }
        _ => {}
    }
}

pub fn load_battle_high_score() -> u32 {
    if let Ok(content) = std::fs::read_to_string("battle_high_score.txt") {
        content.trim().parse::<u32>().unwrap_or(0)
    } else {
        0
    }
}

pub fn save_battle_high_score(score: u32) {
    let _ = std::fs::write("battle_high_score.txt", score.to_string());
}

pub fn load_user_profile() -> Option<(String, String)> {
    if let Ok(content) = std::fs::read_to_string("user_profile.txt") {
        let parts: Vec<&str> = content.trim().split(',').collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }
    None
}

pub fn save_user_profile(username: &str, uid: &str) {
    let content = format!("{},{}", username, uid);
    if let Err(e) = std::fs::write("user_profile.txt", content) {
        eprintln!("Failed to save user profile: {:?}", e);
    }
}

pub fn base64_encode(input: &str) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::with_capacity((bytes.len() + 2) / 3 * 4);
    
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        let b1 = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
        let b2 = if i + 2 < bytes.len() { bytes[i + 2] } else { 0 };
        
        let val = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        
        let c0 = CHARSET[((val >> 18) & 0x3F) as usize] as char;
        let c1 = CHARSET[((val >> 12) & 0x3F) as usize] as char;
        let c2 = CHARSET[((val >> 6) & 0x3F) as usize] as char;
        let c3 = CHARSET[(val & 0x3F) as usize] as char;
        
        result.push(c0);
        result.push(c1);
        if i + 1 < bytes.len() {
            result.push(c2);
        } else {
            result.push('=');
        }
        if i + 2 < bytes.len() {
            result.push(c3);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_and_uid_reversal() {
        let username = "GiddyUp15";
        let uid = base64_encode(username);
        assert_eq!(uid, "R2lkZHlVcDE1");
        
        save_user_profile(username, &uid);
        let loaded = load_user_profile().expect("Failed to load user profile");
        assert_eq!(loaded.0, username);
        assert_eq!(loaded.1, uid);
        
        let new_name = "PlayerTwo";
        let new_uid = base64_encode(new_name);
        save_user_profile(new_name, &new_uid);
        
        let loaded2 = load_user_profile().expect("Failed to load updated profile");
        assert_eq!(loaded2.0, new_name);
        assert_eq!(loaded2.1, new_uid);
        
        let _ = std::fs::remove_file("user_profile.txt");
    }
}
