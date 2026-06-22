use bevy::prelude::*;
use crate::components::CpuClockMode;

#[derive(Resource)]
pub struct TutorialState {
    pub visible: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self { visible: true }
    }
}

#[derive(Resource, Default)]
pub struct PendingGameLoad {
    pub should_load: bool,
}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub struct LevelState {
    pub current_level: u32,
}

impl Default for LevelState {
    fn default() -> Self {
        Self { current_level: 1 }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum AppState {
    #[default]
    TitleScreen,
    UserRegister,
    ModeSelect,
    Game,
    BossTransition,
    Settings,
    Achievements,
    DlcMenu,
    DeathScreen,
    DemoComplete,
    MultiplayerMenu,
    HostWaiting,
    JoinInput,
    WeaponDesigner,
    BattleArena,
    UserNameChange,
}

#[derive(Resource, Default)]
pub struct RoomCodeInput {
    pub code: String,
    pub status_message: String,
    pub error_message: String,
    pub is_connecting: bool,
}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub struct HackerMode {
    pub active: bool,
}

impl Default for HackerMode {
    fn default() -> Self {
        Self { active: false }
    }
}

#[derive(Resource, Default)]
pub struct MenuSelection {
    pub selected_index: usize,
}

#[derive(Resource, Default)]
pub struct OverclockState {
    pub mode: CpuClockMode,
    pub timer: f32,
    pub overclock_cooldown: f32,
    pub underclock_cooldown: f32,
}

#[derive(Resource, Default, Clone)]
pub struct Achievements {
    pub phase_shift: bool,
    pub turbo_charged: bool,
    pub airborne: bool,
    pub system_crash: bool,
    pub speed_daemon: bool,
    pub better_call_tech_support: bool,
    pub death_count: u32,
}

#[derive(Resource, Clone)]
pub struct CustomWeapon {
    pub name: String,
    pub damage_stat: u32,       // 1 to 10
    pub cooldown_stat: u32,     // 1 to 10
    pub speed_stat: u32,        // 1 to 10
    pub multishot_stat: u32,    // 1 to 5 (bullets)
    pub color_idx: u32,         // 0: Cyan, 1: Pink, 2: Yellow, 3: Green
    pub shoot_cooldown: f32,
}

impl Default for CustomWeapon {
    fn default() -> Self {
        Self {
            name: "KERNEL_BLASTER".to_string(),
            damage_stat: 3,
            cooldown_stat: 3,
            speed_stat: 5,
            multishot_stat: 1,
            color_idx: 0,
            shoot_cooldown: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
}

#[derive(Resource, Default)]
pub struct PlayerPowerUpState {
    pub overclock_timer: f32,
    pub shield_timer: f32,
}

#[derive(Resource, Default, Clone)]
pub struct UserProfile {
    pub username: String,
    pub uid: String,
}

