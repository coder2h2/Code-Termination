use bevy::prelude::*;
use crate::constants::*;
use crate::components::*;
use crate::resources::*;
use crate::helpers::{unlock_achievement, load_level, save_game};

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawning Ground
    commands.spawn((
        LevelEntity,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            height: Val::Px(GROUND_SIZE.y),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.14, 0.19)),
    ));

    // Spawning Player
    commands.spawn((
        Player,
        Velocity(Vec2::ZERO),
        JumpState::default(),
        DashState::default(),
        GlitchState::default(),
        RamState::default(),
        Sprite::from_color(Color::srgb(0.48, 0.86, 0.62), PLAYER_SIZE),
        Transform::from_xyz(-350.0, GROUND_Y, 1.0),
    ));
}

pub fn reset_virtual_time_system(
    mut virtual_time: ResMut<Time<Virtual>>,
    mut overclock: ResMut<OverclockState>,
) {
    virtual_time.set_relative_speed(1.0);
    *overclock = OverclockState::default();
}

pub fn auto_save_system(
    time: Res<Time>,
    mut timer: Local<f32>,
    player_query: Query<(&Transform, &RamState), With<Player>>,
    tutorial_state: Res<TutorialState>,
    level_state: Res<LevelState>,
    hacker_mode: Res<HackerMode>,
) {
    *timer += time.delta_secs();
    if *timer >= 2.0 {
        *timer = 0.0;
        if let Ok((transform, ram_state)) = player_query.single() {
            let ram_to_save = if ram_state.current == 0 {
                ram_state.max
            } else {
                ram_state.current
            };
            save_game(
                transform.translation.x,
                transform.translation.y,
                ram_to_save,
                tutorial_state.visible,
                level_state.current_level,
                hacker_mode.active,
            );
        }
    }
}

pub fn save_game_state(
    player_query: Query<(&Transform, &RamState), With<Player>>,
    tutorial_state: Res<TutorialState>,
    level_state: Res<LevelState>,
    hacker_mode: Res<HackerMode>,
) {
    if let Ok((transform, ram_state)) = player_query.single() {
        let ram_to_save = if ram_state.current == 0 {
            ram_state.max
        } else {
            ram_state.current
        };
        save_game(
            transform.translation.x,
            transform.translation.y,
            ram_to_save,
            tutorial_state.visible,
            level_state.current_level,
            hacker_mode.active,
        );
    }
}

pub fn update_overclock(
    mut commands: Commands,
    time: Res<Time>,
    mut virtual_time: ResMut<Time<Virtual>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut overclock: ResMut<OverclockState>,
    mut achievements: ResMut<Achievements>,
    toast_container_query: Query<Entity, With<ToastContainer>>,
    level_state: Res<LevelState>,
) {
    let delta = time.delta_secs();

    if overclock.mode != CpuClockMode::Normal {
        overclock.timer -= delta;
        if overclock.timer <= 0.0 {
            overclock.mode = CpuClockMode::Normal;
            overclock.timer = 0.0;
            overclock.overclock_cooldown = 15.0;
            overclock.underclock_cooldown = 15.0;
            virtual_time.set_relative_speed(1.0);
        }
    } else {
        if overclock.overclock_cooldown > 0.0 {
            overclock.overclock_cooldown -= delta;
            if overclock.overclock_cooldown <= 0.0 {
                overclock.overclock_cooldown = 0.0;
            }
        }
        if overclock.underclock_cooldown > 0.0 {
            overclock.underclock_cooldown -= delta;
            if overclock.underclock_cooldown <= 0.0 {
                overclock.underclock_cooldown = 0.0;
            }
        }
    }

    let c_pressed = keyboard.just_pressed(KeyCode::KeyC) || keyboard.just_pressed(KeyCode::KeyQ);
    let gp_oc_pressed = gamepads.iter().any(|g| g.just_pressed(GamepadButton::RightThumb));

    let z_pressed = keyboard.just_pressed(KeyCode::KeyZ) || keyboard.just_pressed(KeyCode::KeyE);
    let gp_uc_pressed = gamepads.iter().any(|g| g.just_pressed(GamepadButton::LeftThumb));

    if overclock.mode == CpuClockMode::Normal {
        if level_state.current_level >= 2 {
            if (c_pressed || gp_oc_pressed) && overclock.overclock_cooldown == 0.0 {
                overclock.mode = CpuClockMode::Overclocked;
                overclock.timer = 3.0;
                virtual_time.set_relative_speed(1.6);
                unlock_achievement(&mut commands, &mut achievements, &toast_container_query, "turbo_charged");
            } else if (z_pressed || gp_uc_pressed) && overclock.underclock_cooldown == 0.0 {
                overclock.mode = CpuClockMode::Underclocked;
                overclock.timer = 3.0;
                virtual_time.set_relative_speed(0.4);
            }
        }
    }
}

