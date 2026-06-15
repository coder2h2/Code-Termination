mod constants;
mod components;
mod resources;
mod helpers;
mod systems;

use bevy::prelude::*;
use bevy::window::{WindowMode, MonitorSelection};

use resources::*;
use helpers::{load_achievements, run_auto_update};
use systems::title_screen::*;
use systems::achievements::*;
use systems::settings::*;
use systems::death_screen::*;
use systems::demo_complete::*;
use systems::player::*;
use systems::enemy::*;
use systems::gameplay::*;
use systems::dlc_menu::*;
use systems::boss_transition::*;
use systems::mode_select::*;

fn main() {
    run_auto_update();
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.14)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Code-Termination".into(),
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .insert_resource(TutorialState::default())
        .insert_resource(PendingGameLoad::default())
        .insert_resource(MenuSelection::default())
        .insert_resource(load_achievements())
        .insert_resource(OverclockState::default())
        .insert_resource(LevelState::default())
        .insert_resource(HackerMode::default())
        .add_systems(Startup, setup)
        // Title screen
        .add_systems(OnEnter(AppState::TitleScreen), (reset_virtual_time_system, setup_title_screen))
        .add_systems(OnExit(AppState::TitleScreen), cleanup_title_screen)
        .add_systems(Update, title_button_system.run_if(in_state(AppState::TitleScreen)))
        // Achievements screen
        .add_systems(OnEnter(AppState::Achievements), setup_achievements_screen)
        .add_systems(OnExit(AppState::Achievements), cleanup_achievements_screen)
        .add_systems(Update, achievements_screen_system.run_if(in_state(AppState::Achievements)))
        // Game
        .add_systems(
            OnTransition {
                exited: AppState::TitleScreen,
                entered: AppState::Game,
            },
            reset_player_system,
        )
        .add_systems(
            OnTransition {
                exited: AppState::BossTransition,
                entered: AppState::Game,
            },
            reset_player_system,
        )
        .add_systems(
            OnTransition {
                exited: AppState::ModeSelect,
                entered: AppState::Game,
            },
            reset_player_system,
        )
        // Boss Transition Screen
        .add_systems(OnEnter(AppState::BossTransition), setup_boss_transition)
        .add_systems(OnExit(AppState::BossTransition), cleanup_boss_transition)
        .add_systems(Update, boss_transition_system.run_if(in_state(AppState::BossTransition)))
        // Mode Select Screen
        .add_systems(OnEnter(AppState::ModeSelect), setup_mode_select)
        .add_systems(OnExit(AppState::ModeSelect), cleanup_mode_select)
        .add_systems(Update, mode_select_button_system.run_if(in_state(AppState::ModeSelect)))
        .add_systems(Update, toggle_settings_menu)
        .add_systems(Update, (
            (
                move_player,
                jump_player,
                apply_velocity,
                update_glitch,
                handle_damage,
                resolve_collisions,
                update_overclock,
                update_enemies,
                update_lasers,
                check_gate_collision,
            ).chain(),
            update_hud,
            auto_save_system,
        ).run_if(in_state(AppState::Game)))
        .add_systems(Update, update_achievement_toasts)
        .add_systems(OnExit(AppState::Game), save_game_state)
        // Pause/Settings menu
        .add_systems(OnEnter(AppState::Settings), setup_settings_menu)
        .add_systems(OnExit(AppState::Settings), cleanup_settings_menu)
        .add_systems(Update, settings_button_system.run_if(in_state(AppState::Settings)))
        // Death screen
        .add_systems(OnEnter(AppState::DeathScreen), setup_death_screen)
        .add_systems(OnExit(AppState::DeathScreen), cleanup_death_screen)
        .add_systems(Update, death_screen_button_system.run_if(in_state(AppState::DeathScreen)))
        .add_systems(
            OnTransition {
                exited: AppState::DeathScreen,
                entered: AppState::Game,
            },
            reset_player_system,
        )
        // Demo Complete screen
        .add_systems(OnEnter(AppState::DemoComplete), setup_demo_complete)
        .add_systems(OnExit(AppState::DemoComplete), cleanup_demo_complete)
        .add_systems(Update, demo_complete_system.run_if(in_state(AppState::DemoComplete)))
        // DLC screen
        .add_systems(OnEnter(AppState::DlcMenu), setup_dlc_screen)
        .add_systems(OnExit(AppState::DlcMenu), cleanup_dlc_screen)
        .add_systems(Update, dlc_screen_system.run_if(in_state(AppState::DlcMenu)))
        .run();
}
