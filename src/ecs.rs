use bevy_ecs::prelude::*;
use ratatui::style::Color;

use crate::data::grid::{ORIGIN, Vector};

pub const SIGN_POSITION: Vector = Vector { x: 4, y: 1 };

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position(pub Vector);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Renderable {
    pub glyph: char,
    pub color: Color,
    pub bold: bool,
    pub z: i32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickToWalk;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalkTarget {
    pub destination: Vector,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlFocus {
    pub entity: Entity,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewFocus {
    pub center: Vector,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PendingWalkDestination(pub Option<Vector>);

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ActiveWalkDestination(pub Option<Vector>);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct RenderableEntities {
    pub entities: Vec<Entity>,
}

pub fn spawn_initial_entities(world: &mut World) {
    let player = world
        .spawn((
            Player,
            ClickToWalk,
            Position(ORIGIN),
            Renderable {
                glyph: '@',
                color: Color::White,
                bold: true,
                z: 100,
            },
        ))
        .id();
    let sign = world
        .spawn((
            Position(SIGN_POSITION),
            Renderable {
                glyph: '|',
                color: Color::Rgb(139, 69, 19),
                bold: false,
                z: 10,
            },
        ))
        .id();

    world.insert_resource(ControlFocus { entity: player });
    world.insert_resource(ViewFocus { center: ORIGIN });
    world.insert_resource(PendingWalkDestination::default());
    world.insert_resource(ActiveWalkDestination::default());
    world.insert_resource(RenderableEntities {
        entities: vec![player, sign],
    });
}

pub fn movement_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            assign_walk_destination_system,
            walk_system,
            sync_view_focus_system,
        )
            .chain(),
    );
    schedule
}

pub fn assign_walk_destination_system(world: &mut World) {
    let destination = world.resource_mut::<PendingWalkDestination>().0.take();
    let Some(destination) = destination else {
        return;
    };

    let focused = world.resource::<ControlFocus>().entity;
    if world.get::<ClickToWalk>(focused).is_some() {
        world.entity_mut(focused).insert(WalkTarget { destination });
        world.resource_mut::<ActiveWalkDestination>().0 = Some(destination);
    }
}

pub fn walk_system(world: &mut World) {
    let mut arrived = Vec::new();
    let mut query = world.query::<(Entity, &mut Position, &WalkTarget)>();

    for (entity, mut position, target) in query.iter_mut(world) {
        if position.0 == target.destination {
            arrived.push((entity, target.destination));
            continue;
        }

        position.0 = step_toward(position.0, target.destination);

        if position.0 == target.destination {
            arrived.push((entity, target.destination));
        }
    }

    drop(query);

    let focused = world.resource::<ControlFocus>().entity;
    for (entity, destination) in arrived {
        world.entity_mut(entity).remove::<WalkTarget>();
        let active = &mut world.resource_mut::<ActiveWalkDestination>().0;
        if entity == focused && *active == Some(destination) {
            *active = None;
        }
    }
}

pub fn sync_view_focus_system(world: &mut World) {
    let focused = world.resource::<ControlFocus>().entity;
    let Some(position) = world.get::<Position>(focused).map(|position| position.0) else {
        return;
    };

    world.resource_mut::<ViewFocus>().center = position;
}

fn step_toward(current: Vector, destination: Vector) -> Vector {
    Vector {
        x: current
            .x
            .saturating_add((destination.x - current.x).signum()),
        y: current
            .y
            .saturating_add((destination.y - current.y).signum()),
    }
}