pub fn handle_damage(
    time: Res<Time>,
    mut commands: Commands,
    mut achievements: ResMut<Achievements>,
    toast_container_query: Query<Entity, With<ToastContainer>>,
    mut player_query: Query<
        (
            &Transform,
            &Velocity,
            &JumpState,
            &DashState,
            &GlitchState,
            &mut RamState,
        ),
        With<Player>,
    >,
    spike_query: Query<&Transform, (With<Spike>, Without<Player>)>,
    wall_query: Query<&Transform, (With<Wall>, Without<Player>)>,
    mut enemy_query: Query<(&Transform, &mut Enemy, &mut Visibility, Option<&mut Boss>), (Without<Player>, Without<Spike>, Without<Wall>)>,
    laser_query: Query<(&Transform, &Laser), (Without<Player>, Without<Spike>, Without<Wall>)>,
    mut next_state: ResMut<NextState<AppState>>,
    hacker_mode: Res<HackerMode>,
) {
    let delta = time.delta_secs();

    if let Ok((
        player_trans,
        _velocity,
        jump_state,
        dash_state,
        glitch_state,
        mut ram_state,
    )) = player_query.single_mut() {
        // Update invulnerability timer
        if ram_state.invulnerability_timer > 0.0 {
            ram_state.invulnerability_timer -= delta;
            if ram_state.invulnerability_timer < 0.0 {
                ram_state.invulnerability_timer = 0.0;
            }
        }

        let player_x = player_trans.translation.x;
        let player_y = player_trans.translation.y;
        let player_left = player_x - PLAYER_SIZE.x / 2.0;
        let player_right = player_x + PLAYER_SIZE.x / 2.0;
        let player_top = player_y + PLAYER_SIZE.y / 2.0;
        let player_bottom = player_y - PLAYER_SIZE.y / 2.0;

        // Check if player has fallen off the floor and into the void
        if player_y < GROUND_Y - 300.0 {
            ram_state.current = 0;
            achievements.death_count += 1;
            crate::helpers::save_achievements(&achievements);
            
            unlock_achievement(&mut commands, &mut achievements, &toast_container_query, "system_crash");
            if achievements.death_count >= 10 {
                unlock_achievement(&mut commands, &mut achievements, &toast_container_query, "better_call_tech_support");
            }
            
            next_state.set(AppState::DeathScreen);
            return;
        }

        let mut take_damage = false;

        // 1. Check spike collisions (hazardous red zones) - only if we are NOT currently glitching
        if glitch_state.timer == 0.0 {
            for spike_trans in &spike_query {
                let spike_x = spike_trans.translation.x;
                let spike_y = spike_trans.translation.y;
                let spike_size = Vec2::new(48.0, 48.0);
                let spike_left = spike_x - spike_size.x / 2.0;
                let spike_right = spike_x + spike_size.x / 2.0;
                let spike_top = spike_y + spike_size.y / 2.0;
                let spike_bottom = spike_y - spike_size.y / 2.0;

                if player_right > spike_left
                    && player_left < spike_right
                    && player_top > spike_bottom
                    && player_bottom < spike_top
                {
                    take_damage = true;
                    break;
                }
            }
        }

        // 2. Check wall glitch failures: only check if the glitch just ended this frame
        if !take_damage && glitch_state.just_ended {
            for wall_trans in &wall_query {
                let wall_x = wall_trans.translation.x;
                let wall_y = wall_trans.translation.y;
                let wall_left = wall_x - WALL_SIZE.x / 2.0;
                let wall_right = wall_x + WALL_SIZE.x / 2.0;
                let wall_top = wall_y + WALL_SIZE.y / 2.0;
                let wall_bottom = wall_y - WALL_SIZE.y / 2.0;

                if player_right > wall_left
                    && player_left < wall_right
                    && player_top > wall_bottom
                    && player_bottom < wall_top
                {
                    take_damage = true;
                    break;
                }
            }
        }

        // 3. Check enemy collisions - only if we are NOT currently glitching
        if glitch_state.timer == 0.0 {
            for (enemy_trans, mut enemy, mut visibility, mut boss_opt) in &mut enemy_query {
                if enemy.is_destroyed {
                    continue;
                }

                let enemy_size = if boss_opt.is_some() {
                    Vec2::new(128.0, 128.0)
                } else {
                    Vec2::new(64.0, 64.0)
                };

                let enemy_x = enemy_trans.translation.x;
                let enemy_y = enemy_trans.translation.y;
                let enemy_left = enemy_x - enemy_size.x / 2.0;
                let enemy_right = enemy_x + enemy_size.x / 2.0;
                let enemy_top = enemy_y + enemy_size.y / 2.0;
                let enemy_bottom = enemy_y - enemy_size.y / 2.0;

                if player_right > enemy_left
                    && player_left < enemy_right
                    && player_top > enemy_bottom
                    && player_bottom < enemy_top
                {
                    let is_attacking = dash_state.dash_timer > 0.0 || jump_state.is_smashing;
                    if is_attacking {
                        if let Some(ref mut boss) = boss_opt {
                            if boss.invulnerable_timer == 0.0 {
                                if boss.health > 1 {
                                    boss.health -= 1;
                                    boss.invulnerable_timer = 0.5; // 0.5 seconds invulnerable
                                } else {
                                    boss.health = 0;
                                    enemy.is_destroyed = true;
                                    *visibility = Visibility::Hidden;

                                    // Spawn the gateway console since the boss is defeated!
                                    commands.spawn((
                                        LevelEntity,
                                        GatewayConsole,
                                        Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::new(40.0, 60.0)),
                                        Transform::from_xyz(300.0, GROUND_Y, 1.0),
                                    ));
                                }
                            }
                        } else {
                            enemy.is_destroyed = true;
                            *visibility = Visibility::Hidden;
                        }
                    } else {
                        take_damage = true;
                        break;
                    }
                }
            }
        }

        // 4. Check laser collisions - only if we are NOT currently glitching
        if !take_damage && glitch_state.timer == 0.0 {
            for (laser_trans, laser) in &laser_query {
                if !laser.is_active {
                    continue;
                }
                let laser_x = laser_trans.translation.x;
                let laser_y = laser_trans.translation.y;
                let laser_size = Vec2::new(1000.0, 24.0);
                let laser_left = laser_x - laser_size.x / 2.0;
                let laser_right = laser_x + laser_size.x / 2.0;
                let laser_top = laser_y + laser_size.y / 2.0;
                let laser_bottom = laser_y - laser_size.y / 2.0;

                if player_right > laser_left
                    && player_left < laser_right
                    && player_top > laser_bottom
                    && player_bottom < laser_top
                {
                    take_damage = true;
                    break;
                }
            }
        }

        // Apply damage if not invulnerable
        if take_damage && ram_state.invulnerability_timer == 0.0 {
            let damage = if hacker_mode.active { 4 } else { 2 };
            if ram_state.current >= damage {
                ram_state.current -= damage;
            } else {
                ram_state.current = 0;
            }
            ram_state.invulnerability_timer = if hacker_mode.active { 0.5 } else { 1.0 };

            // If crash, transition to DeathScreen
            if ram_state.current == 0 {
                achievements.death_count += 1;
                crate::helpers::save_achievements(&achievements);
                
                unlock_achievement(&mut commands, &mut achievements, &toast_container_query, "system_crash");
                if achievements.death_count >= 10 {
                    unlock_achievement(&mut commands, &mut achievements, &toast_container_query, "better_call_tech_support");
                }
                
                next_state.set(AppState::DeathScreen);
            }
        }
    }
}

