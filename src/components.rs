use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct JumpState {
    pub jumps_remaining: u32,
    pub max_jumps: u32,
    pub is_smashing: bool,
}

impl Default for JumpState {
    fn default() -> Self {
        Self {
            jumps_remaining: 2,
            max_jumps: 2,
            is_smashing: false,
        }
    }
}

#[derive(Component)]
pub struct DashState {
    pub last_a_press: f32,
    pub last_d_press: f32,
    pub dash_timer: f32,
    pub dash_dir: f32,
    pub air_dash_used: bool,
}

impl Default for DashState {
    fn default() -> Self {
        Self {
            last_a_press: -10.0,
            last_d_press: -10.0,
            dash_timer: 0.0,
            dash_dir: 0.0,
            air_dash_used: false,
        }
    }
}

#[derive(Component)]
pub struct GlitchState {
    pub timer: f32,
    pub cooldown: f32,
    pub just_ended: bool,
    pub prev_shake: Vec2,
}

impl Default for GlitchState {
    fn default() -> Self {
        Self {
            timer: 0.0,
            cooldown: 0.0,
            just_ended: false,
            prev_shake: Vec2::ZERO,
        }
    }
}

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct GameHUD;

#[derive(Component)]
pub struct Spike;

#[derive(Component)]
pub struct Enemy {
    pub patrol_left: f32,
    pub patrol_right: f32,
    pub speed: f32,
    pub direction: f32,
    pub is_destroyed: bool,
}

#[derive(Component)]
pub struct RamState {
    pub current: u32,
    pub max: u32,
    pub invulnerability_timer: f32,
}

impl Default for RamState {
    fn default() -> Self {
        Self {
            current: 6,
            max: 6,
            invulnerability_timer: 0.0,
        }
    }
}

#[derive(Component)]
pub struct RamHUD;

#[derive(Component)]
pub struct TutorialHUD;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
pub struct BossSpeechText;

#[derive(Component)]
pub struct GatewayConsole;

#[derive(Component)]
pub struct LevelHUD;

#[derive(Component)]
pub struct BossHealthBar;

#[derive(Component)]
pub struct BossHealthBarFill;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossAttackState {
    Intro,
    Patrol,
    PrepareRam,
    Ram,
    PrepareSweep,
    Sweep,
    PrepareLaser,
    Laser,
    Cooldown,
}

#[derive(Component)]
pub struct Boss {
    pub health: u32,
    pub invulnerable_timer: f32,
    pub state: BossAttackState,
    pub state_timer: f32,
    pub attack_cooldown_timer: f32,
    pub ram_direction: f32,
    pub sweep_target_x: f32,
}

#[derive(Component)]
pub struct Laser {
    pub timer: f32,
    pub is_active: bool,
}

#[derive(Component)]
pub struct DeathScreenUI;

#[derive(Component)]
pub struct DemoCompleteUI;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DeathScreenButtonAction {
    Respawn,
    Title,
}

#[derive(Component)]
pub struct TitleScreenUI;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum TitleButtonAction {
    Play,
    NewGame,
    Multiplayer,
    Achievements,
    Dlc,
    Quit,
}

#[derive(Component)]
pub struct RemotePlayer {
    pub level: u32,
    pub is_active: bool,
}

#[derive(Component)]
pub struct MultiplayerHudText;

#[derive(Component)]
pub struct MultiplayerMenuUI;

#[derive(Component)]
pub struct HostWaitingUI;

#[derive(Component)]
pub struct JoinInputUI;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MultiplayerButtonAction {
    Host,
    Join,
    Back,
    CancelHost,
    BackToMenu,
}

#[derive(Component)]
pub struct AchievementsMenuUI;

#[derive(Component)]
pub struct ToastContainer;

#[derive(Component)]
pub struct AchievementToast {
    pub timer: f32,
}

#[derive(Component)]
pub struct OverclockHUD;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CpuClockMode {
    #[default]
    Normal,
    Overclocked,
    Underclocked,
}

#[derive(Component)]
pub struct SettingsMenuUI;

#[derive(Component, Clone, Copy)]
pub enum SettingsButtonAction {
    Resume,
    Title,
    TutorialDone,
}
