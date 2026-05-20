use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::{
    ARENA_H, ARENA_W, CAPTURE_DISTANCE, HIDER_RADIUS, HIDER_SPEED, NUM_HIDERS, NUM_SEEKERS,
    SEEKER_RADIUS, SEEKER_SPEED, SIGHT_RANGE, SPAWN_CLEARANCE,
};

const OBSTACLES: [(Vec2, Vec2); 5] = [
    (Vec2::new(-310.0, 145.0), Vec2::new(250.0, 26.0)),
    (Vec2::new(265.0, -155.0), Vec2::new(300.0, 26.0)),
    (Vec2::new(20.0, 205.0), Vec2::new(30.0, 175.0)),
    (Vec2::new(-70.0, -245.0), Vec2::new(250.0, 24.0)),
    (Vec2::new(385.0, 120.0), Vec2::new(28.0, 185.0)),
];

const AGENT_GROUPS: CollisionGroups = CollisionGroups::new(Group::GROUP_1, Group::GROUP_2);
const WALL_GROUPS: CollisionGroups = CollisionGroups::new(Group::GROUP_2, Group::GROUP_1);
const EDGE_AVOID_DISTANCE: f32 = 120.0;
const EDGE_AVOID_WEIGHT: f32 = 2.4;
const OBSTACLE_AVOID_DISTANCE: f32 = 58.0;
const OBSTACLE_AVOID_WEIGHT: f32 = 2.0;
const HIDER_SEPARATION_RADIUS: f32 = 34.0;
const HIDER_SEPARATION_WEIGHT: f32 = 1.4;
const SEEKER_AVOID_WEIGHT: f32 = 3.2;
const STRAFE_WEIGHT: f32 = 0.8;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    spawn_background(&mut commands);
    spawn_boundaries(&mut commands);
    spawn_obstacles(&mut commands);

    let mut rng = rand::thread_rng();

    for _ in 0..NUM_HIDERS {
        let pos = random_position(&mut rng);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.2, 0.6, 1.0),
                    custom_size: Some(Vec2::splat(HIDER_RADIUS * 2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 2.0),
                ..default()
            },
            Hider { captured: false },
            DesiredVelocity { value: Vec2::ZERO },
            RigidBody::KinematicPositionBased,
            Collider::ball(HIDER_RADIUS),
            AGENT_GROUPS,
            character_controller(),
        ));
    }

    for _ in 0..NUM_SEEKERS {
        let pos = random_position(&mut rng);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.25, 0.25),
                    custom_size: Some(Vec2::splat(SEEKER_RADIUS * 2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 2.0),
                ..default()
            },
            Seeker,
            DesiredVelocity { value: Vec2::ZERO },
            RigidBody::KinematicPositionBased,
            Collider::ball(SEEKER_RADIUS),
            AGENT_GROUPS,
            character_controller(),
        ));
    }

    commands.spawn((
        TextBundle::from_section(
            "starting simulation",
            TextStyle {
                font_size: 22.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            top: Val::Px(12.0),
            ..default()
        }),
        StatsText,
    ));
}

fn spawn_background(commands: &mut Commands) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.055, 0.064, 0.078),
            custom_size: Some(Vec2::new(ARENA_W, ARENA_H)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..default()
    });
}

fn spawn_boundaries(commands: &mut Commands) {
    let thickness = 16.0;

    let walls = [
        (Vec2::new(0.0, ARENA_H / 2.0), Vec2::new(ARENA_W, thickness)),
        (
            Vec2::new(0.0, -ARENA_H / 2.0),
            Vec2::new(ARENA_W, thickness),
        ),
        (
            Vec2::new(-ARENA_W / 2.0, 0.0),
            Vec2::new(thickness, ARENA_H),
        ),
        (Vec2::new(ARENA_W / 2.0, 0.0), Vec2::new(thickness, ARENA_H)),
    ];

    for (pos, size) in walls {
        spawn_wall(commands, pos, size);
    }
}

fn spawn_obstacles(commands: &mut Commands) {
    for (pos, size) in OBSTACLES {
        spawn_wall(commands, pos, size);
    }
}

fn spawn_wall(commands: &mut Commands, pos: Vec2, size: Vec2) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.48, 0.5, 0.54),
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 1.0),
            ..default()
        },
        Wall,
        RigidBody::Fixed,
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        WALL_GROUPS,
    ));
}

fn character_controller() -> KinematicCharacterController {
    KinematicCharacterController {
        filter_groups: Some(AGENT_GROUPS),
        ..default()
    }
}

fn random_position(rng: &mut impl Rng) -> Vec2 {
    for _ in 0..1_000 {
        let pos = Vec2::new(
            rng.gen_range(-ARENA_W * 0.42..ARENA_W * 0.42),
            rng.gen_range(-ARENA_H * 0.42..ARENA_H * 0.42),
        );

        if is_spawn_clear(pos) {
            return pos;
        }
    }

    Vec2::ZERO
}

