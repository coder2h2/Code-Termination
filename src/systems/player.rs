use bevy::prelude::*;
use crate::constants::*;
use crate::components::*;
use crate::resources::*;
use crate::helpers::{unlock_achievement, load_game, load_level, save_game};

pub fn reset_player_system(
    commands: Commands,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut JumpState, &mut DashState, &mut GlitchState, &mut RamState), (With<Player>, Without<GatewayConsole>)>,
    level_entity_query: Query<Entity, With<LevelEntity>>,
    hud_query: Query<Entity, With<GameHUD>>,
    mut level_state: ResMut<LevelState>,
    mut tutorial_state: ResMut<TutorialState>,
    pending_load: Res<PendingGameLoad>,
) {
    let mut loaded_level = 1;
    let mut loaded = false;
    if pending_load.should_load {
        if let Some((x, y, ram, tut_visible, level)) = load_game() {
            tutorial_state.visible = tut_visible;
            loaded_level = level;
            level_state.current_level = level;
            for (mut transform, mut velocity, mut jump_state, mut dash_state, mut glitch_state, mut ram_state) in &mut player_query {
                transform.translation = Vec3::new(x, y, 1.0);
                velocity.0 = Vec2::ZERO;
                *jump_state = JumpState::default();
                *dash_state = DashState::default();
                *glitch_state = GlitchState::default();
                ram_state.current = if ram == 0 { ram_state.max } else { ram.min(ram_state.max) };
            }
            loaded = true;
        }
    }

    if !loaded {
        tutorial_state.visible = true;
        level_state.current_level = 1;
        loaded_level = 1;
        for (mut transform, mut velocity, mut jump_state, mut dash_state, mut glitch_state, mut ram_state) in &mut player_query {
            transform.translation = Vec3::new(-350.0, GROUND_Y, 1.0);
            velocity.0 = Vec2::ZERO;
            *jump_state = JumpState::default();
            *dash_state = DashState::default();
            *glitch_state = GlitchState::default();
            ram_state.current = ram_state.max;
        }
        save_game(-350.0, GROUND_Y, 6, true, 1);
    }

    // Load level entities and HUD
    load_level(
        loaded_level,
        commands,
        &level_entity_query,
        &mut player_query,
        &hud_query,
        &tutorial_state,
    );
}

pub fn update_glitch(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(&mut GlitchState, &mut Sprite, &mut Transform, &mut JumpState), With<Player>>,
) {
    let delta = time.delta_secs();

    for (mut glitch_state, mut sprite, mut transform, mut jump_state) in &mut query {
        glitch_state.just_ended = false;

        // Restore base position from previous frame's shake offset
        transform.translation.x -= glitch_state.prev_shake.x;
        transform.translation.y -= glitch_state.prev_shake.y;
        glitch_state.prev_shake = Vec2::ZERO;

        if glitch_state.timer > 0.0 {
            glitch_state.timer -= delta;
            if glitch_state.timer <= 0.0 {
                glitch_state.timer = 0.0;
                sprite.color = Color::srgb(0.48, 0.86, 0.62);
                glitch_state.just_ended = true;
            } else {
                let elapsed = time.elapsed_secs();
                
                // Rapidly cycle colors for visual glitch effect
                let color_idx = (elapsed * 40.0) as i32 % 5;
                sprite.color = match color_idx {
                    0 => Color::srgba(0.0, 1.0, 1.0, 0.6), // Cyan
                    1 => Color::srgba(1.0, 0.0, 0.5, 0.6), // Pink/Magenta
                    2 => Color::srgba(0.0, 1.0, 0.0, 0.6), // Lime Green
                    3 => Color::srgba(1.0, 1.0, 0.0, 0.6), // Electric Yellow
                    _ => Color::srgba(0.8, 0.0, 1.0, 0.6), // Neon Purple
                };

                // Apply dynamic high-frequency shake offset
                let shake_x = (elapsed * 130.0).sin() * 8.0;
                let shake_y = (elapsed * 170.0).cos() * 8.0;
                let shake = Vec2::new(shake_x, shake_y);

                transform.translation.x += shake.x;
                transform.translation.y += shake.y;
                glitch_state.prev_shake = shake;
            }
        }

        if glitch_state.cooldown > 0.0 {
            glitch_state.cooldown -= delta;
            if glitch_state.cooldown < 0.0 {
                glitch_state.cooldown = 0.0;
            }
        }

        let gp_glitch_pressed = gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::West)
                || g.just_pressed(GamepadButton::LeftTrigger)
                || g.just_pressed(GamepadButton::LeftTrigger2)
        });

        let mouse_clicked = mouse_buttons.just_pressed(MouseButton::Left);

        let shift_pressed = keyboard.just_pressed(KeyCode::ShiftLeft)
            || keyboard.just_pressed(KeyCode::ShiftRight)
            || keyboard.just_pressed(KeyCode::KeyF)
            || gp_glitch_pressed
            || mouse_clicked;

        if shift_pressed && glitch_state.cooldown == 0.0 && glitch_state.timer == 0.0 {
            glitch_state.timer = GLITCH_DURATION;
            glitch_state.cooldown = GLITCH_COOLDOWN;
            jump_state.is_smashing = false; // Cancel any active smashdown

            let mut dir = 0.0;
            if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
                dir -= 1.0;
            }
            if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
                dir += 1.0;
            }
            for gamepad in &gamepads {
                if gamepad.pressed(GamepadButton::DPadLeft) {
                    dir -= 1.0;
                }
                if gamepad.pressed(GamepadButton::DPadRight) {
                    dir += 1.0;
                }
                if let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX) {
                    if left_stick_x.abs() > 0.1 {
                        dir += left_stick_x;
                    }
                }
            }

            let mut glitched_anywhere = false;
            if dir == 0.0 {
                // If they are not moving, try to glitch to the cursor position
                if let Ok(window) = windows.single() {
                    if let Some(cursor_pos) = window.cursor_position() {
                        if let Ok((camera, camera_transform)) = camera_query.single() {
                            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                                transform.translation.x = world_pos.x;
                                transform.translation.y = world_pos.y;
                                glitched_anywhere = true;
                            }
                        }
                    }
                }
            }

            if !glitched_anywhere {
                if dir == 0.0 {
                    dir = 1.0;
                }
                let dir = dir.signum();
                transform.translation.x += dir * GLITCH_DISTANCE;
            }
        }
    }
}