pub fn resolve_collisions(
    mut commands: Commands,
    mut achievements: ResMut<Achievements>,
    toast_container_query: Query<Entity, With<ToastContainer>>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut JumpState,
            &mut DashState,
            &GlitchState,
        ),
        With<Player>,
    >,
    wall_query: Query<&Transform, (With<Wall>, Without<Player>)>,
    spike_query: Query<&Transform, (With<Spike>, Without<Player>, Without<Wall>)>,
) {
    for (mut player_trans, mut velocity, mut jump_state, mut dash_state, glitch_state) in &mut player_query {
        let player_x = player_trans.translation.x;
        let player_y = player_trans.translation.y;

        let player_left = player_x - PLAYER_SIZE.x / 2.0;
        let player_right = player_x + PLAYER_SIZE.x / 2.0;
        let player_top = player_y + PLAYER_SIZE.y / 2.0;
        let player_bottom = player_y - PLAYER_SIZE.y / 2.0;

        if glitch_state.timer > 0.0 {
            // Check if player overlaps with any wall or spike while glitching
            let mut overlapping = false;

            for wall_trans in &wall_query {
                let wall_x = wall_trans.translation.x;
                let wall_y = wall_trans.translation.y;
                let wall_left = wall_x - WALL_SIZE.x / 2.0;
                let wall_right = wall_x + WALL_SIZE.x / 2.0;
                let wall_top = wall_y + WALL_SIZE.y / 2.0;
                let wall_bottom = wall_y - WALL_SIZE.y / 2.0;

                if player_right > wall_left
                    && player_left < wall_right
                    && player_top > wall_bottom
                    && player_bottom < wall_top
                {
                    overlapping = true;
                    break;
                }
            }

            if !overlapping {
                for spike_trans in &spike_query {
                    let spike_x = spike_trans.translation.x;
                    let spike_y = spike_trans.translation.y;
                    let spike_size = Vec2::new(48.0, 48.0);
                    let spike_left = spike_x - spike_size.x / 2.0;
                    let spike_right = spike_x + spike_size.x / 2.0;
                    let spike_top = spike_y + spike_size.y / 2.0;
                    let spike_bottom = spike_y - spike_size.y / 2.0;

                    if player_right > spike_left
                        && player_left < spike_right
                        && player_top > spike_bottom
                        && player_bottom < spike_top
                    {
                        overlapping = true;
                        break;
                    }
                }
            }

            if overlapping {
                unlock_achievement(&mut commands, &mut achievements, &toast_container_query, "phase_shift");
            }
            continue;
        }

        for wall_trans in &wall_query {
            let wall_x = wall_trans.translation.x;
            let wall_y = wall_trans.translation.y;

            let wall_left = wall_x - WALL_SIZE.x / 2.0;
            let wall_right = wall_x + WALL_SIZE.x / 2.0;
            let wall_top = wall_y + WALL_SIZE.y / 2.0;
            let wall_bottom = wall_y - WALL_SIZE.y / 2.0;

            if player_right > wall_left
                && player_left < wall_right
                && player_top > wall_bottom
                && player_bottom < wall_top
            {
                let overlap_x = if player_x < wall_x {
                    player_right - wall_left
                } else {
                    wall_right - player_left
                };

                let overlap_y = if player_y < wall_y {
                    player_top - wall_bottom
                } else {
                    wall_top - player_bottom
                };

                if overlap_x < overlap_y {
                    if player_x < wall_x {
                        player_trans.translation.x -= overlap_x;
                    } else {
                        player_trans.translation.x += overlap_x;
                    }
                } else {
                    if player_y < wall_y {
                        player_trans.translation.y -= overlap_y;
                        if velocity.0.y > 0.0 {
                            velocity.0.y = 0.0;
                        }
                    } else {
                        player_trans.translation.y += overlap_y;
                        if velocity.0.y < 0.0 {
                            velocity.0.y = 0.0;
                        }
                        jump_state.jumps_remaining = jump_state.max_jumps;
                        jump_state.is_smashing = false;
                        dash_state.air_dash_used = false;
                    }
                }
            }
        }
    }
}

