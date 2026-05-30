use std::{
    collections::VecDeque,
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
};

use bevy_ecs::prelude::*;
use ratatui::style::Color;

use crate::{
    data::grid::{ORIGIN, Vector},
    rules::{
        Ability, AbilityScores, ArmorKind, CharacterTemplate, ClassKind, DamageDice, SpeciesKind,
        WeaponKind, proficiency_bonus, squirrel_template,
    },
};

pub const SIGN_POSITION: Vector = Vector { x: 4, y: 1 };
pub const PARTY_ID: u32 = 1;
pub const SQUIRREL_ENCOUNTER_ID: u32 = 1;
pub const ENCOUNTER_TRIGGER_M: i32 = 10;
pub const SQUIRREL_POSITIONS: [Vector; 3] = [
    Vector { x: -2, y: -24 },
    Vector { x: 0, y: -25 },
    Vector { x: 2, y: -24 },
];
pub const TURN_START_PAUSE_TICKS: u8 = 5;
pub const TURN_END_PAUSE_TICKS: u8 = 5;

const DEFAULT_FACING: Vector = Vector { x: 0, y: -1 };
const MAX_COMBAT_LOG_LINES: usize = 48;

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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackMoveTarget {
    pub target: Entity,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Name(pub &'static str);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Species(pub SpeciesKind);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Classes {
    pub primary: ClassKind,
    pub level: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Abilities(pub AbilityScores);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatStats {
    pub armor_class: i32,
    pub max_hp: i32,
    pub hp: i32,
    pub speed_m: i32,
    pub proficiency_bonus: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Weapon(WeaponKind),
    Armor(ArmorKind),
    Shield,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    pub items: Vec<Item>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Equipment {
    pub weapon: WeaponKind,
    pub armor: ArmorKind,
    pub shield: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
    Party,
    Hostile,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyMember {
    pub party_id: u32,
    pub order: usize,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncounterGroup {
    pub id: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostileAi;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackProfile {
    pub name: &'static str,
    pub attack_bonus: i32,
    pub damage: DamageDice,
    pub reach_m: i32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionState {
    pub remaining_movement: i32,
    pub has_attacked: bool,
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

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Explore,
    TurnBased,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::Explore
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct PartyRoster {
    pub party_id: u32,
    pub members: Vec<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnEntry {
    pub entity: Entity,
    pub initiative: i32,
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct TurnOrder {
    pub entries: Vec<TurnEntry>,
    pub current_index: usize,
    pub round: u32,
    pub encounter_group: Option<u32>,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TurnStartPause {
    pub remaining_ticks: u8,
    paused_this_tick: bool,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TurnEndPause {
    pub remaining_ticks: u8,
    pending_advance: bool,
}

impl TurnStartPause {
    pub fn waiting(self) -> bool {
        self.paused_this_tick
    }

    pub fn clear(&mut self) {
        self.remaining_ticks = 0;
        self.paused_this_tick = false;
    }
}

impl TurnEndPause {
    pub fn waiting(self) -> bool {
        self.pending_advance
    }

    pub fn start(&mut self) {
        self.remaining_ticks = TURN_END_PAUSE_TICKS;
        self.pending_advance = true;
    }

    pub fn clear(&mut self) {
        self.remaining_ticks = 0;
        self.pending_advance = false;
    }
}

impl TurnOrder {
    pub fn active(&self) -> Option<Entity> {
        self.entries
            .get(self.current_index.min(self.entries.len().saturating_sub(1)))
            .map(|entry| entry.entity)
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct EncounterConfig {
    pub trigger_radius_m: i32,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct CombatLog {
    lines: VecDeque<String>,
    file_path: Option<PathBuf>,
    file_error: Option<String>,
}

impl Default for CombatLog {
    fn default() -> Self {
        Self {
            lines: VecDeque::new(),
            file_path: None,
            file_error: None,
        }
    }
}

impl CombatLog {
    pub fn with_file(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        Ok(Self {
            file_path: Some(path),
            ..Self::default()
        })
    }

    pub fn set_file(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
        let path = path.into();
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;
        self.file_path = Some(path);
        self.file_error = None;
        Ok(())
    }

    pub fn push(&mut self, line: impl Into<String>) {
        let line = line.into();
        if let Err(error) = self.append_to_file(&line) {
            self.file_error = Some(error.to_string());
        }

        self.lines.push_back(line);
        while self.lines.len() > MAX_COMBAT_LOG_LINES {
            self.lines.pop_front();
        }
    }

    pub fn lines(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.lines.iter().map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    pub fn file_error(&self) -> Option<&str> {
        self.file_error.as_deref()
    }

    fn append_to_file(&self, line: &str) -> io::Result<()> {
        let Some(path) = &self.file_path else {
            return Ok(());
        };
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{line}")
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct DiceRng {
    state: u64,
    scripted: VecDeque<i32>,
}

impl Default for DiceRng {
    fn default() -> Self {
        Self {
            state: 0x5EED_5EED_F00D_BAAD,
            scripted: VecDeque::new(),
        }
    }
}

impl DiceRng {
    pub fn scripted(rolls: impl IntoIterator<Item = i32>) -> Self {
        Self {
            state: 0x5EED_5EED_F00D_BAAD,
            scripted: rolls.into_iter().collect(),
        }
    }

    pub fn queue_roll(&mut self, roll: i32) {
        self.scripted.push_back(roll);
    }

    pub fn roll_die(&mut self, sides: u8) -> i32 {
        if sides <= 1 {
            return 1;
        }
        if let Some(roll) = self.scripted.pop_front() {
            return roll.clamp(1, sides as i32);
        }

        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) % sides as u64 + 1) as i32
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LastLeaderDirection(pub Vector);

pub fn spawn_initial_entities(world: &mut World) {
    let mut renderables = Vec::new();
    let mut party = Vec::new();
    let colors = [Color::Cyan, Color::Yellow, Color::Magenta, Color::LightBlue];
    let positions = [
        ORIGIN,
        Vector { x: -1, y: 1 },
        Vector { x: 0, y: 2 },
        Vector { x: 1, y: 1 },
    ];

    for (order, template) in crate::rules::starter_party().into_iter().enumerate() {
        let entity = spawn_party_member(world, template, order, positions[order], colors[order]);
        if order == 0 {
            world.entity_mut(entity).insert(Player);
        }
        renderables.push(entity);
        party.push(entity);
    }

    let sign = world
        .spawn((
            Name("Sign"),
            Position(SIGN_POSITION),
            Renderable {
                glyph: '|',
                color: Color::Rgb(139, 69, 19),
                bold: false,
                z: 10,
            },
        ))
        .id();
    renderables.push(sign);

    for (index, position) in SQUIRREL_POSITIONS.into_iter().enumerate() {
        let squirrel = spawn_squirrel(world, index, position);
        renderables.push(squirrel);
    }

    let leader = party[0];
    world.insert_resource(ControlFocus { entity: leader });
    world.insert_resource(ViewFocus { center: ORIGIN });
    world.insert_resource(PendingWalkDestination::default());
    world.insert_resource(ActiveWalkDestination::default());
    world.insert_resource(RenderableEntities {
        entities: renderables,
    });
    world.insert_resource(GameMode::Explore);
    world.insert_resource(PartyRoster {
        party_id: PARTY_ID,
        members: party,
    });
    world.insert_resource(TurnOrder::default());
    world.insert_resource(TurnStartPause::default());
    world.insert_resource(TurnEndPause::default());
    world.insert_resource(EncounterConfig {
        trigger_radius_m: ENCOUNTER_TRIGGER_M,
    });
    world.insert_resource(CombatLog::default());
    world.insert_resource(DiceRng::default());
    world.insert_resource(LastLeaderDirection(DEFAULT_FACING));
}

fn spawn_party_member(
    world: &mut World,
    template: CharacterTemplate,
    order: usize,
    position: Vector,
    color: Color,
) -> Entity {
    let damage = template.damage_dice();
    world
        .spawn((
            Name(template.name),
            Species(template.species),
            Classes {
                primary: template.class,
                level: 1,
            },
            Abilities(template.abilities),
            CombatStats {
                armor_class: template.armor_class(),
                max_hp: template.max_hp,
                hp: template.max_hp,
                speed_m: template.speed_m,
                proficiency_bonus: proficiency_bonus(1),
            },
            Inventory {
                items: inventory_items(template),
            },
            Equipment {
                weapon: template.weapon,
                armor: template.armor,
                shield: template.shield,
            },
            Faction::Party,
            PartyMember {
                party_id: PARTY_ID,
                order,
            },
            ClickToWalk,
            Position(position),
            AttackProfile {
                name: weapon_name(template.weapon),
                attack_bonus: template.attack_bonus(),
                damage,
                reach_m: 1,
            },
            Renderable {
                glyph: '@',
                color,
                bold: true,
                z: 100,
            },
        ))
        .id()
}

fn spawn_squirrel(world: &mut World, index: usize, position: Vector) -> Entity {
    let template = squirrel_template();
    let names = ["Squirrel A", "Squirrel B", "Squirrel C"];

    world
        .spawn((
            Name(names[index.min(names.len() - 1)]),
            Abilities(template.abilities),
            CombatStats {
                armor_class: template.armor_class,
                max_hp: template.max_hp,
                hp: template.max_hp,
                speed_m: template.speed_m,
                proficiency_bonus: 0,
            },
            Faction::Hostile,
            EncounterGroup {
                id: SQUIRREL_ENCOUNTER_ID,
            },
            HostileAi,
            Position(position),
            AttackProfile {
                name: "Bite",
                attack_bonus: template.attack_bonus,
                damage: template.damage,
                reach_m: 1,
            },
            Renderable {
                glyph: 'r',
                color: Color::Gray,
                bold: false,
                z: 90,
            },
        ))
        .id()
}

fn inventory_items(template: CharacterTemplate) -> Vec<Item> {
    let mut items = vec![Item::Weapon(template.weapon), Item::Armor(template.armor)];
    if template.shield {
        items.push(Item::Shield);
    }
    items
}

fn weapon_name(kind: WeaponKind) -> &'static str {
    crate::rules::Weapon::for_kind(kind).name
}

pub fn movement_schedule() -> Schedule {
    gameplay_schedule()
}

pub fn gameplay_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            turn_end_pause_system,
            turn_start_pause_system,
            assign_pending_intent_system,
            enemy_ai_system,
            explore_party_follow_system,
            walk_system,
            combat_attack_resolution_system,
            enemy_turn_completion_system,
            encounter_detection_system,
            sync_view_focus_system,
        )
            .chain(),
    );
    schedule
}

pub fn turn_end_pause_system(world: &mut World) {
    if *world.resource::<GameMode>() != GameMode::TurnBased {
        world.resource_mut::<TurnEndPause>().clear();
        return;
    }

    let should_advance = {
        let mut pause = world.resource_mut::<TurnEndPause>();
        if !pause.pending_advance {
            return;
        }
        if pause.remaining_ticks > 0 {
            pause.remaining_ticks -= 1;
            false
        } else {
            pause.clear();
            true
        }
    };

    if should_advance {
        advance_turn(world);
    }
}

pub fn turn_start_pause_system(world: &mut World) {
    if *world.resource::<GameMode>() != GameMode::TurnBased {
        world.resource_mut::<TurnStartPause>().clear();
        return;
    }

    let mut pause = world.resource_mut::<TurnStartPause>();
    pause.paused_this_tick = pause.remaining_ticks > 0;
    if pause.remaining_ticks > 0 {
        pause.remaining_ticks -= 1;
    }
}

pub fn assign_pending_intent_system(world: &mut World) {
    if *world.resource::<GameMode>() == GameMode::TurnBased && turn_pause_active(world) {
        return;
    }
    let destination = world.resource_mut::<PendingWalkDestination>().0.take();
    let Some(destination) = destination else {
        return;
    };

    match *world.resource::<GameMode>() {
        GameMode::Explore => assign_explore_destination(world, destination),
        GameMode::TurnBased => assign_combat_destination(world, destination),
    }
}

fn assign_explore_destination(world: &mut World, destination: Vector) {
    let focused = world.resource::<ControlFocus>().entity;
    if world.get::<ClickToWalk>(focused).is_none() {
        return;
    }

    if let Ok(mut entity) = world.get_entity_mut(focused) {
        entity.remove::<AttackMoveTarget>();
        entity.insert(WalkTarget { destination });
    }
    world.resource_mut::<ActiveWalkDestination>().0 = Some(destination);
}

fn assign_combat_destination(world: &mut World, destination: Vector) {
    if turn_pause_active(world) {
        return;
    }
    let Some(active) = active_turn_entity(world) else {
        world.resource_mut::<ActiveWalkDestination>().0 = None;
        return;
    };
    if world.get::<Faction>(active) != Some(&Faction::Party) {
        world.resource_mut::<ActiveWalkDestination>().0 = None;
        return;
    }

    let Some(action) = world.get::<ActionState>(active).copied() else {
        world.resource_mut::<ActiveWalkDestination>().0 = None;
        return;
    };

    if let Some(target) = hostile_entity_at(world, active, destination) {
        if action.has_attacked {
            log_line(
                world,
                format!("{} has already attacked.", name_of(world, active)),
            );
            world.resource_mut::<ActiveWalkDestination>().0 = None;
            return;
        }

        let target_position = world.get::<Position>(target).map(|position| position.0);
        if let Some(target_position) = target_position {
            let in_reach = is_in_reach(world, active, target);
            let mut entity = world.entity_mut(active);
            entity.insert(AttackMoveTarget { target });
            if !in_reach {
                entity.insert(WalkTarget {
                    destination: target_position,
                });
            }
            world.resource_mut::<ActiveWalkDestination>().0 = Some(target_position);
        }
        return;
    }

    if action.remaining_movement <= 0 {
        log_line(
            world,
            format!("{} has no movement remaining.", name_of(world, active)),
        );
        world.resource_mut::<ActiveWalkDestination>().0 = None;
        return;
    }

    world
        .entity_mut(active)
        .insert(WalkTarget { destination })
        .remove::<AttackMoveTarget>();
    world.resource_mut::<ActiveWalkDestination>().0 = Some(destination);
}

pub fn walk_system(world: &mut World) {
    let mode = *world.resource::<GameMode>();
    if mode == GameMode::TurnBased && turn_pause_active(world) {
        return;
    }
    let active = active_turn_entity(world);
    let leader = party_leader(world);
    let walkers = {
        let mut query = world.query::<(Entity, &Position, &WalkTarget)>();
        query
            .iter(world)
            .map(|(entity, position, target)| (entity, position.0, target.destination))
            .collect::<Vec<_>>()
    };
    let mut arrived = Vec::new();

    for (entity, current, destination) in walkers {
        if mode == GameMode::TurnBased && Some(entity) != active {
            continue;
        }
        if current == destination {
            arrived.push((entity, destination));
            continue;
        }
        if mode == GameMode::TurnBased {
            if world
                .get::<AttackMoveTarget>(entity)
                .and_then(|target| target_position(world, target.target))
                .is_some_and(|target_position| distance_m(current, target_position) <= 1)
            {
                arrived.push((entity, destination));
                continue;
            }
            let remaining = world
                .get::<ActionState>(entity)
                .map(|action| action.remaining_movement)
                .unwrap_or(0);
            if remaining <= 0 {
                continue;
            }
        }

        let next = step_toward(current, destination);
        if let Some(mut position) = world.get_mut::<Position>(entity) {
            position.0 = next;
        }

        if mode == GameMode::Explore && Some(entity) == leader {
            world.resource_mut::<LastLeaderDirection>().0 = Vector {
                x: (next.x - current.x).signum(),
                y: (next.y - current.y).signum(),
            };
        }

        if mode == GameMode::TurnBased {
            let remaining = if let Some(mut action) = world.get_mut::<ActionState>(entity) {
                action.remaining_movement = action.remaining_movement.saturating_sub(1);
                action.remaining_movement
            } else {
                0
            };
            log_line(
                world,
                format!(
                    "{} moves to ({}, {}), {}m remaining.",
                    name_of(world, entity),
                    next.x,
                    next.y,
                    remaining
                ),
            );
        }

        if next == destination {
            arrived.push((entity, destination));
        }
    }

    let focused = world.resource::<ControlFocus>().entity;
    for (entity, destination) in arrived {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.remove::<WalkTarget>();
        }
        let active_destination = &mut world.resource_mut::<ActiveWalkDestination>().0;
        if (entity == focused || Some(entity) == active) && *active_destination == Some(destination)
        {
            *active_destination = None;
        }
    }
}

pub fn explore_party_follow_system(world: &mut World) {
    if *world.resource::<GameMode>() != GameMode::Explore {
        return;
    }
    let Some(leader) = party_leader(world) else {
        return;
    };
    let Some(leader_position) = target_position(world, leader) else {
        return;
    };
    let facing = world.resource::<LastLeaderDirection>().0;
    let members = world.resource::<PartyRoster>().members.clone();

    for member in members {
        if member == leader || !is_living(world, member) {
            continue;
        }
        let Some(order) = world.get::<PartyMember>(member).map(|member| member.order) else {
            continue;
        };
        let Some(current) = target_position(world, member) else {
            continue;
        };
        let destination = formation_target(leader_position, facing, order);
        if current == destination {
            if let Ok(mut entity) = world.get_entity_mut(member) {
                entity.remove::<WalkTarget>();
            }
        } else {
            world.entity_mut(member).insert(WalkTarget { destination });
        }
    }
}

pub fn encounter_detection_system(world: &mut World) {
    if *world.resource::<GameMode>() != GameMode::Explore {
        return;
    }
    let radius = world.resource::<EncounterConfig>().trigger_radius_m;
    let party_positions = living_party_positions(world);
    let hostiles = living_hostiles(world);

    for (hostile, hostile_position, group_id) in hostiles {
        if party_positions
            .iter()
            .any(|(_, party_position)| distance_m(*party_position, hostile_position) <= radius)
        {
            start_encounter(world, group_id);
            log_line(
                world,
                format!("{} notices the party.", name_of(world, hostile)),
            );
            break;
        }
    }
}

pub fn start_encounter(world: &mut World, group_id: u32) {
    *world.resource_mut::<GameMode>() = GameMode::TurnBased;
    world.resource_mut::<PendingWalkDestination>().0 = None;
    world.resource_mut::<ActiveWalkDestination>().0 = None;
    clear_motion_components(world);

    let mut combatants = world.resource::<PartyRoster>().members.clone();
    combatants.extend(
        world
            .resource::<RenderableEntities>()
            .entities
            .iter()
            .copied()
            .filter(|entity| {
                is_living(world, *entity)
                    && world
                        .get::<EncounterGroup>(*entity)
                        .is_some_and(|group| group.id == group_id)
            }),
    );
    combatants.retain(|entity| is_living(world, *entity));
    combatants.sort_by_key(|entity| world.get::<PartyMember>(*entity).map(|member| member.order));
    combatants.dedup();

    let entries = roll_initiative(world, combatants);
    *world.resource_mut::<TurnOrder>() = TurnOrder {
        entries,
        current_index: 0,
        round: 1,
        encounter_group: Some(group_id),
    };
    log_line(world, "Battle begins.");
    begin_active_turn(world);
}

pub fn enemy_ai_system(world: &mut World) {
    if *world.resource::<GameMode>() != GameMode::TurnBased {
        return;
    }
    if turn_pause_active(world) {
        return;
    }
    let Some(active) = active_turn_entity(world) else {
        return;
    };
    if world.get::<Faction>(active) != Some(&Faction::Hostile) {
        return;
    }
    if world.get::<AttackMoveTarget>(active).is_some() {
        return;
    }
    let Some(action) = world.get::<ActionState>(active).copied() else {
        return;
    };
    if action.has_attacked {
        schedule_enemy_turn_end_pause(world);
        return;
    }
    let Some(target) = nearest_living_party_member(world, active) else {
        finish_combat(world, "The party falls.");
        return;
    };

    let Some(target_position) = target_position(world, target) else {
        schedule_enemy_turn_end_pause(world);
        return;
    };
    world.entity_mut(active).insert(AttackMoveTarget { target });

    if !is_in_reach(world, active, target) {
        if action.remaining_movement > 0 {
            world.entity_mut(active).insert(WalkTarget {
                destination: target_position,
            });
            world.resource_mut::<ActiveWalkDestination>().0 = Some(target_position);
        } else {
            log_line(
                world,
                format!(
                    "{} cannot reach {}.",
                    name_of(world, active),
                    name_of(world, target)
                ),
            );
            schedule_enemy_turn_end_pause(world);
        }
    }
}

pub fn combat_attack_resolution_system(world: &mut World) {
    if *world.resource::<GameMode>() != GameMode::TurnBased {
        return;
    }
    if turn_pause_active(world) {
        return;
    }
    let Some(active) = active_turn_entity(world) else {
        return;
    };
    let Some(target) = world
        .get::<AttackMoveTarget>(active)
        .map(|target| target.target)
    else {
        return;
    };
    if !is_living(world, active) || !is_living(world, target) {
        clear_active_attack(world, active);
        return;
    }
    if !is_in_reach(world, active, target) {
        return;
    }
    if world
        .get::<ActionState>(active)
        .is_none_or(|action| action.has_attacked)
    {
        clear_active_attack(world, active);
        return;
    }

    resolve_attack(world, active, target);
    clear_active_attack(world, active);
    world.resource_mut::<ActiveWalkDestination>().0 = None;

    if finish_combat_if_resolved(world) {
        return;
    }
    if world.get::<Faction>(active) == Some(&Faction::Hostile) {
        schedule_enemy_turn_end_pause(world);
    }
}

pub fn enemy_turn_completion_system(world: &mut World) {
    if *world.resource::<GameMode>() != GameMode::TurnBased {
        return;
    }
    if turn_pause_active(world) {
        return;
    }
    let Some(active) = active_turn_entity(world) else {
        return;
    };
    if world.get::<Faction>(active) != Some(&Faction::Hostile) {
        return;
    }
    let remaining = world
        .get::<ActionState>(active)
        .map(|action| action.remaining_movement)
        .unwrap_or_default();
    if remaining > 0 {
        return;
    }
    if let Some(target) = world
        .get::<AttackMoveTarget>(active)
        .map(|target| target.target)
        && is_in_reach(world, active, target)
    {
        return;
    }

    log_line(world, format!("{} ends its turn.", name_of(world, active)));
    schedule_enemy_turn_end_pause(world);
}

pub fn end_current_turn(world: &mut World) {
    if *world.resource::<GameMode>() == GameMode::TurnBased && !turn_end_pause_active(world) {
        advance_turn(world);
    }
}

fn resolve_attack(world: &mut World, attacker: Entity, target: Entity) {
    let attacker_name = name_of(world, attacker).to_string();
    let target_name = name_of(world, target).to_string();
    let Some(attack) = world.get::<AttackProfile>(attacker).copied() else {
        return;
    };
    let target_ac = world
        .get::<CombatStats>(target)
        .map(|stats| stats.armor_class)
        .unwrap_or(10);
    let attack_roll = roll_die(world, 20);
    let total = attack_roll + attack.attack_bonus;
    let hit = attack_roll == 20 || (attack_roll != 1 && total >= target_ac);

    if hit {
        let damage = roll_damage(world, attack.damage).max(1);
        let hp_after = if let Some(mut stats) = world.get_mut::<CombatStats>(target) {
            stats.hp = stats.hp.saturating_sub(damage);
            stats.hp
        } else {
            0
        };
        log_line(
            world,
            format!(
                "{attacker_name} hits {target_name} with {} for {damage} damage ({hp_after} HP).",
                attack.name
            ),
        );
        if hp_after <= 0 {
            log_line(world, format!("{target_name} falls."));
            despawn_combatant(world, target);
        }
    } else {
        log_line(
            world,
            format!(
                "{attacker_name} misses {target_name} with {} ({total} vs AC {target_ac}).",
                attack.name
            ),
        );
    }

    if let Some(mut action) = world.get_mut::<ActionState>(attacker) {
        action.has_attacked = true;
    }
}

fn advance_turn(world: &mut World) {
    if *world.resource::<GameMode>() != GameMode::TurnBased {
        return;
    }
    if let Some(active) = active_turn_entity(world)
        && let Ok(mut entity) = world.get_entity_mut(active)
    {
        entity.remove::<ActionState>();
        entity.remove::<WalkTarget>();
        entity.remove::<AttackMoveTarget>();
    }
    world.resource_mut::<ActiveWalkDestination>().0 = None;
    world.resource_mut::<TurnEndPause>().clear();

    if finish_combat_if_resolved(world) {
        return;
    }

    prune_turn_order(world);
    let len = world.resource::<TurnOrder>().entries.len();
    if len == 0 {
        finish_combat(world, "Battle ends.");
        return;
    }

    let new_round = {
        let mut order = world.resource_mut::<TurnOrder>();
        if order.current_index + 1 >= len {
            order.round = order.round.saturating_add(1);
            true
        } else {
            order.current_index += 1;
            false
        }
    };

    if new_round {
        reroll_initiative(world);
        let round = world.resource::<TurnOrder>().round;
        log_line(world, format!("Round {round} begins."));
    }

    begin_active_turn(world);
}

fn begin_active_turn(world: &mut World) {
    let Some(active) = active_turn_entity(world) else {
        return;
    };
    if !is_living(world, active) {
        advance_turn(world);
        return;
    }
    let speed = world
        .get::<CombatStats>(active)
        .map(|stats| stats.speed_m)
        .unwrap_or_default();
    world.entity_mut(active).insert(ActionState {
        remaining_movement: speed,
        has_attacked: false,
    });
    *world.resource_mut::<TurnStartPause>() = TurnStartPause {
        remaining_ticks: TURN_START_PAUSE_TICKS,
        paused_this_tick: true,
    };
    world.resource_mut::<ControlFocus>().entity = active;
    sync_view_focus_system(world);
    log_line(world, format!("{}'s turn.", name_of(world, active)));
}

fn roll_initiative(world: &mut World, combatants: Vec<Entity>) -> Vec<TurnEntry> {
    let living = combatants
        .into_iter()
        .filter(|entity| is_living(world, *entity))
        .collect::<Vec<_>>();
    let mut entries = Vec::with_capacity(living.len());
    for entity in living {
        let dexterity = world
            .get::<Abilities>(entity)
            .map(|abilities| abilities.0.modifier(Ability::Dexterity))
            .unwrap_or_default();
        entries.push(TurnEntry {
            entity,
            initiative: roll_die(world, 20) + dexterity,
        });
    }
    entries.sort_by(|a, b| b.initiative.cmp(&a.initiative));
    entries
}

fn reroll_initiative(world: &mut World) {
    let group = world.resource::<TurnOrder>().encounter_group;
    let combatants = world
        .resource::<TurnOrder>()
        .entries
        .iter()
        .map(|entry| entry.entity)
        .filter(|entity| is_living(world, *entity))
        .collect::<Vec<_>>();
    let entries = roll_initiative(world, combatants);
    let mut order = world.resource_mut::<TurnOrder>();
    order.entries = entries;
    order.current_index = 0;
    order.encounter_group = group;
}

fn prune_turn_order(world: &mut World) {
    let living = world
        .resource::<TurnOrder>()
        .entries
        .iter()
        .copied()
        .filter(|entry| is_living(world, entry.entity))
        .collect::<Vec<_>>();
    let mut order = world.resource_mut::<TurnOrder>();
    order.entries = living;
    if order.current_index >= order.entries.len() {
        order.current_index = order.entries.len().saturating_sub(1);
    }
}

fn finish_combat_if_resolved(world: &mut World) -> bool {
    if *world.resource::<GameMode>() != GameMode::TurnBased {
        return false;
    }
    let party_alive = world
        .resource::<PartyRoster>()
        .members
        .iter()
        .any(|entity| is_living(world, *entity));
    let hostiles_alive = world.resource::<TurnOrder>().entries.iter().any(|entry| {
        is_living(world, entry.entity)
            && world.get::<Faction>(entry.entity) == Some(&Faction::Hostile)
    });

    if !party_alive {
        finish_combat(world, "The party falls.");
        true
    } else if !hostiles_alive {
        finish_combat(world, "Battle ends.");
        true
    } else {
        false
    }
}

fn finish_combat(world: &mut World, message: &'static str) {
    *world.resource_mut::<GameMode>() = GameMode::Explore;
    world.resource_mut::<TurnOrder>().entries.clear();
    world.resource_mut::<TurnOrder>().current_index = 0;
    world.resource_mut::<TurnOrder>().encounter_group = None;
    world.resource_mut::<ActiveWalkDestination>().0 = None;
    world.resource_mut::<TurnStartPause>().clear();
    world.resource_mut::<TurnEndPause>().clear();
    clear_motion_components(world);
    if let Some(leader) = party_leader(world) {
        world.resource_mut::<ControlFocus>().entity = leader;
    }
    log_line(world, message);
}

fn despawn_combatant(world: &mut World, entity: Entity) {
    if let Some(mut renderables) = world.get_resource_mut::<RenderableEntities>() {
        renderables
            .entities
            .retain(|candidate| *candidate != entity);
    }
    if let Some(mut party) = world.get_resource_mut::<PartyRoster>() {
        party.members.retain(|candidate| *candidate != entity);
    }
    if let Some(mut order) = world.get_resource_mut::<TurnOrder>() {
        order.entries.retain(|entry| entry.entity != entity);
    }
    if world.get_entity(entity).is_ok() {
        world.entity_mut(entity).despawn();
    }
}

fn clear_active_attack(world: &mut World, entity: Entity) {
    if let Ok(mut entity) = world.get_entity_mut(entity) {
        entity.remove::<AttackMoveTarget>();
        entity.remove::<WalkTarget>();
    }
}

fn clear_motion_components(world: &mut World) {
    let entities = world
        .resource::<RenderableEntities>()
        .entities
        .iter()
        .copied()
        .collect::<Vec<_>>();
    for entity in entities {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.remove::<WalkTarget>();
            entity_mut.remove::<AttackMoveTarget>();
            entity_mut.remove::<ActionState>();
        }
    }
}

fn schedule_enemy_turn_end_pause(world: &mut World) {
    world.resource_mut::<ActiveWalkDestination>().0 = None;
    world.resource_mut::<TurnEndPause>().start();
}

pub fn sync_view_focus_system(world: &mut World) {
    if *world.resource::<GameMode>() == GameMode::TurnBased {
        if let Some(center) = combatants_average_position(world) {
            world.resource_mut::<ViewFocus>().center = center;
        }
        return;
    }

    let focused = world.resource::<ControlFocus>().entity;
    let Some(position) = world.get::<Position>(focused).map(|position| position.0) else {
        return;
    };

    world.resource_mut::<ViewFocus>().center = position;
}

fn combatants_average_position(world: &World) -> Option<Vector> {
    let positions = world
        .resource::<TurnOrder>()
        .entries
        .iter()
        .filter(|entry| is_living(world, entry.entity))
        .filter_map(|entry| target_position(world, entry.entity))
        .collect::<Vec<_>>();
    if positions.is_empty() {
        return None;
    }

    let count = positions.len() as f64;
    let sum_x = positions
        .iter()
        .map(|position| position.x as i64)
        .sum::<i64>();
    let sum_y = positions
        .iter()
        .map(|position| position.y as i64)
        .sum::<i64>();

    Some(Vector {
        x: (sum_x as f64 / count).round() as i32,
        y: (sum_y as f64 / count).round() as i32,
    })
}

fn active_turn_entity(world: &World) -> Option<Entity> {
    (*world.resource::<GameMode>() == GameMode::TurnBased)
        .then(|| world.resource::<TurnOrder>().active())
        .flatten()
}

fn turn_start_pause_active(world: &World) -> bool {
    world.resource::<TurnStartPause>().waiting()
}

fn turn_end_pause_active(world: &World) -> bool {
    world.resource::<TurnEndPause>().waiting()
}

fn turn_pause_active(world: &World) -> bool {
    turn_start_pause_active(world) || turn_end_pause_active(world)
}

fn party_leader(world: &World) -> Option<Entity> {
    world.resource::<PartyRoster>().members.first().copied()
}

fn living_party_positions(world: &World) -> Vec<(Entity, Vector)> {
    world
        .resource::<PartyRoster>()
        .members
        .iter()
        .copied()
        .filter(|entity| is_living(world, *entity))
        .filter_map(|entity| target_position(world, entity).map(|position| (entity, position)))
        .collect()
}

fn living_hostiles(world: &World) -> Vec<(Entity, Vector, u32)> {
    world
        .resource::<RenderableEntities>()
        .entities
        .iter()
        .copied()
        .filter(|entity| {
            is_living(world, *entity) && world.get::<Faction>(*entity) == Some(&Faction::Hostile)
        })
        .filter_map(|entity| {
            let position = target_position(world, entity)?;
            let group = world.get::<EncounterGroup>(entity)?.id;
            Some((entity, position, group))
        })
        .collect()
}

fn hostile_entity_at(world: &World, attacker: Entity, coord: Vector) -> Option<Entity> {
    let faction = world.get::<Faction>(attacker).copied()?;
    world
        .resource::<RenderableEntities>()
        .entities
        .iter()
        .copied()
        .filter(|entity| {
            is_living(world, *entity)
                && world
                    .get::<Faction>(*entity)
                    .is_some_and(|other| *other != faction)
                && world
                    .get::<Position>(*entity)
                    .is_some_and(|position| position.0 == coord)
        })
        .max_by_key(|entity| {
            world
                .get::<Renderable>(*entity)
                .map(|renderable| renderable.z)
        })
}

fn nearest_living_party_member(world: &World, active: Entity) -> Option<Entity> {
    let active_position = target_position(world, active)?;
    living_party_positions(world)
        .into_iter()
        .min_by_key(|(_, position)| distance_m(active_position, *position))
        .map(|(entity, _)| entity)
}

fn is_living(world: &World, entity: Entity) -> bool {
    world
        .get::<CombatStats>(entity)
        .is_some_and(|stats| stats.hp > 0)
}

fn target_position(world: &World, entity: Entity) -> Option<Vector> {
    world.get::<Position>(entity).map(|position| position.0)
}

fn is_in_reach(world: &World, attacker: Entity, target: Entity) -> bool {
    let reach = world
        .get::<AttackProfile>(attacker)
        .map(|attack| attack.reach_m)
        .unwrap_or(1);
    let Some(attacker_position) = target_position(world, attacker) else {
        return false;
    };
    let Some(target_position) = target_position(world, target) else {
        return false;
    };
    distance_m(attacker_position, target_position) <= reach
}

pub fn distance_m(a: Vector, b: Vector) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn formation_target(leader: Vector, facing: Vector, order: usize) -> Vector {
    let facing = if facing.x == 0 && facing.y == 0 {
        DEFAULT_FACING
    } else {
        facing
    };
    let behind = Vector {
        x: -facing.x,
        y: -facing.y,
    };
    let perpendicular = Vector {
        x: -facing.y,
        y: facing.x,
    };
    let base = Vector {
        x: leader.x + behind.x * 2,
        y: leader.y + behind.y * 2,
    };

    match order {
        1 => base,
        2 => Vector {
            x: base.x + perpendicular.x,
            y: base.y + perpendicular.y,
        },
        3 => Vector {
            x: base.x - perpendicular.x,
            y: base.y - perpendicular.y,
        },
        _ => Vector {
            x: leader.x + behind.x * (order as i32 + 1),
            y: leader.y + behind.y * (order as i32 + 1),
        },
    }
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

fn roll_die(world: &mut World, sides: u8) -> i32 {
    world.resource_mut::<DiceRng>().roll_die(sides)
}

fn roll_damage(world: &mut World, damage: DamageDice) -> i32 {
    let dice = (0..damage.count)
        .map(|_| roll_die(world, damage.sides))
        .sum::<i32>();
    dice + damage.bonus
}

fn name_of(world: &World, entity: Entity) -> &'static str {
    world
        .get::<Name>(entity)
        .map(|name| name.0)
        .unwrap_or("Someone")
}

fn log_line(world: &mut World, line: impl Into<String>) {
    world.resource_mut::<CombatLog>().push(line);
}

#[cfg(test)]
mod tests {
    use super::{
        ActionState, AttackMoveTarget, CombatLog, CombatStats, ControlFocus, EncounterConfig,
        Faction, GameMode, PartyRoster, Position, Renderable, RenderableEntities,
        SQUIRREL_ENCOUNTER_ID, SQUIRREL_POSITIONS, TURN_END_PAUSE_TICKS, TurnEndPause, TurnOrder,
        TurnStartPause, ViewFocus, WalkTarget, distance_m, end_current_turn, movement_schedule,
        spawn_initial_entities, start_encounter, sync_view_focus_system,
    };
    use crate::data::grid::{ORIGIN, Vector};
    use bevy_ecs::world::World;
    use ratatui::style::Color;

    fn world_with_entities() -> World {
        let mut world = World::new();
        spawn_initial_entities(&mut world);
        world
    }

    fn clear_turn_pauses(world: &mut World) {
        world.resource_mut::<TurnStartPause>().clear();
        world.resource_mut::<TurnEndPause>().clear();
    }

    #[test]
    fn startup_creates_party_sign_squirrels_and_resources() {
        let mut world = world_with_entities();

        assert_eq!(*world.resource::<GameMode>(), GameMode::Explore);
        assert_eq!(world.resource::<EncounterConfig>().trigger_radius_m, 10);
        assert_eq!(world.resource::<PartyRoster>().members.len(), 4);
        assert!(world.get_resource::<CombatLog>().is_some());

        let mut renderables = world.query::<(&Position, &Renderable)>();
        let cells = renderables.iter(&world).collect::<Vec<_>>();
        assert!(
            cells
                .iter()
                .any(|(position, renderable)| position.0 == ORIGIN
                    && renderable.glyph == '@'
                    && renderable.color == Color::Cyan)
        );
        assert!(
            cells
                .iter()
                .any(|(position, renderable)| position.0 == Vector { x: 4, y: 1 }
                    && renderable.glyph == '|')
        );
        assert!(
            cells
                .iter()
                .any(|(position, renderable)| position.0 == SQUIRREL_POSITIONS[0]
                    && renderable.glyph == 'r'
                    && renderable.color == Color::Gray)
        );
    }

    #[test]
    fn distance_uses_one_meter_grid_steps() {
        assert_eq!(distance_m(ORIGIN, Vector { x: 3, y: 2 }), 3);
    }

    #[test]
    fn explore_follow_assigns_diamond_slots_to_party() {
        let mut world = world_with_entities();
        let leader = world.resource::<ControlFocus>().entity;
        world.entity_mut(leader).insert(WalkTarget {
            destination: Vector { x: 0, y: -1 },
        });

        super::explore_party_follow_system(&mut world);

        let party = world.resource::<PartyRoster>().members.clone();
        let follower = party[1];
        assert!(world.get::<WalkTarget>(follower).is_some());
    }

    #[test]
    fn encounter_starts_when_hostile_is_within_trigger_radius() {
        let mut world = world_with_entities();
        let leader = world.resource::<ControlFocus>().entity;
        world.get_mut::<Position>(leader).unwrap().0 = Vector { x: 0, y: -15 };

        let mut schedule = movement_schedule();
        schedule.run(&mut world);

        assert_eq!(*world.resource::<GameMode>(), GameMode::TurnBased);
        assert_eq!(
            world.resource::<TurnOrder>().encounter_group,
            Some(SQUIRREL_ENCOUNTER_ID)
        );
        assert!(!world.resource::<TurnOrder>().entries.is_empty());
    }

    #[test]
    fn start_encounter_rolls_initiative_and_focuses_active_entity() {
        let mut world = world_with_entities();
        start_encounter(&mut world, SQUIRREL_ENCOUNTER_ID);

        let active = world.resource::<TurnOrder>().active().unwrap();
        assert_eq!(world.resource::<ControlFocus>().entity, active);
        assert!(world.get::<ActionState>(active).is_some());
    }

    #[test]
    fn combat_view_focus_uses_average_combatant_position() {
        let mut world = world_with_entities();
        start_encounter(&mut world, SQUIRREL_ENCOUNTER_ID);
        let party = world.resource::<PartyRoster>().members.clone();
        let squirrels = world
            .resource::<RenderableEntities>()
            .entities
            .iter()
            .copied()
            .filter(|entity| world.get::<Faction>(*entity) == Some(&Faction::Hostile))
            .collect::<Vec<_>>();

        world.get_mut::<Position>(party[0]).unwrap().0 = Vector { x: 0, y: 0 };
        world.get_mut::<Position>(party[1]).unwrap().0 = Vector { x: 0, y: 2 };
        world.get_mut::<Position>(party[2]).unwrap().0 = Vector { x: 2, y: 0 };
        world.get_mut::<Position>(party[3]).unwrap().0 = Vector { x: 2, y: 2 };
        world.get_mut::<Position>(squirrels[0]).unwrap().0 = Vector { x: 10, y: 0 };
        world.get_mut::<Position>(squirrels[1]).unwrap().0 = Vector { x: 10, y: 2 };
        world.get_mut::<Position>(squirrels[2]).unwrap().0 = Vector { x: 11, y: 1 };

        sync_view_focus_system(&mut world);

        assert_eq!(world.resource::<ViewFocus>().center, Vector { x: 5, y: 1 });
    }

    #[test]
    fn combat_click_on_hostile_creates_attack_move_target() {
        let mut world = world_with_entities();
        start_encounter(&mut world, SQUIRREL_ENCOUNTER_ID);
        clear_turn_pauses(&mut world);
        let party = world.resource::<PartyRoster>().members[0];
        let squirrel = world
            .resource::<RenderableEntities>()
            .entities
            .iter()
            .copied()
            .find(|entity| world.get::<Faction>(*entity) == Some(&Faction::Hostile))
            .unwrap();
        *world.resource_mut::<TurnOrder>() = TurnOrder {
            entries: vec![super::TurnEntry {
                entity: party,
                initiative: 20,
            }],
            current_index: 0,
            round: 1,
            encounter_group: Some(SQUIRREL_ENCOUNTER_ID),
        };
        world.resource_mut::<ControlFocus>().entity = party;
        world.entity_mut(party).insert(ActionState {
            remaining_movement: 9,
            has_attacked: false,
        });
        world.resource_mut::<super::PendingWalkDestination>().0 =
            Some(world.get::<Position>(squirrel).unwrap().0);

        super::assign_pending_intent_system(&mut world);

        assert_eq!(
            world.get::<AttackMoveTarget>(party).unwrap().target,
            squirrel
        );
    }

    #[test]
    fn attack_damage_can_remove_dead_hostiles() {
        let mut world = world_with_entities();
        start_encounter(&mut world, SQUIRREL_ENCOUNTER_ID);
        clear_turn_pauses(&mut world);
        let party = world.resource::<PartyRoster>().members[0];
        let squirrel = world
            .resource::<RenderableEntities>()
            .entities
            .iter()
            .copied()
            .find(|entity| world.get::<Faction>(*entity) == Some(&Faction::Hostile))
            .unwrap();
        world.get_mut::<Position>(party).unwrap().0 = Vector { x: -2, y: -23 };
        world.entity_mut(party).insert(ActionState {
            remaining_movement: 9,
            has_attacked: false,
        });
        world
            .entity_mut(party)
            .insert(AttackMoveTarget { target: squirrel });
        world.resource_mut::<super::DiceRng>().queue_roll(20);
        world.resource_mut::<super::DiceRng>().queue_roll(1);
        *world.resource_mut::<TurnOrder>() = TurnOrder {
            entries: vec![
                super::TurnEntry {
                    entity: party,
                    initiative: 20,
                },
                super::TurnEntry {
                    entity: squirrel,
                    initiative: 1,
                },
            ],
            current_index: 0,
            round: 1,
            encounter_group: Some(SQUIRREL_ENCOUNTER_ID),
        };

        super::combat_attack_resolution_system(&mut world);

        assert!(world.get_entity(squirrel).is_err());
    }

    #[test]
    fn space_style_end_turn_advances_focus() {
        let mut world = world_with_entities();
        start_encounter(&mut world, SQUIRREL_ENCOUNTER_ID);
        let first = world.resource::<ControlFocus>().entity;

        end_current_turn(&mut world);

        assert_ne!(world.resource::<ControlFocus>().entity, first);
    }

    #[test]
    fn enemy_ai_assigns_a_target() {
        let mut world = world_with_entities();
        start_encounter(&mut world, SQUIRREL_ENCOUNTER_ID);
        clear_turn_pauses(&mut world);
        let squirrel = world
            .resource::<RenderableEntities>()
            .entities
            .iter()
            .copied()
            .find(|entity| world.get::<Faction>(*entity) == Some(&Faction::Hostile))
            .unwrap();
        *world.resource_mut::<TurnOrder>() = TurnOrder {
            entries: vec![super::TurnEntry {
                entity: squirrel,
                initiative: 20,
            }],
            current_index: 0,
            round: 1,
            encounter_group: Some(SQUIRREL_ENCOUNTER_ID),
        };
        world.entity_mut(squirrel).insert(ActionState {
            remaining_movement: 6,
            has_attacked: false,
        });

        super::enemy_ai_system(&mut world);

        assert!(world.get::<AttackMoveTarget>(squirrel).is_some());
    }

    #[test]
    fn turn_start_pause_delays_combat_intents_until_elapsed() {
        let mut world = world_with_entities();
        start_encounter(&mut world, SQUIRREL_ENCOUNTER_ID);
        let party = world.resource::<PartyRoster>().members[0];
        let squirrel = world
            .resource::<RenderableEntities>()
            .entities
            .iter()
            .copied()
            .find(|entity| world.get::<Faction>(*entity) == Some(&Faction::Hostile))
            .unwrap();
        *world.resource_mut::<TurnOrder>() = TurnOrder {
            entries: vec![super::TurnEntry {
                entity: party,
                initiative: 20,
            }],
            current_index: 0,
            round: 1,
            encounter_group: Some(SQUIRREL_ENCOUNTER_ID),
        };
        world.entity_mut(party).insert(ActionState {
            remaining_movement: 9,
            has_attacked: false,
        });
        world.resource_mut::<super::PendingWalkDestination>().0 =
            Some(world.get::<Position>(squirrel).unwrap().0);

        super::assign_pending_intent_system(&mut world);

        assert!(world.get::<AttackMoveTarget>(party).is_none());
        for _ in 0..=super::TURN_START_PAUSE_TICKS {
            super::turn_start_pause_system(&mut world);
            super::assign_pending_intent_system(&mut world);
        }

        assert_eq!(
            world.get::<AttackMoveTarget>(party).unwrap().target,
            squirrel
        );
    }

    #[test]
    fn enemy_turn_end_pause_lingers_before_focus_advances() {
        let mut world = world_with_entities();
        let party = world.resource::<PartyRoster>().members[0];
        let squirrel = world
            .resource::<RenderableEntities>()
            .entities
            .iter()
            .copied()
            .find(|entity| world.get::<Faction>(*entity) == Some(&Faction::Hostile))
            .unwrap();
        *world.resource_mut::<GameMode>() = GameMode::TurnBased;
        *world.resource_mut::<TurnOrder>() = TurnOrder {
            entries: vec![
                super::TurnEntry {
                    entity: squirrel,
                    initiative: 20,
                },
                super::TurnEntry {
                    entity: party,
                    initiative: 10,
                },
            ],
            current_index: 0,
            round: 1,
            encounter_group: Some(SQUIRREL_ENCOUNTER_ID),
        };
        world.resource_mut::<ControlFocus>().entity = squirrel;
        world.get_mut::<Position>(squirrel).unwrap().0 = Vector { x: 0, y: -1 };
        world.get_mut::<Position>(party).unwrap().0 = ORIGIN;
        world.entity_mut(squirrel).insert(ActionState {
            remaining_movement: 6,
            has_attacked: false,
        });
        clear_turn_pauses(&mut world);

        super::enemy_ai_system(&mut world);
        super::combat_attack_resolution_system(&mut world);

        assert!(world.resource::<TurnEndPause>().waiting());
        assert_eq!(world.resource::<ControlFocus>().entity, squirrel);

        for _ in 0..TURN_END_PAUSE_TICKS {
            super::turn_end_pause_system(&mut world);
            assert_eq!(world.resource::<ControlFocus>().entity, squirrel);
        }

        super::turn_end_pause_system(&mut world);

        assert_eq!(world.resource::<ControlFocus>().entity, party);
    }

    #[test]
    fn combat_movement_spends_budget() {
        let mut world = world_with_entities();
        let party = world.resource::<PartyRoster>().members[0];
        *world.resource_mut::<GameMode>() = GameMode::TurnBased;
        *world.resource_mut::<TurnOrder>() = TurnOrder {
            entries: vec![super::TurnEntry {
                entity: party,
                initiative: 20,
            }],
            current_index: 0,
            round: 1,
            encounter_group: Some(SQUIRREL_ENCOUNTER_ID),
        };
        world.entity_mut(party).insert(ActionState {
            remaining_movement: 2,
            has_attacked: false,
        });
        world.entity_mut(party).insert(WalkTarget {
            destination: Vector { x: 2, y: 0 },
        });

        super::walk_system(&mut world);

        assert_eq!(
            world.get::<CombatStats>(party).unwrap().speed_m,
            9,
            "speed itself remains unchanged"
        );
        assert_eq!(
            world.get::<ActionState>(party).unwrap().remaining_movement,
            1
        );
    }

    #[test]
    fn combat_log_can_write_to_file_without_terminal_output() {
        let path = std::env::temp_dir().join(format!(
            "frust-combat-log-{}-{}.log",
            std::process::id(),
            "combat_log_can_write_to_file"
        ));
        let _ = std::fs::remove_file(&path);
        let mut log = CombatLog::with_file(&path).unwrap();

        log.push("Battle begins.");
        log.push("Mara hits Squirrel A.");

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "Battle begins.\nMara hits Squirrel A.\n");
        assert_eq!(log.file_path(), Some(path.as_path()));
        assert_eq!(log.file_error(), None);

        let _ = std::fs::remove_file(&path);
    }
}