pub fn move_player(
    gamepads: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    mut achievements: ResMut<Achievements>,
    toast_container_query: Query<Entity, With<ToastContainer>>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut DashState,
            &mut Sprite,
            &JumpState,
            &GlitchState,
            &RamState,
        ),
        With<Player>,
    >,
    level_state: Res<LevelState>,
    overclock: Res<OverclockState>,
) {
    let now = time.elapsed_secs();
    let delta = time.delta_secs();

    for (mut transform, mut dash_state, mut sprite, jump_state, glitch_state, ram_state) in &mut player_query {
        if jump_state.is_smashing {
            sprite.color = Color::srgb(1.0, 0.25, 0.0);
            dash_state.dash_timer = 0.0;
            continue;
        }

        let floor_half_width = FLOOR_WIDTH / 2.0;
        let on_floor = transform.translation.x >= -floor_half_width && transform.translation.x <= floor_half_width;
        let is_in_air = transform.translation.y > GROUND_Y || !on_floor;

        // Detect gamepad direction and button dash
        let mut gamepad_dir = 0.0;
        for gamepad in &gamepads {
            if gamepad.pressed(GamepadButton::DPadLeft) {
                gamepad_dir -= 1.0;
            }
            if gamepad.pressed(GamepadButton::DPadRight) {
                gamepad_dir += 1.0;
            }
            if let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX) {
                if left_stick_x.abs() > 0.1 {
                    gamepad_dir += left_stick_x;
                }
            }
        }
        gamepad_dir = gamepad_dir.clamp(-1.0, 1.0);

        let mut direction = 0.0;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }
        direction += gamepad_dir;
        direction = direction.clamp(-1.0, 1.0);

        let gp_dash_pressed = gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::North)
                || g.just_pressed(GamepadButton::RightTrigger)
                || g.just_pressed(GamepadButton::RightTrigger2)
        });

        // Detect double tap for A or ArrowLeft (left)
        let mut should_dash = gp_dash_pressed;
        let mut dash_dir_input = if direction != 0.0 { direction.signum() } else { 1.0 };

        if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
            if now - dash_state.last_a_press < DOUBLE_TAP_TIMEOUT && dash_state.dash_timer <= 0.0 {
                should_dash = true;
                dash_dir_input = -1.0;
            }
            dash_state.last_a_press = now;
        }

        // Detect double tap for D or ArrowRight (right)
        if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
            if now - dash_state.last_d_press < DOUBLE_TAP_TIMEOUT && dash_state.dash_timer <= 0.0 {
                should_dash = true;
                dash_dir_input = 1.0;
            }
            dash_state.last_d_press = now;
        }

        if level_state.current_level < 3 {
            should_dash = false;
        }

        if should_dash && dash_state.dash_timer <= 0.0 {
            if !is_in_air || !dash_state.air_dash_used {
                dash_state.dash_timer = DASH_DURATION;
                dash_state.dash_dir = dash_dir_input;
                if is_in_air {
                    dash_state.air_dash_used = true;
                }
                unlock_achievement(&mut commands, &mut achievements, &toast_container_query, "speed_daemon");
            }
        }

        // Apply movement
        let is_dashing = dash_state.dash_timer > 0.0;
        let is_glitching = glitch_state.timer > 0.0;

        let speed_multiplier = if overclock.mode == CpuClockMode::Overclocked {
            1.5
        } else {
            1.0
        };

        if is_dashing {
            transform.translation.x += dash_state.dash_dir * DASH_SPEED * speed_multiplier * delta;
            dash_state.dash_timer -= delta;
            if dash_state.dash_timer < 0.0 {
                dash_state.dash_timer = 0.0;
            }
        } else {
            if direction != 0.0 {
                transform.translation.x += direction * PLAYER_SPEED * speed_multiplier * delta;
            }
        }

        // Apply sprite color based on status
        if is_glitching {
            // Color is handled dynamically by update_glitch system
        } else if ram_state.invulnerability_timer > 0.0 {
            // Flash red/orange and semi-transparent
            let elapsed = time.elapsed_secs();
            if (elapsed * 15.0) as i32 % 2 == 0 {
                sprite.color = Color::srgba(1.0, 0.2, 0.2, 0.5);
            } else {
                sprite.color = Color::srgba(0.48, 0.86, 0.62, 0.3);
            }
        } else if is_dashing {
            sprite.color = Color::srgb(0.0, 1.0, 0.0); // Bright Lime Green
        } else {
            sprite.color = Color::srgb(0.48, 0.86, 0.62); // Light Mint Green
        }
    }
}

