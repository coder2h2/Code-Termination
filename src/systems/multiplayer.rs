use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::constants::{PLAYER_SIZE, GROUND_Y};
use std::net::{UdpSocket, SocketAddr};

pub struct MultiplayerSocketData {
    pub socket: UdpSocket,
    pub peer_addr: Option<SocketAddr>,
    pub is_host: bool,
    pub room_code: String,
    pub last_received: f32,
    pub is_connected: bool,
}

#[derive(Resource, Default)]
pub struct MultiplayerSocket {
    pub data: Option<MultiplayerSocketData>,
}

#[derive(Resource)]
pub struct MultiplayerChannel {
    pub rx: std::sync::Mutex<Option<std::sync::mpsc::Receiver<MultiplayerEvent>>>,
}

impl Default for MultiplayerChannel {
    fn default() -> Self {
        Self {
            rx: std::sync::Mutex::new(None),
        }
    }
}

pub enum MultiplayerEvent {
    HostSuccess {
        u_socket: UdpSocket,
        room_code: String,
    },
    HostFailure(String),
    JoinSuccess {
        u_socket: UdpSocket,
        peer_addr: SocketAddr,
        room_code: String,
    },
    JoinFailure(String),
}

#[derive(Component)]
pub struct JoinInputText;

#[derive(Component)]
pub struct JoinStatusText;

#[derive(Component)]
pub struct HostWaitingRoomCodeText;

// --- Base64 Encoder ---
fn base64_encode(bytes: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        let b1 = if i + 1 < bytes.len() { Some(bytes[i + 1]) } else { None };
        let b2 = if i + 2 < bytes.len() { Some(bytes[i + 2]) } else { None };
        
        let u24 = ((b0 as u32) << 16)
            | ((b1.unwrap_or(0) as u32) << 8)
            | (b2.unwrap_or(0) as u32);
            
        result.push(CHARSET[((u24 >> 18) & 63) as usize] as char);
        result.push(CHARSET[((u24 >> 12) & 63) as usize] as char);
        
        if b1.is_some() {
            result.push(CHARSET[((u24 >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }
        
        if b2.is_some() {
            result.push(CHARSET[(u24 & 63) as usize] as char);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

// --- GitHub API Helpers ---
fn get_github_token() -> Option<String> {
    std::env::var("DLC_PAT").or_else(|_| std::env::var("GITHUB_TOKEN")).ok()
}

fn host_room_via_proxy(room_code: &str, ip_port: &str) -> Result<(), String> {
    let url = format!(
        "https://code-termination-proxy.coder2h2.workers.dev/rooms/{}",
        room_code
    );
    
    let output = std::process::Command::new("curl")
        .arg("-X")
        .arg("PUT")
        .arg("-s")
        .arg("-m")
        .arg("6") // 6 seconds timeout
        .arg("-d")
        .arg(ip_port)
        .arg(&url)
        .output();
        
    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                let body = String::from_utf8_lossy(&out.stdout).to_string();
                Err(format!("Proxy error: {} {}", err, body))
            }
        }
        Err(e) => Err(format!("Failed to run curl: {:?}", e)),
    }
}

fn generate_room_code() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let code = (nanos % 9000) + 1000; // Generate 4-digit code (1000 - 9999)
    code.to_string()
}

fn host_room_on_github(room_code: &str, ip_port: &str, token: &str) -> Result<(), String> {
    let payload = format!(
        r#"{{"message":"Host room {}","content":"{}"}}"#,
        room_code,
        base64_encode(ip_port.as_bytes())
    );
    let url = format!(
        "https://api.github.com/repos/coder2h2/Transmit-Center/contents/{}.txt",
        room_code
    );
    
    let output = std::process::Command::new("curl")
        .arg("-X")
        .arg("PUT")
        .arg("-s")
        .arg("-m")
        .arg("5") // 5s timeout
        .arg("-H")
        .arg(format!("Authorization: token {}", token))
        .arg("-H")
        .arg("User-Agent: Code-Termination")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(payload)
        .arg(&url)
        .output();
        
    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                let err = String::from_utf8_lossy(&out.stderr).to_string();
                let body = String::from_utf8_lossy(&out.stdout).to_string();
                Err(format!("GitHub API error: {} {}", err, body))
            }
        }
        Err(e) => Err(format!("Failed to run curl: {:?}", e)),
    }
}

