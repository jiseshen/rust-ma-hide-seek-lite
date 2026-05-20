use bevy::prelude::*;

#[derive(Component)]
pub struct Hider {
    pub captured: bool,
}

#[derive(Component)]
pub struct Seeker;

#[derive(Component)]
pub struct DesiredVelocity {
    pub value: Vec2,
}

#[derive(Component)]
pub struct StatsText;

#[derive(Component)]
pub struct Wall;

#[derive(Resource)]
pub struct ControlTimer {
    pub timer: Timer,
}