fn is_spawn_clear(pos: Vec2) -> bool {
    OBSTACLES.iter().all(|(wall_pos, wall_size)| {
        !point_overlaps_rect(pos, *wall_pos, *wall_size, SPAWN_CLEARANCE)
    })
}

fn point_overlaps_rect(point: Vec2, rect_center: Vec2, rect_size: Vec2, clearance: f32) -> bool {
    let half_size = rect_size / 2.0 + Vec2::splat(clearance);

    point.x >= rect_center.x - half_size.x
        && point.x <= rect_center.x + half_size.x
        && point.y >= rect_center.y - half_size.y
        && point.y <= rect_center.y + half_size.y
}

pub fn hider_policy(
    seekers: Query<&Transform, With<Seeker>>,
    mut hiders: Query<(&Hider, &Transform, &mut DesiredVelocity), Without<Seeker>>,
) {
    let seeker_positions: Vec<Vec2> = seekers
        .iter()
        .map(|transform| transform.translation.truncate())
        .collect();
    let hider_positions: Vec<Vec2> = hiders
        .iter()
        .filter(|(hider, _, _)| !hider.captured)
        .map(|(_, transform, _)| transform.translation.truncate())
        .collect();

    for (hider, transform, mut velocity) in hiders.iter_mut() {
        if hider.captured {
            velocity.value = Vec2::ZERO;
            continue;
        }

        let pos = transform.translation.truncate();
        let mut steering = edge_avoidance(pos) * EDGE_AVOID_WEIGHT;
        steering += obstacle_avoidance(pos) * OBSTACLE_AVOID_WEIGHT;
        steering += hider_separation(pos, &hider_positions) * HIDER_SEPARATION_WEIGHT;

        let mut nearest_seeker = None;
        let mut nearest_distance = f32::MAX;

        for seeker_pos in &seeker_positions {
            let away = pos - *seeker_pos;
            let distance = away.length().max(1.0);
            let threat = ((SIGHT_RANGE - distance).max(0.0) / SIGHT_RANGE).powi(2);
            steering += away.normalize_or_zero() * (0.25 + threat * SEEKER_AVOID_WEIGHT);

            if distance < nearest_distance {
                nearest_seeker = Some(*seeker_pos);
                nearest_distance = distance;
            }
        }

        if let Some(seeker_pos) = nearest_seeker {
            let away = (pos - seeker_pos).normalize_or_zero();
            let side = if (pos.x * 0.013 + pos.y * 0.017).sin() >= 0.0 {
                1.0
            } else {
                -1.0
            };
            steering += Vec2::new(-away.y, away.x) * side * STRAFE_WEIGHT;
        }

        velocity.value = steering.normalize_or_zero() * HIDER_SPEED;
    }
}

fn edge_avoidance(pos: Vec2) -> Vec2 {
    let left = -ARENA_W / 2.0;
    let right = ARENA_W / 2.0;
    let bottom = -ARENA_H / 2.0;
    let top = ARENA_H / 2.0;
    let mut force = Vec2::ZERO;

    force.x += edge_push(pos.x - left);
    force.x -= edge_push(right - pos.x);
    force.y += edge_push(pos.y - bottom);
    force.y -= edge_push(top - pos.y);

    force
}

fn edge_push(distance: f32) -> f32 {
    if distance >= EDGE_AVOID_DISTANCE {
        0.0
    } else {
        ((EDGE_AVOID_DISTANCE - distance) / EDGE_AVOID_DISTANCE).powi(2)
    }
}

fn obstacle_avoidance(pos: Vec2) -> Vec2 {
    let mut force = Vec2::ZERO;

    for (center, size) in OBSTACLES {
        let closest = Vec2::new(
            pos.x
                .clamp(center.x - size.x / 2.0, center.x + size.x / 2.0),
            pos.y
                .clamp(center.y - size.y / 2.0, center.y + size.y / 2.0),
        );
        let away = pos - closest;
        let distance = away.length();

        if distance < OBSTACLE_AVOID_DISTANCE {
            let direction = if distance > 0.001 {
                away / distance
            } else {
                (pos - center).normalize_or_zero()
            };
            let strength = ((OBSTACLE_AVOID_DISTANCE - distance) / OBSTACLE_AVOID_DISTANCE).powi(2);
            force += direction * strength;
        }
    }

    force
}

fn hider_separation(pos: Vec2, hider_positions: &[Vec2]) -> Vec2 {
    let mut force = Vec2::ZERO;
    let separation_radius_sq = HIDER_SEPARATION_RADIUS * HIDER_SEPARATION_RADIUS;

    for other_pos in hider_positions {
        let away = pos - *other_pos;
        let distance_sq = away.length_squared();

        if distance_sq > 0.001 && distance_sq < separation_radius_sq {
            let distance = distance_sq.sqrt();
            let strength = ((HIDER_SEPARATION_RADIUS - distance) / HIDER_SEPARATION_RADIUS).powi(2);
            force += away / distance * strength;
        }
    }

    force
}

