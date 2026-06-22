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
    pub is_battle: bool,
    pub cancel_token: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
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

#[derive(Resource)]
pub struct ClientDiscoveryChannel {
    pub tx: std::sync::mpsc::Sender<SocketAddr>,
    pub rx: std::sync::Mutex<std::sync::mpsc::Receiver<SocketAddr>>,
}

impl Default for ClientDiscoveryChannel {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            tx,
            rx: std::sync::Mutex::new(rx),
        }
    }
}

pub enum MultiplayerEvent {
    HostSuccess {
        u_socket: UdpSocket,
        room_code: String,
        is_battle: bool,
    },
    HostFailure(String),
    JoinSuccess {
        u_socket: UdpSocket,
        peer_addr: SocketAddr,
        room_code: String,
        is_battle: bool,
    },
    JoinFailure(String),
}

#[derive(Component)]
pub struct JoinInputText;

#[derive(Component)]
pub struct JoinStatusText;

#[derive(Component)]
pub struct HostWaitingRoomCodeText;



fn get_public_addr_from_stun(socket: &UdpSocket) -> Option<SocketAddr> {
    let stun_servers = [
        "stun.l.google.com:19302",
        "stun1.l.google.com:19302",
        "stun2.l.google.com:19302",
        "stun3.l.google.com:19302",
        "stun4.l.google.com:19302",
    ];

    let mut request = [0u8; 20];
    request[0] = 0x00; request[1] = 0x01; // Message Type: Binding Request
    request[2] = 0x00; request[3] = 0x00; // Message Length: 0
    request[4] = 0x21; request[5] = 0x12; request[6] = 0xA4; request[7] = 0x42; // Magic Cookie
    // Transaction ID (12 bytes)
    for (i, val) in request.iter_mut().enumerate().skip(8) {
        *val = i as u8;
    }

    let prev_timeout = socket.read_timeout().ok().flatten();
    let _ = socket.set_read_timeout(Some(std::time::Duration::from_millis(800)));

    for server in &stun_servers {
        use std::net::ToSocketAddrs;
        if let Ok(mut addrs) = server.to_socket_addrs() {
            if let Some(server_addr) = addrs.next() {
                if socket.send_to(&request, server_addr).is_err() {
                    continue;
                }

                let mut response = [0u8; 512];
                if let Ok((size, _)) = socket.recv_from(&mut response) {
                    if size >= 20 {
                        // Check message type (Binding Success Response: 0x0101)
                        let msg_type = ((response[0] as u16) << 8) | (response[1] as u16);
                        if msg_type == 0x0101 {
                            // Check transaction ID matches
                            if response[8..20] == request[8..20] {
                                // Parse attributes
                                let mut offset = 20;
                                let msg_length = (((response[2] as usize) << 8) | (response[3] as usize)) + 20;
                                let max_len = std::cmp::min(size, msg_length);

                                while offset + 4 <= max_len {
                                    let attr_type = ((response[offset] as u16) << 8) | (response[offset + 1] as u16);
                                    let attr_len = ((response[offset + 2] as usize) << 8) | (response[offset + 3] as usize);
                                    offset += 4;

                                    if offset + attr_len > max_len {
                                        break;
                                    }

                                    if attr_type == 0x0001 { // MAPPED-ADDRESS
                                        if attr_len >= 8 {
                                            let family = response[offset + 1];
                                            if family == 0x01 { // IPv4
                                                let port = ((response[offset + 2] as u16) << 8) | (response[offset + 3] as u16);
                                                let ip = std::net::Ipv4Addr::new(
                                                    response[offset + 4],
                                                    response[offset + 5],
                                                    response[offset + 6],
                                                    response[offset + 7],
                                                );
                                                let _ = socket.set_read_timeout(prev_timeout);
                                                return Some(SocketAddr::V4(std::net::SocketAddrV4::new(ip, port)));
                                            }
                                        }
                                    } else if attr_type == 0x0020 { // XOR-MAPPED-ADDRESS
                                        if attr_len >= 8 {
                                            let family = response[offset + 1];
                                            if family == 0x01 { // IPv4
                                                let raw_port = ((response[offset + 2] as u16) << 8) | (response[offset + 3] as u16);
                                                let port = raw_port ^ 0x2112; // XOR with magic cookie port part
                                                let ip_bytes = [
                                                    response[offset + 4] ^ 0x21,
                                                    response[offset + 5] ^ 0x12,
                                                    response[offset + 6] ^ 0xA4,
                                                    response[offset + 7] ^ 0x42,
                                                ];
                                                let ip = std::net::Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
                                                let _ = socket.set_read_timeout(prev_timeout);
                                                return Some(SocketAddr::V4(std::net::SocketAddrV4::new(ip, port)));
                                            }
                                        }
                                    }

                                    // Attribute values are aligned on 4-byte boundaries
                                    offset += (attr_len + 3) & !3;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = socket.set_read_timeout(prev_timeout);
    None
}



fn host_room_via_proxy(room_code: &str, ip_port: &str) -> Result<(), String> {
    let url = format!(
        "https://code-termination-proxy.gideon-a-e-laurie.workers.dev/rooms/{}",
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


fn close_room_via_proxy(room_code: &str) {
    let url = format!(
        "https://code-termination-proxy.gideon-a-e-laurie.workers.dev/rooms/{}",
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
    close_room_via_proxy(room_code);
}

fn fetch_room_address(room_code: &str) -> Result<String, String> {
    let url = format!(
        "https://code-termination-proxy.gideon-a-e-laurie.workers.dev/rooms/{}",
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
                if content.is_empty() || content.contains("404") || content.contains("Not Found") || content.contains("Error") {
                    Err("Room not found".to_string())
                } else {
                    Ok(content)
                }
            } else {
                Err("Failed to fetch room code from proxy".to_string())
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

        // Host Battle Button
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
            BorderColor::all(Color::srgb(0.0, 1.0, 1.0)), // Cyan border for battle
            MultiplayerButtonAction::HostBattle,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("HOST BATTLE ROOM"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 1.0)), // Cyan text
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
    profile: Res<UserProfile>,
) {
    let mut buttons = button_query.iter_mut().collect::<Vec<_>>();
    buttons.sort_by_key(|(_, _, action, _)| match action {
        MultiplayerButtonAction::Host => 0,
        MultiplayerButtonAction::HostBattle => 1,
        MultiplayerButtonAction::Join => 2,
        MultiplayerButtonAction::Back => 3,
        _ => 4,
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
        if *action == MultiplayerButtonAction::Host || *action == MultiplayerButtonAction::HostBattle || *action == MultiplayerButtonAction::Join {
            if profile.username.is_empty() {
                next_state.set(AppState::UserRegister);
                return;
            }
        }
        match action {
            MultiplayerButtonAction::Host => {
                host_game_start(channel, next_state, false);
            }
            MultiplayerButtonAction::HostBattle => {
                host_game_start(channel, next_state, true);
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
    is_battle: bool,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    if let Ok(mut rx_opt) = channel.rx.lock() {
        *rx_opt = Some(rx);
    }
    
    std::thread::spawn(move || {
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
        
        // Query public IP address via STUN first
        let stun_addr = get_public_addr_from_stun(&socket);
        let public_addr_str = if let Some(addr) = stun_addr {
            addr.to_string()
        } else {
            // Fallback: Query public IP address via curl
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
            format!("{}:{}", public_ip, bound_port)
        };
        
        let mut ip_port = public_addr_str;
        if is_battle {
            ip_port.push_str("|BATTLE");
        }
        
        let room_code = generate_room_code();
        match host_room_via_proxy(&room_code, &ip_port) {
            Ok(_) => {
                let _ = socket.set_nonblocking(true);
                let _ = tx.send(MultiplayerEvent::HostSuccess { u_socket: socket, room_code, is_battle });
            }
            Err(_) => {
                // Fallback to direct connection if proxy fails
                let _ = socket.set_nonblocking(true);
                let _ = tx.send(MultiplayerEvent::HostSuccess { 
                    u_socket: socket, 
                    room_code: format!("DIRECT ({})", ip_port),
                    is_battle,
                });
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
                        close_room_on_github(&format!("{}_client", data.room_code));
                    }
                    if let Some(ref token) = data.cancel_token {
                        token.store(true, std::sync::atomic::Ordering::SeqCst);
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
    discovery_channel: Res<ClientDiscoveryChannel>,
    time: Res<Time>,
    mut punch_timer: Local<f32>,
) {
    let Some(ref mut data) = socket.data else { return; };
    
    // 1. If we are the host and don't have peer_addr yet, try to retrieve it from the discovery channel
    if data.is_host && data.peer_addr.is_none() {
        if let Ok(rx) = discovery_channel.rx.lock() {
            if let Ok(addr) = rx.try_recv() {
                println!("[Multiplayer] Discovered client address: {}", addr);
                data.peer_addr = Some(addr);
            }
        }
    }
    
    // 2. If we have peer_addr but are not connected yet, host sends punch/ACK packets periodically
    if data.is_host && data.peer_addr.is_some() && !data.is_connected {
        *punch_timer += time.delta_secs();
        if *punch_timer >= 0.2 {
            *punch_timer = 0.0;
            if let Some(peer_addr) = data.peer_addr {
                // Send a PUNCH/ACK packet to punch a hole in host's NAT
                let _ = data.socket.send_to(b"ACK", peer_addr);
            }
        }
    }

    // 3. Receive incoming packets
    let mut buf = [0u8; 1024];
    if let Ok((size, addr)) = data.socket.recv_from(&mut buf) {
        let msg = String::from_utf8_lossy(&buf[..size]);
        if msg.trim() == "JOIN" {
            data.peer_addr = Some(addr);
            data.is_connected = true;
            for _ in 0..5 {
                let _ = data.socket.send_to(b"ACK", addr);
            }
            if data.is_battle {
                next_state.set(AppState::BattleArena);
            } else {
                next_state.set(AppState::Game);
            }
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
        let is_direct = room_code.contains('.') || room_code.contains(':');
        
        let addr_str = if is_direct {
            room_code.clone()
        } else {
            match fetch_room_address(&room_code) {
                Ok(a) => a,
                Err(e) => {
                    let _ = tx.send(MultiplayerEvent::JoinFailure(e));
                    return;
                }
            }
        };
        
        let mut clean_addr = addr_str.clone();
        let mut is_battle = false;
        if clean_addr.contains("|BATTLE") {
            is_battle = true;
            clean_addr = clean_addr.replace("|BATTLE", "");
        }
        
        let peer_addr = match clean_addr.parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => {
                let raw_ip = format!("{}:50505", clean_addr.trim());
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
        
        if !is_direct {
            // Resolve the client's public mapped address via STUN
            let stun_addr = get_public_addr_from_stun(&socket);
            let client_public_addr_str = if let Some(addr) = stun_addr {
                addr.to_string()
            } else {
                // Fallback: Query public IP address via curl
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
                let bound_port = socket.local_addr().map(|a| a.port()).unwrap_or(0);
                format!("{}:{}", public_ip, bound_port)
            };
            
            let client_room_code = format!("{}_client", room_code);
            let _ = host_room_via_proxy(&client_room_code, &client_public_addr_str);
        }
        
        let _ = socket.set_nonblocking(true);
        let _ = tx.send(MultiplayerEvent::JoinSuccess { u_socket: socket, peer_addr, room_code: if is_direct { room_code } else { clean_addr }, is_battle });
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
    discovery_channel: Res<ClientDiscoveryChannel>,
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
            MultiplayerEvent::HostSuccess { u_socket, room_code, is_battle } => {
                let cancel_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                
                socket.data = Some(MultiplayerSocketData {
                    socket: u_socket,
                    peer_addr: None,
                    is_host: true,
                    room_code: room_code.clone(),
                    last_received: time.elapsed_secs(),
                    is_connected: false,
                    is_battle,
                    cancel_token: Some(cancel_token.clone()),
                });

                // Spawn background thread to poll for client address if not direct connection
                if !room_code.starts_with("DIRECT") {
                    let tx_clone = discovery_channel.tx.clone();
                    let room_code_clone = room_code.clone();
                    std::thread::spawn(move || {
                        for _ in 0..90 { // 3 minutes timeout
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            if cancel_token.load(std::sync::atomic::Ordering::SeqCst) {
                                break;
                            }
                            match fetch_room_address(&format!("{}_client", room_code_clone)) {
                                Ok(addr_str) => {
                                    let mut clean_addr = addr_str.clone();
                                    if clean_addr.contains("|BATTLE") {
                                        clean_addr = clean_addr.replace("|BATTLE", "");
                                    }
                                    if let Ok(addr) = clean_addr.parse::<SocketAddr>() {
                                        let _ = tx_clone.send(addr);
                                        break;
                                    } else {
                                        let raw_ip = format!("{}:50505", clean_addr.trim());
                                        if let Ok(addr) = raw_ip.parse::<SocketAddr>() {
                                            let _ = tx_clone.send(addr);
                                            break;
                                        }
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                    });
                }
            }
            MultiplayerEvent::HostFailure(err) => {
                println!("[Multiplayer] Host failed: {}", err);
                next_state.set(AppState::MultiplayerMenu);
            }
            MultiplayerEvent::JoinSuccess { u_socket, peer_addr, room_code, is_battle } => {
                socket.data = Some(MultiplayerSocketData {
                    socket: u_socket,
                    peer_addr: Some(peer_addr),
                    is_host: false,
                    room_code,
                    last_received: time.elapsed_secs(),
                    is_connected: false,
                    is_battle,
                    cancel_token: None,
                });
                if is_battle {
                    next_state.set(AppState::BattleArena);
                } else {
                    next_state.set(AppState::Game);
                }
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
    profile: Res<UserProfile>,
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
            
            let payload = format!("STATE {:.2} {:.2} {} 0 1 {}", x, y, level, profile.username);
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
    mut remote_name_query: Query<(&ChildOf, &mut Text2d), With<RemotePlayerName>>,
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
                    let rx_username = if parts.len() >= 7 { parts[6].to_string() } else { "Player".to_string() };
                    
                    let cur_level = level_state.current_level;
                    
                    if let Some((entity, mut transform, mut remote)) = remote_player_query.iter_mut().next() {
                        remote.level = rx_level;
                        remote.is_active = rx_active;
                        
                        if rx_level == cur_level && rx_active {
                            transform.translation = Vec3::new(rx_x, rx_y, 1.0);
                        } else {
                            transform.translation = Vec3::new(rx_x, -9999.0, 1.0);
                        }
                        
                        for (parent, mut text) in &mut remote_name_query {
                            if parent.parent() == entity {
                                if text.0 != rx_username {
                                    text.0 = rx_username.clone();
                                }
                            }
                        }
                    } else {
                        let start_y = if rx_level == cur_level && rx_active { rx_y } else { -9999.0 };
                        let parent_entity = commands.spawn((
                            LevelEntity, // automatically despawns on level change/reload
                            RemotePlayer { level: rx_level, is_active: rx_active },
                            Sprite::from_color(Color::srgb(1.0, 0.5, 0.0), PLAYER_SIZE), // Neon Orange Remote Player
                            Transform::from_xyz(rx_x, start_y, 1.0),
                        )).id();
                        
                        commands.entity(parent_entity).with_children(|parent| {
                            parent.spawn((
                                RemotePlayerName,
                                Text2d::new(rx_username.clone()),
                                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                Transform::from_xyz(0.0, PLAYER_SIZE.y / 2.0 + 15.0, 2.0),
                            ));
                        });
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
        if data.is_battle {
            format!("MULTIPLAYER BATTLE: CONNECTED | ROOM: {}", code_label)
        } else {
            format!("MULTIPLAYER: CONNECTED | ROOM: {} | PEER LEVEL: {}", code_label, peer_level)
        }
    } else {
        let code_label = if data.room_code.starts_with("DIRECT") { "DIRECT" } else { &data.room_code };
        if data.is_battle {
            format!("MULTIPLAYER BATTLE: WAITING FOR PEER | ROOM: {}", code_label)
        } else {
            format!("MULTIPLAYER: WAITING FOR PEER | ROOM: {}", code_label)
        }
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
        if !data.room_code.starts_with("DIRECT") {
            if data.is_host {
                close_room_on_github(&data.room_code);
                close_room_on_github(&format!("{}_client", data.room_code));
            } else {
                close_room_on_github(&format!("{}_client", data.room_code));
            }
        }
        if let Some(ref token) = data.cancel_token {
            token.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
    socket.data = None;
}