pub fn check_gate_collision(
    commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut level_state: ResMut<LevelState>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut JumpState, &mut DashState, &mut GlitchState, &mut RamState), (With<Player>, Without<GatewayConsole>)>,
    gate_query: Query<&Transform, (With<GatewayConsole>, Without<Player>)>,
    level_entity_query: Query<Entity, With<LevelEntity>>,
    hud_query: Query<Entity, With<GameHUD>>,
    tutorial_state: Res<TutorialState>,
    hacker_mode: Res<HackerMode>,
) {
    if let Ok((player_trans, _velocity, _jump_state, _dash_state, _glitch_state, _ram_state)) = player_query.single_mut() {
        for gate_trans in &gate_query {
            let px = player_trans.translation.x;
            let py = player_trans.translation.y;
            let gx = gate_trans.translation.x;
            let gy = gate_trans.translation.y;
            
            let dx = (px - gx).abs();
            let dy = (py - gy).abs();
            if dx < (48.0 + 40.0) / 2.0 && dy < (64.0 + 60.0) / 2.0 {
                if level_state.current_level < 3 {
                    level_state.current_level += 1;
                    load_level(
                        level_state.current_level,
                        commands,
                        &level_entity_query,
                        &mut player_query,
                        &hud_query,
                        &tutorial_state,
                        hacker_mode.active,
                    );
                    save_game(-350.0, GROUND_Y, 6, tutorial_state.visible, level_state.current_level, hacker_mode.active);
                } else if level_state.current_level == 3 {
                    level_state.current_level += 1;
                    save_game(-350.0, GROUND_Y, 6, tutorial_state.visible, level_state.current_level, hacker_mode.active);
                    next_state.set(AppState::BossTransition);
                } else {
                    next_state.set(AppState::DemoComplete);
                }
                break;
            }
        }
    }
}

