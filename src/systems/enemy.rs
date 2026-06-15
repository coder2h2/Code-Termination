use bevy::prelude::*;
use crate::constants::*;
use crate::components::*;
use crate::resources::*;

pub fn update_enemies(
    mut commands: Commands,
    time: Res<Time>,
    overclock: Res<OverclockState>,
    mut query: Query<(&mut Transform, &mut Enemy, &mut Visibility, &mut Sprite, Option<&mut Boss>), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    speech_query: Query<(Entity, &ChildOf), With<BossSpeechText>>,
    hacker_mode: Res<HackerMode>,
) {
    let delta = time.delta_secs();
    let mut speed_multiplier = match overclock.mode {
        CpuClockMode::Overclocked => 0.5,
        CpuClockMode::Underclocked => 0.4,
        CpuClockMode::Normal => 1.0,
    };
    if hacker_mode.active {
        speed_multiplier *= 1.6;
    }

    // Get player x position
    let player_x = if let Ok(player_trans) = player_query.single() {
        player_trans.translation.x
    } else {
        0.0
    };

    for (mut transform, mut enemy, mut visibility, mut sprite, mut boss_opt) in &mut query {
        if enemy.is_destroyed {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;

        // If it's the Boss, run the Boss State Machine
        if let Some(ref mut boss) = boss_opt {
            boss.state_timer -= delta;
            if boss.state_timer < 0.0 {
                boss.state_timer = 0.0;
            }

            match boss.state {
                BossAttackState::Intro => {
                    // Descend from ceiling (y = 400.0) to landing height (y = -96.0)
                    let target_y = -96.0;
                    if transform.translation.y > target_y {
                        transform.translation.y -= 250.0 * delta;
                        if transform.translation.y <= target_y {
                            transform.translation.y = target_y;
                        }
                    }

                    if boss.state_timer == 0.0 {
                        boss.state = BossAttackState::Patrol;
                        boss.attack_cooldown_timer = 2.0;

                        // Despawn speech bubble
                        for (speech_entity, _) in &speech_query {
                            commands.entity(speech_entity).despawn();
                        }
                    }
                }
                BossAttackState::Patrol => {
                    // Standard patrolling behavior
                    // Check if time to initiate an attack
                    boss.attack_cooldown_timer -= delta;
                    if boss.attack_cooldown_timer <= 0.0 {
                        // Choose between Ram, Sweep, and Laser
                        let elapsed = time.elapsed_secs();
                        let choice = (elapsed as i32) % 3;
                        if choice == 0 {
                            // Prepare Ram
                            boss.state = BossAttackState::PrepareRam;
                            boss.state_timer = 1.2; // 1.2 second warning
                            boss.ram_direction = if player_x > transform.translation.x { 1.0 } else { -1.0 };
                        } else if choice == 1 {
                            // Prepare Sweep
                            boss.state = BossAttackState::PrepareSweep;
                            // If player is on the right, boss starts sweep from the left (x = -280.0)
                            let start_x = if player_x > 0.0 { -280.0 } else { 280.0 };
                            boss.sweep_target_x = start_x;
                            boss.state_timer = 0.0; // move to startup position immediately
                        } else {
                            // Prepare Laser
                            boss.state = BossAttackState::PrepareLaser;
                            boss.state_timer = 0.0; // move to center immediately
                        }
                    } else {
                        // Regular patrol movement
                        transform.translation.x += enemy.direction * enemy.speed * speed_multiplier * delta;

                        // Check bounds and reverse direction
                        if transform.translation.x >= enemy.patrol_right {
                            transform.translation.x = enemy.patrol_right;
                            enemy.direction = -1.0;
                        } else if transform.translation.x <= enemy.patrol_left {
                            transform.translation.x = enemy.patrol_left;
                            enemy.direction = 1.0;
                        }
                    }
                }
                BossAttackState::PrepareRam => {
                    // Flash yellow/orange warning color, stationary
                    if boss.state_timer == 0.0 {
                        // Transition to Ram
                        boss.state = BossAttackState::Ram;
                        boss.state_timer = 0.8; // Ram charge duration
                    }
                }
                BossAttackState::Ram => {
                    // Charge at high speed
                    let charge_speed = if hacker_mode.active { 900.0 } else { 600.0 };
                    transform.translation.x += boss.ram_direction * charge_speed * speed_multiplier * delta;

                    // Bound checks
                    if transform.translation.x <= -320.0 {
                        transform.translation.x = -320.0;
                        boss.state_timer = 0.0;
                    } else if transform.translation.x >= 320.0 {
                        transform.translation.x = 320.0;
                        boss.state_timer = 0.0;
                    }

                    if boss.state_timer == 0.0 {
                        boss.state = BossAttackState::Cooldown;
                        boss.state_timer = 1.0; // 1.0 second recovery
                    }
                }
                BossAttackState::PrepareSweep => {
                    // Move to the startup position
                    let current_x = transform.translation.x;
                    let diff = boss.sweep_target_x - current_x;
                    if diff.abs() > 10.0 {
                        let move_dir = diff.signum();
                        transform.translation.x += move_dir * enemy.speed * speed_multiplier * delta;
                    } else {
                        // Reached startup position! Prepare warning for 1.0s
                        transform.translation.x = boss.sweep_target_x;
                        boss.state = BossAttackState::Sweep;
                        boss.state_timer = 1.0;
                        boss.sweep_target_x = -boss.sweep_target_x; // Target is opposite side
                    }
                }
                BossAttackState::Sweep => {
                    if boss.state_timer > 0.0 {
                        // Stationary warning
                    } else {
                        // Sweep across the ground to the other side
                        let sweep_speed = if hacker_mode.active { 700.0 } else { 450.0 };
                        let diff = boss.sweep_target_x - transform.translation.x;
                        if diff.abs() > 10.0 {
                            let sweep_dir = diff.signum();
                            transform.translation.x += sweep_dir * sweep_speed * speed_multiplier * delta;
                        } else {
                            // Sweep complete!
                            transform.translation.x = boss.sweep_target_x;
                            boss.state = BossAttackState::Cooldown;
                            boss.state_timer = 1.0;
                        }
                    }
                }
                BossAttackState::PrepareLaser => {
                    // Move to the center of the arena (x = 0.0)
                    let current_x = transform.translation.x;
                    let diff = 0.0 - current_x;
                    if diff.abs() > 10.0 {
                        let move_dir = diff.signum();
                        transform.translation.x += move_dir * enemy.speed * speed_multiplier * delta;
                    } else {
                        // Reached center! Spawn the warning laser beam
                        transform.translation.x = 0.0;
                        boss.state = BossAttackState::Laser;
                        boss.state_timer = 2.7; // 1.2s warning + 1.5s active firing

                        commands.spawn((
                            LevelEntity,
                            Laser {
                                timer: 1.2,
                                is_active: false,
                            },
                            Sprite {
                                color: Color::srgba(1.0, 0.0, 0.0, 0.5),
                                custom_size: Some(Vec2::new(1000.0, 4.0)),
                                ..default()
                            },
                            Transform::from_xyz(0.0, GROUND_Y - 10.0, 1.0),
                        ));
                    }
                }
                BossAttackState::Laser => {
                    // Channeling laser - stationary
                    if boss.state_timer == 0.0 {
                        boss.state = BossAttackState::Cooldown;
                        boss.state_timer = 1.2; // 1.2 second recovery after laser
                    }
                }
                BossAttackState::Cooldown => {
                    if boss.state_timer == 0.0 {
                        boss.state = BossAttackState::Patrol;
                        boss.attack_cooldown_timer = 4.0; // 4 seconds of patrolling
                    }
                }
            }
        } else {
            // Regular enemy update
            transform.translation.x += enemy.direction * enemy.speed * speed_multiplier * delta;

            // Check bounds and reverse direction
            if transform.translation.x >= enemy.patrol_right {
                transform.translation.x = enemy.patrol_right;
                enemy.direction = -1.0;
            } else if transform.translation.x <= enemy.patrol_left {
                transform.translation.x = enemy.patrol_left;
                enemy.direction = 1.0;
            }
        }

        // Tick invulnerability
        if let Some(ref mut boss) = boss_opt {
            if boss.invulnerable_timer > 0.0 {
                boss.invulnerable_timer -= delta;
                if boss.invulnerable_timer < 0.0 {
                    boss.invulnerable_timer = 0.0;
                }
            }
        }

        // Change color dynamically
        if let Some(ref boss) = boss_opt {
            if boss.invulnerable_timer > 0.0 {
                // Flash white/red
                let elapsed = time.elapsed_secs();
                if (elapsed * 20.0) as i32 % 2 == 0 {
                    sprite.color = Color::srgb(1.0, 1.0, 1.0); // White flash
                } else {
                    sprite.color = Color::srgb(1.0, 0.3, 0.3); // Red flash
                }
            } else {
                match boss.state {
                    BossAttackState::Intro => {
                        sprite.color = Color::srgb(1.0, 0.0, 0.0);
                    }
                    BossAttackState::Patrol => {
                        match overclock.mode {
                            CpuClockMode::Overclocked => {
                                sprite.color = Color::srgb(1.0, 0.4, 0.0); // Neon Orange
                            }
                            CpuClockMode::Underclocked => {
                                sprite.color = Color::srgb(0.0, 0.6, 1.0); // Neon Cyan/Blue
                            }
                            CpuClockMode::Normal => {
                                sprite.color = Color::srgb(1.0, 0.0, 0.0);
                            }
                        }
                    }
                    BossAttackState::PrepareRam => {
                        let elapsed = time.elapsed_secs();
                        if (elapsed * 15.0) as i32 % 2 == 0 {
                            sprite.color = Color::srgb(1.0, 0.8, 0.0); // Yellow
                        } else {
                            sprite.color = Color::srgb(1.0, 0.4, 0.0); // Orange
                        }
                    }
                    BossAttackState::Ram => {
                        sprite.color = Color::srgb(1.0, 0.2, 0.0); // Intense red-orange
                    }
                    BossAttackState::PrepareSweep => {
                        let elapsed = time.elapsed_secs();
                        if (elapsed * 10.0) as i32 % 2 == 0 {
                            sprite.color = Color::srgb(0.8, 0.0, 0.8);
                        } else {
                            sprite.color = Color::srgb(0.5, 0.0, 0.5);
                        }
                    }
                    BossAttackState::Sweep => {
                        if boss.state_timer > 0.0 {
                            let elapsed = time.elapsed_secs();
                            if (elapsed * 15.0) as i32 % 2 == 0 {
                                sprite.color = Color::srgb(1.0, 0.0, 1.0); // Bright Magenta
                            } else {
                                sprite.color = Color::srgb(0.5, 0.0, 0.5);
                            }
                        } else {
                            sprite.color = Color::srgb(0.8, 0.0, 1.0); // Purple glow
                        }
                    }
                    BossAttackState::PrepareLaser => {
                        let elapsed = time.elapsed_secs();
                        if (elapsed * 10.0) as i32 % 2 == 0 {
                            sprite.color = Color::srgb(0.0, 1.0, 1.0); // Cyan
                        } else {
                            sprite.color = Color::srgb(1.0, 0.0, 0.0); // Red
                        }
                    }
                    BossAttackState::Laser => {
                        sprite.color = Color::srgb(0.0, 1.0, 1.0); // Glowing cyan during firing
                    }
                    BossAttackState::Cooldown => {
                        sprite.color = Color::srgb(0.4, 0.4, 0.4); // Grey cooldown
                    }
                }
            }
        } else {
            match overclock.mode {
                CpuClockMode::Overclocked => {
                    sprite.color = Color::srgb(1.0, 0.4, 0.0); // Neon Orange
                }
                CpuClockMode::Underclocked => {
                    sprite.color = Color::srgb(0.0, 0.6, 1.0); // Neon Cyan/Blue
                }
                CpuClockMode::Normal => {
                    sprite.color = Color::srgb(1.0, 0.0, 0.5);
                }
            }
        }
    }
}

pub fn update_lasers(
    time: Res<Time>,
    overclock: Res<OverclockState>,
    mut commands: Commands,
    mut laser_query: Query<(Entity, &mut Laser, &mut Sprite)>,
) {
    let delta = time.delta_secs();
    let speed_multiplier = match overclock.mode {
        CpuClockMode::Overclocked => 0.5,
        CpuClockMode::Underclocked => 0.4,
        CpuClockMode::Normal => 1.0,
    };

    for (entity, mut laser, mut sprite) in &mut laser_query {
        laser.timer -= delta * speed_multiplier;
        if laser.timer <= 0.0 {
            if !laser.is_active {
                // Warning phase finished. Transition to active firing!
                laser.is_active = true;
                laser.timer = 1.5; // active firing duration
                sprite.color = Color::srgb(0.0, 1.0, 1.0); // Neon Cyan laser beam
                sprite.custom_size = Some(Vec2::new(1000.0, 24.0));
            } else {
                // Done firing, despawn
                commands.entity(entity).despawn();
            }
        } else if !laser.is_active {
            // Flash the warning laser line
            let elapsed = time.elapsed_secs();
            if (elapsed * 20.0) as i32 % 2 == 0 {
                sprite.color = Color::srgba(1.0, 0.0, 0.0, 0.8);
            } else {
                sprite.color = Color::srgba(1.0, 0.0, 0.0, 0.2);
            }
        }
    }
}
