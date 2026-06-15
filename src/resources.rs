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
    ModeSelect,
    Game,
    BossTransition,
    Settings,
    Achievements,
    DlcMenu,
    DeathScreen,
    DemoComplete,
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