pub fn update_hud(
    mut commands: Commands,
    level_state: Res<LevelState>,
    player_query: Query<&RamState, With<Player>>,
    mut level_hud_query: Query<&mut Text, (With<LevelHUD>, Without<RamHUD>, Without<OverclockHUD>)>,
    mut ram_hud_query: Query<&mut Text, (With<RamHUD>, Without<LevelHUD>, Without<OverclockHUD>)>,
    overclock: Res<OverclockState>,
    mut overclock_hud_query: Query<(&mut Text, &mut TextColor), (With<OverclockHUD>, Without<LevelHUD>, Without<RamHUD>)>,
    boss_query: Query<&Boss>,
    mut fill_query: Query<&mut Node, With<BossHealthBarFill>>,
    bar_query: Query<Entity, With<BossHealthBar>>,
    hacker_mode: Res<HackerMode>,
) {
    if let Ok(mut text) = level_hud_query.single_mut() {
        text.0 = format!("ZONE: TUTORIAL [SECTOR 0{}]", level_state.current_level);
    }

    if let Ok(ram_state) = player_query.single() {
        if let Ok(mut text) = ram_hud_query.single_mut() {
            text.0 = format!("SYSTEM MEMORY: {}GB / {}GB", ram_state.current, ram_state.max);
        }
    }

    if let Ok((mut text, mut color)) = overclock_hud_query.single_mut() {
        if level_state.current_level < 2 {
            text.0 = "CPU CLOCK: LOCKED (Unlock at Level 2)".to_string();
            color.0 = Color::srgb(0.5, 0.5, 0.5); // Gray
        } else {
            match overclock.mode {
                CpuClockMode::Overclocked => {
                    text.0 = format!("CPU CLOCK: 9.8GHz [OVERCLOCKED] ({:.1}s)", overclock.timer);
                    color.0 = Color::srgb(1.0, 0.4, 0.0); // Neon Orange
                }
                CpuClockMode::Underclocked => {
                    text.0 = format!("CPU CLOCK: 1.2GHz [UNDERCLOCKED] ({:.1}s)", overclock.timer);
                    color.0 = Color::srgb(0.0, 0.6, 1.0); // Neon Cyan/Blue
                }
                CpuClockMode::Normal => {
                    if overclock.overclock_cooldown > 0.0 {
                        text.0 = format!("CPU CLOCK: 4.2GHz [OC COOLDOWN] ({:.1}s)", overclock.overclock_cooldown);
                        color.0 = Color::srgb(0.5, 0.5, 0.5); // Gray
                    } else if overclock.underclock_cooldown > 0.0 {
                        text.0 = format!("CPU CLOCK: 4.2GHz [UC COOLDOWN] ({:.1}s)", overclock.underclock_cooldown);
                        color.0 = Color::srgb(0.5, 0.5, 0.5); // Gray
                    } else {
                        text.0 = "CPU CLOCK: 4.2GHz (READY - C: Overclock / Z: Underclock)".to_string();
                        color.0 = Color::srgb(0.0, 1.0, 0.0); // Neon Green
                    }
                }
            }
        }
    }

    // Update Boss Health Bar
    if let Ok(boss) = boss_query.single() {
        if boss.health > 0 {
            if let Ok(mut fill_node) = fill_query.single_mut() {
                fill_node.width = Val::Percent((boss.health as f32 / (if hacker_mode.active { 6.0 } else { 3.0 })) * 100.0);
            }
        } else {
            if let Ok(bar_entity) = bar_query.single() {
                commands.entity(bar_entity).despawn();
            }
        }
    } else {
        if let Ok(bar_entity) = bar_query.single() {
            commands.entity(bar_entity).despawn();
        }
    }
}