pub fn tick_control_timer(time: Res<Time>, mut control_timer: ResMut<ControlTimer>) {
    control_timer.timer.tick(time.delta());
}

pub fn control_tick_ready(control_timer: Res<ControlTimer>) -> bool {
    control_timer.timer.just_finished()
}

pub fn seeker_policy(
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
    hiders: Query<(Entity, &Hider, &Transform), Without<Seeker>>,
    mut seekers: Query<(&Transform, &mut DesiredVelocity), With<Seeker>>,
) {
    let hider_targets: Vec<Vec2> = hiders
        .iter()
        .filter(|(_, hider, _)| !hider.captured)
        .map(|(_, _, transform)| transform.translation.truncate())
        .collect();

    for (transform, mut velocity) in seekers.iter_mut() {
        let pos = transform.translation.truncate();
        let mut nearest_visible = None;
        let mut nearest_distance = SIGHT_RANGE * SIGHT_RANGE;

        for target in &hider_targets {
            let distance = pos.distance_squared(*target);

            if distance < nearest_distance && has_line_of_sight(&rapier_context, pos, *target) {
                nearest_visible = Some(*target);
                nearest_distance = distance;
            }
        }

        if let Some(target) = nearest_visible {
            velocity.value = (target - pos).normalize_or_zero() * SEEKER_SPEED;
        } else {
            let angle = time.elapsed_seconds() * 1.2 + pos.x * 0.01;
            velocity.value = Vec2::new(angle.cos(), angle.sin()) * SEEKER_SPEED * 0.45;
        }
    }
}

fn has_line_of_sight(rapier_context: &RapierContext, seeker_pos: Vec2, hider_pos: Vec2) -> bool {
    let ray = hider_pos - seeker_pos;
    let distance = ray.length();

    if distance > SIGHT_RANGE {
        return false;
    }

    let direction = ray.normalize_or_zero();

    if direction == Vec2::ZERO {
        return true;
    }

    let filter = QueryFilter::only_fixed();

    match rapier_context.cast_ray(seeker_pos, direction, distance, true, filter) {
        Some((_wall_entity, _toi)) => false,
        None => true,
    }
}

pub fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&DesiredVelocity, &mut KinematicCharacterController), Without<Wall>>,
) {
    let dt = time.delta_seconds();

    for (velocity, mut controller) in query.iter_mut() {
        controller.translation = Some(velocity.value * dt);
    }
}

pub fn update_agent_visuals(mut agents: Query<(&DesiredVelocity, &mut Transform), Without<Wall>>) {
    for (velocity, mut transform) in agents.iter_mut() {
        if velocity.value.length_squared() > 1.0 {
            transform.rotation = Quat::from_rotation_z(velocity.value.y.atan2(velocity.value.x));
        }
    }
}

pub fn capture_system(
    mut commands: Commands,
    seekers: Query<&Transform, With<Seeker>>,
    mut hiders: Query<
        (
            Entity,
            &mut Hider,
            &Transform,
            &mut Sprite,
            &mut DesiredVelocity,
        ),
        Without<Seeker>,
    >,
) {
    let seeker_positions: Vec<Vec2> = seekers
        .iter()
        .map(|transform| transform.translation.truncate())
        .collect();

    for (entity, mut hider, transform, mut sprite, mut velocity) in hiders.iter_mut() {
        if hider.captured {
            continue;
        }

        let pos = transform.translation.truncate();
        let captured = seeker_positions
            .iter()
            .any(|seeker_pos| pos.distance(*seeker_pos) < CAPTURE_DISTANCE);

        if captured {
            hider.captured = true;
            velocity.value = Vec2::ZERO;
            sprite.color = Color::srgb(0.25, 0.25, 0.25);
            sprite.custom_size = Some(Vec2::splat(HIDER_RADIUS * 1.4));
            commands
                .entity(entity)
                .remove::<(RigidBody, Collider, KinematicCharacterController)>();
        }
    }
}

pub fn keep_inside_arena(mut query: Query<&mut Transform, Without<Wall>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x = transform
            .translation
            .x
            .clamp(-ARENA_W / 2.0 + 20.0, ARENA_W / 2.0 - 20.0);
        transform.translation.y = transform
            .translation
            .y
            .clamp(-ARENA_H / 2.0 + 20.0, ARENA_H / 2.0 - 20.0);
    }
}

pub fn update_stats_text(
    diagnostics: Res<DiagnosticsStore>,
    hiders: Query<&Hider>,
    mut query: Query<&mut Text, With<StatsText>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    let (alive, captured) = hiders.iter().fold((0, 0), |(alive, captured), hider| {
        if hider.captured {
            (alive, captured + 1)
        } else {
            (alive + 1, captured)
        }
    });

    for mut text in query.iter_mut() {
        text.sections[0].value =
            format!("alive hiders: {alive} | captured: {captured} | fps: {fps:.0}");
    }
}