fn close_room_via_proxy(room_code: &str) {
    let url = format!(
        "https://code-termination-proxy.coder2h2.workers.dev/rooms/{}",
        room_code
    );
    
    let _ = std::process::Command::new("curl")
        .arg("-X")
        .arg("DELETE")
        .arg("-s")
        .arg("-m")
        .arg("6") // 6 seconds timeout
        .arg(&url)
        .output();
}

pub fn close_room_on_github(room_code: &str) {
    let Some(token) = get_github_token() else {
        close_room_via_proxy(room_code);
        return;
    };
    let get_url = format!(
        "https://api.github.com/repos/coder2h2/Transmit-Center/contents/{}.txt",
        room_code
    );
    
    let get_output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-m")
        .arg("5")
        .arg("-H")
        .arg(format!("Authorization: token {}", token))
        .arg("-H")
        .arg("User-Agent: Code-Termination")
        .arg(&get_url)
        .output();
        
    if let Ok(out) = get_output {
        let s = String::from_utf8_lossy(&out.stdout);
        if let Some(sha_idx) = s.find("\"sha\":\"") {
            let sha = &s[sha_idx + 7..];
            if let Some(end_idx) = sha.find('"') {
                let sha_val = &sha[..end_idx];
                let payload = format!(r#"{{"message":"Close room","sha":"{}"}}"#, sha_val);
                let _ = std::process::Command::new("curl")
                    .arg("-X")
                    .arg("DELETE")
                    .arg("-s")
                    .arg("-m")
                    .arg("5")
                    .arg("-H")
                    .arg(format!("Authorization: token {}", token))
                    .arg("-H")
                    .arg("User-Agent: Code-Termination")
                    .arg("-H")
                    .arg("Content-Type: application/json")
                    .arg("-d")
                    .arg(payload)
                    .arg(&get_url)
                    .output();
            }
        }
    }
}

fn fetch_room_address(room_code: &str) -> Result<String, String> {
    let url = format!(
        "https://raw.githubusercontent.com/coder2h2/Transmit-Center/main/{}.txt",
        room_code
    );
    
    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-m")
        .arg("5") // 5s timeout
        .arg(&url)
        .output();
        
    match output {
        Ok(out) => {
            if out.status.success() {
                let content = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if content.is_empty() || content.contains("404") || content.contains("Not Found") {
                    Err("Room not found".to_string())
                } else {
                    Ok(content)
                }
            } else {
                Err("Failed to fetch room code from GitHub".to_string())
            }
        }
        Err(e) => Err(format!("Failed to run curl: {:?}", e)),
    }
}

// --- UI Screens ---

pub fn setup_multiplayer_menu(
    mut commands: Commands,
    mut menu_selection: ResMut<MenuSelection>,
) {
    menu_selection.selected_index = 0;
    
    commands.spawn((
        MultiplayerMenuUI,
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
        parent.spawn((
            Text::new("MULTIPLAYER ROOMS"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // Host Button
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
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            MultiplayerButtonAction::Host,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("HOST GAME"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // Join Button
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
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            MultiplayerButtonAction::Join,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("JOIN GAME"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
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
            MultiplayerButtonAction::Back,
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

pub fn cleanup_multiplayer_menu(
    mut commands: Commands,
    query: Query<Entity, With<MultiplayerMenuUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn multiplayer_menu_button_system(
    mut button_query: Query<
        (Entity, &Interaction, &MultiplayerButtonAction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut menu_selection: ResMut<MenuSelection>,
    gamepads: Query<&Gamepad>,
    mut stick_neutral: Local<bool>,
    channel: Res<MultiplayerChannel>,
) {
    let mut buttons = button_query.iter_mut().collect::<Vec<_>>();
    buttons.sort_by_key(|(_, _, action, _)| match action {
        MultiplayerButtonAction::Host => 0,
        MultiplayerButtonAction::Join => 1,
        MultiplayerButtonAction::Back => 2,
        _ => 3,
    });
    let total_buttons = buttons.len();

    let mut up = false;
    let mut down = false;
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::DPadUp) { up = true; }
        if gamepad.just_pressed(GamepadButton::DPadDown) { down = true; }
    }

    let mut stick_up = false;
    let mut stick_down = false;
    let mut any_active = false;
    for gamepad in &gamepads {
        if let Some(y) = gamepad.get(GamepadAxis::LeftStickY) {
            if y > 0.5 {
                any_active = true;
                if *stick_neutral { stick_up = true; }
            } else if y < -0.5 {
                any_active = true;
                if *stick_neutral { stick_down = true; }
            }
        }
    }
    if any_active { *stick_neutral = false; } else { *stick_neutral = true; }

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
            MultiplayerButtonAction::Host => {
                host_game_start(channel, next_state);
            }
            MultiplayerButtonAction::Join => {
                next_state.set(AppState::JoinInput);
            }
            MultiplayerButtonAction::Back => {
                next_state.set(AppState::TitleScreen);
            }
            _ => {}
        }
    }
}

pub fn host_game_start(
    channel: Res<MultiplayerChannel>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    if let Ok(mut rx_opt) = channel.rx.lock() {
        *rx_opt = Some(rx);
    }
    
    std::thread::spawn(move || {
        // Query public IP address
        let ip_output = std::process::Command::new("curl")
            .arg("-s")
            .arg("-m")
            .arg("3")
            .arg("https://api.ipify.org")
            .output();
            
        let public_ip = if let Ok(out) = ip_output {
            if out.status.success() {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            } else {
                "127.0.0.1".to_string()
            }
        } else {
            "127.0.0.1".to_string()
        };
        
        let port = 50505;
        let socket = match UdpSocket::bind(("0.0.0.0", port)) {
            Ok(s) => s,
            Err(_) => {
                match UdpSocket::bind("0.0.0.0:0") {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(MultiplayerEvent::HostFailure(format!("Failed to bind UDP: {:?}", e)));
                        return;
                    }
                }
            }
        };
        
        let bound_port = socket.local_addr().map(|a| a.port()).unwrap_or(port);
        let ip_port = format!("{}:{}", public_ip, bound_port);
        
        // If developer has a token, register code on GitHub; otherwise try proxy
        if let Some(token) = get_github_token() {
            let room_code = generate_room_code();
            match host_room_on_github(&room_code, &ip_port, &token) {
                Ok(_) => {
                    let _ = socket.set_nonblocking(true);
                    let _ = tx.send(MultiplayerEvent::HostSuccess { u_socket: socket, room_code });
                }
                Err(_) => {
                    // Fallback to direct connection if GitHub fails
                    let _ = socket.set_nonblocking(true);
                    let _ = tx.send(MultiplayerEvent::HostSuccess { 
                        u_socket: socket, 
                        room_code: format!("DIRECT ({})", ip_port)
                    });
                }
            }
        } else {
            // No token present: Try proxy
            let room_code = generate_room_code();
            match host_room_via_proxy(&room_code, &ip_port) {
                Ok(_) => {
                    let _ = socket.set_nonblocking(true);
                    let _ = tx.send(MultiplayerEvent::HostSuccess { u_socket: socket, room_code });
                }
                Err(_) => {
                    // Fallback to direct connection if proxy fails
                    let _ = socket.set_nonblocking(true);
                    let _ = tx.send(MultiplayerEvent::HostSuccess { 
                        u_socket: socket, 
                        room_code: format!("DIRECT ({})", ip_port) 
                    });
                }
            }
        }
    });
    
    next_state.set(AppState::HostWaiting);
}

// --- Host Waiting Screen ---

pub fn setup_host_waiting(
    mut commands: Commands,
) {
    commands.spawn((
        HostWaitingUI,
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
        parent.spawn((
            Text::new("HOSTING MULTIPLAYER ROOM"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new("ROOM CODE:"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        parent.spawn((
            HostWaitingRoomCodeText,
            Text::new("CONNECTING TO GITHUB..."),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        parent.spawn((
            HostWaitingAddressText,
            Text::new("Waiting for a client process to connect..."),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // Cancel Button
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
            BorderColor::all(Color::srgb(0.9, 0.1, 0.1)),
            MultiplayerButtonAction::CancelHost,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("CANCEL"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
            ));
        });
    });
}

pub fn cleanup_host_waiting(
    mut commands: Commands,
    query: Query<Entity, With<HostWaitingUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn host_waiting_button_system(
    mut button_query: Query<
        (&Interaction, &MultiplayerButtonAction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut socket: ResMut<MultiplayerSocket>,
) {
    for (interaction, action, mut bg_color) in &mut button_query {
        if *interaction == Interaction::Hovered {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            if *action == MultiplayerButtonAction::CancelHost {
                if let Some(ref data) = socket.data {
                    if !data.room_code.starts_with("DIRECT") {
                        close_room_on_github(&data.room_code);
                    }
                }
                socket.data = None;
                next_state.set(AppState::MultiplayerMenu);
            }
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }
}

pub fn host_waiting_ui_update_system(
    socket: Res<MultiplayerSocket>,
    mut code_query: Query<&mut Text, (With<HostWaitingRoomCodeText>, Without<HostWaitingAddressText>)>,
    mut addr_query: Query<&mut Text, (With<HostWaitingAddressText>, Without<HostWaitingRoomCodeText>)>,
) {
    let Some(ref data) = socket.data else { return; };
    
    if let Ok(mut text) = code_query.single_mut() {
        if data.room_code.starts_with("DIRECT") {
            if text.0 != "DIRECT CONNECT" {
                text.0 = "DIRECT CONNECT".to_string();
            }
        } else {
            if text.0 != data.room_code {
                text.0 = data.room_code.clone();
            }
        }
    }
    
    if let Ok(mut text) = addr_query.single_mut() {
        if data.room_code.starts_with("DIRECT") {
            let ip_port = data.room_code.replace("DIRECT (", "").replace(")", "");
            let display = format!("Share this address with peer:\n{}", ip_port);
            if text.0 != display {
                text.0 = display;
            }
        } else {
            let display = "Waiting for peer to join...".to_string();
            if text.0 != display {
                text.0 = display;
            }
        }
    }
}

pub fn receive_join_packets_system(
    mut socket: ResMut<MultiplayerSocket>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(ref mut data) = socket.data else { return; };
    let mut buf = [0u8; 1024];
    if let Ok((size, addr)) = data.socket.recv_from(&mut buf) {
        let msg = String::from_utf8_lossy(&buf[..size]);
        if msg.trim() == "JOIN" {
            data.peer_addr = Some(addr);
            data.is_connected = true;
            for _ in 0..5 {
                let _ = data.socket.send_to(b"ACK", addr);
            }
            next_state.set(AppState::Game);
        }
    }
}

// --- Join Input Screen ---

pub fn setup_join_input(
    mut commands: Commands,
    mut code_input: ResMut<RoomCodeInput>,
) {
    *code_input = RoomCodeInput::default();
    
    commands.spawn((
        JoinInputUI,
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
        parent.spawn((
            Text::new("JOIN MULTIPLAYER ROOM"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new("ENTER ROOM CODE OR DIRECT IP:"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        parent.spawn((
            JoinInputText,
            Text::new("_ _ _ _"),
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

        parent.spawn((
            JoinStatusText,
            Text::new("Type code using keyboard digits"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

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
            BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
            MultiplayerButtonAction::BackToMenu,
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

pub fn cleanup_join_input(
    mut commands: Commands,
    query: Query<Entity, With<JoinInputUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn join_input_button_system(
    mut button_query: Query<
        (&Interaction, &MultiplayerButtonAction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    channel: Res<MultiplayerChannel>,
) {
    for (interaction, action, mut bg_color) in &mut button_query {
        if *interaction == Interaction::Hovered {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            if *action == MultiplayerButtonAction::BackToMenu {
                if let Ok(mut rx_opt) = channel.rx.lock() {
                    *rx_opt = None;
                }
                next_state.set(AppState::MultiplayerMenu);
            }
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }
}

pub fn join_input_keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut code_input: ResMut<RoomCodeInput>,
    channel: Res<MultiplayerChannel>,
) {
    if code_input.is_connecting { return; }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MultiplayerMenu);
        return;
    }

    let digits = [
        (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'), (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'), (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'), (KeyCode::Numpad0, '0'), (KeyCode::Numpad1, '1'),
        (KeyCode::Numpad2, '2'), (KeyCode::Numpad3, '3'), (KeyCode::Numpad4, '4'),
        (KeyCode::Numpad5, '5'), (KeyCode::Numpad6, '6'), (KeyCode::Numpad7, '7'),
        (KeyCode::Numpad8, '8'), (KeyCode::Numpad9, '9'),
    ];

    for (key, ch) in digits {
        if keyboard.just_pressed(key) && code_input.code.len() < 22 {
            code_input.code.push(ch);
            code_input.error_message.clear();
        }
    }

    if (keyboard.just_pressed(KeyCode::Period) || keyboard.just_pressed(KeyCode::NumpadDecimal)) && code_input.code.len() < 22 {
        code_input.code.push('.');
        code_input.error_message.clear();
    }

    if keyboard.just_pressed(KeyCode::Semicolon) && code_input.code.len() < 22 {
        code_input.code.push(':');
        code_input.error_message.clear();
    }

    if keyboard.just_pressed(KeyCode::Backspace) && !code_input.code.is_empty() {
        code_input.code.pop();
        code_input.error_message.clear();
    }

    if keyboard.just_pressed(KeyCode::Enter) && !code_input.code.is_empty() {
        code_input.is_connecting = true;
        if code_input.code.contains('.') || code_input.code.contains(':') {
            code_input.status_message = "CONNECTING DIRECTLY...".to_string();
        } else {
            code_input.status_message = "FETCHING ROOM ADDRESS...".to_string();
        }
        join_game_start(code_input.code.clone(), channel);
    }
}

pub fn join_game_start(
    room_code: String,
    channel: Res<MultiplayerChannel>,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    if let Ok(mut rx_opt) = channel.rx.lock() {
        *rx_opt = Some(rx);
    }
    
    std::thread::spawn(move || {
        let addr_str = if room_code.contains('.') || room_code.contains(':') {
            room_code
        } else {
            match fetch_room_address(&room_code) {
                Ok(a) => a,
                Err(e) => {
                    let _ = tx.send(MultiplayerEvent::JoinFailure(e));
                    return;
                }
            }
        };
        
        let peer_addr = match addr_str.parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => {
                let raw_ip = format!("{}:50505", addr_str.trim());
                match raw_ip.parse::<SocketAddr>() {
                    Ok(addr) => addr,
                    Err(e) => {
                        let _ = tx.send(MultiplayerEvent::JoinFailure(format!("Invalid address format: {:?}", e)));
                        return;
                    }
                }
            }
        };
        
        let socket = match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(MultiplayerEvent::JoinFailure(format!("Failed to bind UDP: {:?}", e)));
                return;
            }
        };
        
        let _ = socket.set_nonblocking(true);
        let _ = tx.send(MultiplayerEvent::JoinSuccess { u_socket: socket, peer_addr, room_code: addr_str });
    });
}

pub fn update_join_input_ui_system(
    code_input: Res<RoomCodeInput>,
    mut text_query: Query<&mut Text, (With<JoinInputText>, Without<JoinStatusText>)>,
    mut status_query: Query<&mut Text, (With<JoinStatusText>, Without<JoinInputText>)>,
) {
    if code_input.is_changed() {
        for mut text in &mut text_query {
            if code_input.code.len() <= 4 && !code_input.code.contains('.') && !code_input.code.contains(':') {
                let mut display = String::new();
                for i in 0..4 {
                    if i < code_input.code.len() {
                        display.push(code_input.code.chars().nth(i).unwrap());
                    } else {
                        display.push('_');
                    }
                    if i < 3 {
                        display.push(' ');
                    }
                }
                text.0 = display;
            } else {
                text.0 = code_input.code.clone();
            }
        }

        for mut status in &mut status_query {
            if !code_input.error_message.is_empty() {
                status.0 = code_input.error_message.clone();
            } else if !code_input.status_message.is_empty() {
                status.0 = code_input.status_message.clone();
            } else if code_input.code.contains('.') || code_input.code.contains(':') {
                status.0 = "Press ENTER to connect directly".to_string();
            } else if code_input.code.len() == 4 {
                status.0 = "Press ENTER to fetch room details".to_string();
            } else {
                status.0 = "Type code or host IP/port".to_string();
            }
        }
    }
}

// --- Multiplayer Channels Receiver ---

pub fn multiplayer_channel_system(
    mut socket: ResMut<MultiplayerSocket>,
    channel: Res<MultiplayerChannel>,
    mut next_state: ResMut<NextState<AppState>>,
    mut code_input: ResMut<RoomCodeInput>,
    time: Res<Time>,
) {
    let mut rx_taken = None;
    if let Ok(mut rx_opt) = channel.rx.lock() {
        if let Some(ref rx) = *rx_opt {
            if let Ok(event) = rx.try_recv() {
                rx_taken = Some(event);
            }
        }
        if rx_taken.is_some() {
            *rx_opt = None;
        }
    }

    if let Some(event) = rx_taken {
        match event {
            MultiplayerEvent::HostSuccess { u_socket, room_code } => {
                socket.data = Some(MultiplayerSocketData {
                    socket: u_socket,
                    peer_addr: None,
                    is_host: true,
                    room_code,
                    last_received: time.elapsed_secs(),
                    is_connected: false,
                });
            }
            MultiplayerEvent::HostFailure(err) => {
                println!("[Multiplayer] Host failed: {}", err);
                next_state.set(AppState::MultiplayerMenu);
            }
            MultiplayerEvent::JoinSuccess { u_socket, peer_addr, room_code } => {
                socket.data = Some(MultiplayerSocketData {
                    socket: u_socket,
                    peer_addr: Some(peer_addr),
                    is_host: false,
                    room_code,
                    last_received: time.elapsed_secs(),
                    is_connected: false,
                });
                next_state.set(AppState::Game);
            }
            MultiplayerEvent::JoinFailure(err) => {
                println!("[Multiplayer] Join failed: {}", err);
                code_input.error_message = format!("ERROR: {}", err.to_uppercase());
                code_input.is_connecting = false;
                code_input.status_message.clear();
            }
        }
    }
}

// --- Gameplay State Sync System ---

pub fn multiplayer_send_system(
    socket: Res<MultiplayerSocket>,
    player_query: Query<(&Transform, &Sprite), With<Player>>,
    level_state: Res<LevelState>,
    time: Res<Time>,
    mut local_timer: Local<f32>,
) {
    let Some(ref data) = socket.data else { return; };
    
    if !data.is_connected {
        if !data.is_host {
            *local_timer += time.delta_secs();
            if *local_timer >= 0.5 {
                *local_timer = 0.0;
                let _ = data.socket.send_to(b"JOIN", data.peer_addr.unwrap_or("127.0.0.1:50505".parse().unwrap()));
            }
        }
        return;
    }

    *local_timer += time.delta_secs();
    if *local_timer >= 0.05 { // Send position 20 times per second
        *local_timer = 0.0;
        
        if let Some((transform, _)) = player_query.iter().next() {
            let x = transform.translation.x;
            let y = transform.translation.y;
            let level = level_state.current_level;
            
            let payload = format!("STATE {:.2} {:.2} {} 0 1", x, y, level);
            if let Some(peer_addr) = data.peer_addr {
                let _ = data.socket.send_to(payload.as_bytes(), peer_addr);
            }
        }
    }
}

pub fn multiplayer_receive_system(
    mut commands: Commands,
    mut socket: ResMut<MultiplayerSocket>,
    mut remote_player_query: Query<(Entity, &mut Transform, &mut RemotePlayer)>,
    level_state: Res<LevelState>,
    time: Res<Time>,
) {
    let Some(ref mut data) = socket.data else { return; };
    
    let mut buf = [0u8; 1024];
    while let Ok((size, addr)) = data.socket.recv_from(&mut buf) {
        data.last_received = time.elapsed_secs();
        let msg = String::from_utf8_lossy(&buf[..size]);
        let parts: Vec<&str> = msg.trim().split_whitespace().collect();
        if parts.is_empty() { continue; }
        
        match parts[0] {
            "JOIN" => {
                if data.is_host {
                    data.peer_addr = Some(addr);
                    data.is_connected = true;
                    let _ = data.socket.send_to(b"ACK", addr);
                }
            }
            "ACK" => {
                if !data.is_host {
                    data.is_connected = true;
                }
            }
            "STATE" => {
                if parts.len() >= 6 {
                    let rx_x = parts[1].parse::<f32>().unwrap_or(-350.0);
                    let rx_y = parts[2].parse::<f32>().unwrap_or(GROUND_Y);
                    let rx_level = parts[3].parse::<u32>().unwrap_or(1);
                    let rx_active = parts[5].parse::<u32>().unwrap_or(1) == 1;
                    
                    let cur_level = level_state.current_level;
                    
                    if let Some((_entity, mut transform, mut remote)) = remote_player_query.iter_mut().next() {
                        remote.level = rx_level;
                        remote.is_active = rx_active;
                        
                        if rx_level == cur_level && rx_active {
                            transform.translation = Vec3::new(rx_x, rx_y, 1.0);
                        } else {
                            transform.translation = Vec3::new(rx_x, -9999.0, 1.0);
                        }
                    } else {
                        let start_y = if rx_level == cur_level && rx_active { rx_y } else { -9999.0 };
                        commands.spawn((
                            LevelEntity, // automatically despawns on level change/reload
                            RemotePlayer { level: rx_level, is_active: rx_active },
                            Sprite::from_color(Color::srgb(1.0, 0.5, 0.0), PLAYER_SIZE), // Neon Orange Remote Player
                            Transform::from_xyz(rx_x, start_y, 1.0),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

// --- HUD Status ---

pub fn multiplayer_hud_system(
    mut commands: Commands,
    socket: Res<MultiplayerSocket>,
    hud_text_query: Query<Entity, With<MultiplayerHudText>>,
    remote_player_query: Query<&RemotePlayer>,
) {
    let Some(ref data) = socket.data else {
        for entity in &hud_text_query {
            commands.entity(entity).despawn();
        }
        return;
    };

    let status_str = if data.is_connected || data.peer_addr.is_some() {
        let peer_level = remote_player_query.iter().next().map(|r| r.level).unwrap_or(1);
        let code_label = if data.room_code.starts_with("DIRECT") { "DIRECT" } else { &data.room_code };
        format!("MULTIPLAYER: CONNECTED | ROOM: {} | PEER LEVEL: {}", code_label, peer_level)
    } else {
        let code_label = if data.room_code.starts_with("DIRECT") { "DIRECT" } else { &data.room_code };
        format!("MULTIPLAYER: WAITING FOR PEER | ROOM: {}", code_label)
    };

    if let Some(entity) = hud_text_query.iter().next() {
        commands.entity(entity).insert(Text::new(status_str));
    } else {
        commands.spawn((
            GameHUD,
            MultiplayerHudText,
            Text::new(status_str),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                ..default()
            },
        ));
    }
}

pub fn cleanup_multiplayer_connection(
    mut socket: ResMut<MultiplayerSocket>,
) {
    if let Some(ref data) = socket.data {
        if data.is_host && !data.room_code.starts_with("DIRECT") {
            close_room_on_github(&data.room_code);
        }
    }
    socket.data = None;
}
