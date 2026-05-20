mod components;
mod systems;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use components::*;
use systems::*;

pub const ARENA_W: f32 = 1180.0;
pub const ARENA_H: f32 = 760.0;
pub const NUM_HIDERS: usize = 150;
pub const NUM_SEEKERS: usize = 18;
pub const HIDER_RADIUS: f32 = 5.5;
pub const SEEKER_RADIUS: f32 = 7.0;
pub const HIDER_SPEED: f32 = 92.0;
pub const SEEKER_SPEED: f32 = 102.0;
pub const CAPTURE_DISTANCE: f32 = 14.0;
pub const SIGHT_RANGE: f32 = 390.0;
pub const SPAWN_CLEARANCE: f32 = 14.0;
pub const PHYSICS_HZ: f64 = 60.0;
pub const CONTROL_HZ: f32 = 12.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.028, 0.035)))
        .insert_resource(SimStats::default())
        .insert_resource(ControlTimer {
            timer: Timer::from_seconds(1.0 / CONTROL_HZ, TimerMode::Repeating),
        })
        .insert_resource(Time::<Fixed>::from_hz(PHYSICS_HZ))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Multi-Agent Pursuit-Evasion".to_string(),
                resolution: (ARENA_W, ARENA_H).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                tick_control_timer,
                hider_policy.run_if(control_tick_ready),
                seeker_policy.run_if(control_tick_ready),
                capture_system,
                update_agent_visuals,
                update_stats_text,
            ),
        )
        .add_systems(FixedUpdate, (apply_velocity, keep_inside_arena))
        .run();
}
