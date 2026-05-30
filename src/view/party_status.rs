//! Projection of party roster state into renderable status data.

use bevy_ecs::world::World as EcsWorld;
use ratatui::style::Color;

use crate::{
    ecs::{ActionState, Classes, CombatStats, Name, PartyRoster, Renderable},
    rules::ClassKind,
};

/// Movement budget for a combatant during their active turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyMovementStatus {
    /// Meters already spent this turn.
    pub spent: i32,
    /// Meters still available this turn.
    pub remaining: i32,
}

/// One party member's status, projected for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyStatusMember {
    pub name: &'static str,
    /// Matches the member's `@` color so the box agrees with the map glyph.
    pub name_color: Color,
    pub level: u8,
    pub class_label: &'static str,
    pub hp: i32,
    pub max_hp: i32,
    /// Present only when ECS tracks authoritative movement (the active combatant).
    pub movement: Option<PartyMovementStatus>,
}

/// The party status projection in party order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyStatusView {
    pub members: Vec<PartyStatusMember>,
}

/// Projects the party roster into renderable status data, in party order.
pub fn from_ecs(world: &EcsWorld) -> PartyStatusView {
    let roster = world.resource::<PartyRoster>();
    let mut members = Vec::with_capacity(roster.members.len());

    for entity in roster.members.iter().copied() {
        let Ok(entity_ref) = world.get_entity(entity) else {
            continue;
        };
        let Some(name) = entity_ref.get::<Name>() else {
            continue;
        };
        let Some(classes) = entity_ref.get::<Classes>() else {
            continue;
        };
        let Some(stats) = entity_ref.get::<CombatStats>() else {
            continue;
        };
        let Some(renderable) = entity_ref.get::<Renderable>() else {
            continue;
        };

        let movement = entity_ref.get::<ActionState>().map(|action| {
            let remaining = action.remaining_movement.max(0);
            let spent = (stats.speed_m - action.remaining_movement).max(0);
            PartyMovementStatus { spent, remaining }
        });

        members.push(PartyStatusMember {
            name: name.0,
            name_color: renderable.color,
            level: classes.level,
            class_label: class_label(classes.primary),
            hp: stats.hp.max(0),
            max_hp: stats.max_hp.max(0),
            movement,
        });
    }

    PartyStatusView { members }
}

fn class_label(kind: ClassKind) -> &'static str {
    match kind {
        ClassKind::Fighter => "Fighter",
        ClassKind::Rogue => "Rogue",
        ClassKind::Cleric => "Cleric",
        ClassKind::Wizard => "Wizard",
    }
}

#[cfg(test)]
mod tests {
    use super::{PartyMovementStatus, from_ecs};
    use crate::{
        app::AppState,
        ecs::{ActionState, PartyRoster, Renderable},
    };
    use ratatui::style::Color;

    #[test]
    fn members_appear_in_party_roster_order() {
        let state = AppState::default();
        let view = from_ecs(&state.ecs_world);

        let names = view
            .members
            .iter()
            .map(|member| member.name)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Mara", "Ilyra", "Tovin", "Sable"]);
    }

    #[test]
    fn name_colors_match_renderable_color() {
        let state = AppState::default();
        let view = from_ecs(&state.ecs_world);
        let roster = state.ecs_world.resource::<PartyRoster>();

        for (member, entity) in view.members.iter().zip(roster.members.iter().copied()) {
            let expected = state.ecs_world.get::<Renderable>(entity).unwrap().color;
            assert_eq!(member.name_color, expected);
        }
        assert_eq!(view.members[0].name_color, Color::Cyan);
    }

    #[test]
    fn level_class_and_hp_are_populated() {
        let state = AppState::default();
        let view = from_ecs(&state.ecs_world);

        let mara = view.members[0];
        assert_eq!(mara.level, 1);
        assert_eq!(mara.class_label, "Fighter");
        assert_eq!(mara.hp, 12);
        assert_eq!(mara.max_hp, 12);
    }

    #[test]
    fn explore_mode_has_no_movement() {
        let state = AppState::default();
        let view = from_ecs(&state.ecs_world);

        assert!(view.members.iter().all(|member| member.movement.is_none()));
    }

    #[test]
    fn battle_movement_only_reported_for_members_with_action_state() {
        let mut state = AppState::default();
        let leader = state.ecs_world.resource::<PartyRoster>().members[0];
        state.ecs_world.entity_mut(leader).insert(ActionState {
            remaining_movement: 4,
            has_attacked: false,
        });

        let view = from_ecs(&state.ecs_world);

        // Mara has speed 9; with 4m remaining she has spent 5m.
        assert_eq!(
            view.members[0].movement,
            Some(PartyMovementStatus {
                spent: 5,
                remaining: 4
            })
        );
        assert!(
            view.members[1..]
                .iter()
                .all(|member| member.movement.is_none())
        );
    }
}