pub fn jump_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut commands: Commands,
    mut achievements: ResMut<Achievements>,
    toast_container_query: Query<Entity, With<ToastContainer>>,
    mut player_query: Query<(&Transform, &mut Velocity, &mut JumpState), With<Player>>,
) {
    let down_pressed = keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::ArrowDown)
        || gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::DPadDown)
                || g.get(GamepadAxis::LeftStickY).unwrap_or(0.0) < -0.5
        });

    let up_pressed = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::ArrowUp)
        || gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::South)
                || g.just_pressed(GamepadButton::East)
                || g.just_pressed(GamepadButton::DPadUp)
        });

    for (transform, mut velocity, mut jump_state) in &mut player_query {
        let floor_half_width = FLOOR_WIDTH / 2.0;
        let on_floor = transform.translation.x >= -floor_half_width && transform.translation.x <= floor_half_width;
        let is_in_air = transform.translation.y > GROUND_Y || !on_floor;

        if down_pressed && is_in_air {
            velocity.0.y = SMASHDOWN_SPEED;
            jump_state.is_smashing = true;
        } else if up_pressed && jump_state.jumps_remaining > 0 && !jump_state.is_smashing {
            if jump_state.jumps_remaining == 1 {
                unlock_achievement(&mut commands, &mut achievements, &toast_container_query, "airborne");
            }
            velocity.0.y = JUMP_SPEED;
            jump_state.jumps_remaining -= 1;
        }
    }
}

pub fn apply_velocity(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut JumpState, &mut DashState), With<Player>>,
) {
    for (mut transform, mut velocity, mut jump_state, mut dash_state) in &mut player_query {
        if dash_state.dash_timer > 0.0 {
            velocity.0.y = 0.0;
        } else {
            velocity.0.y -= GRAVITY * time.delta_secs();
        }
        
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        let floor_half_width = FLOOR_WIDTH / 2.0;
        let on_floor = transform.translation.x >= -floor_half_width && transform.translation.x <= floor_half_width;

        if on_floor && transform.translation.y < GROUND_Y {
            transform.translation.y = GROUND_Y;
            velocity.0.y = 0.0;
            jump_state.jumps_remaining = jump_state.max_jumps;
            jump_state.is_smashing = false;
            dash_state.air_dash_used = false;
        }
    }
}
