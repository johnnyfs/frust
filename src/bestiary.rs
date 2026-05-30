//! Creature registry: the hand-authored bestiary of ~40 low-level creatures.
//!
//! Stat blocks are derived from the D&D System Reference Document 5.2.1,
//! © Wizards of the Coast LLC, licensed under CC-BY-4.0.
//!
//! Each entry pairs the rules-level [`CreatureTemplate`] (abilities, HP, AC, attack)
//! with its on-screen representation. Glyphs follow NetHack monster-class conventions
//! (e.g. `r` rodent, `d` canine, `Z` skeleton/zombie, `:` reptile, `;` sea creature),
//! and colors use ratatui's named palette (NetHack "brown" -> [`Color::Yellow`], bright
//! yellow -> [`Color::LightYellow`], black -> [`Color::DarkGray`]).

use ratatui::style::Color;

use crate::rules::{
    AbilityScores, CreatureTemplate, DamageDice, DamageType, feet_to_meters, squirrel_template,
};

/// Where a creature is typically encountered. A creature may belong to several.
/// `Caverns` covers the catacombs and caves beneath a city.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Habitat {
    Wilderness,
    Caverns,
    Urban,
    Water,
    Cliffs,
}

/// All five habitats, for iteration in tests and (eventually) spawn-table builders.
pub const ALL_HABITATS: [Habitat; 5] = [
    Habitat::Wilderness,
    Habitat::Caverns,
    Habitat::Urban,
    Habitat::Water,
    Habitat::Cliffs,
];

/// A bestiary entry: rules stats plus how the creature is drawn and where it lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureSpec {
    pub template: CreatureTemplate,
    /// NetHack-convention monster-class glyph.
    pub glyph: char,
    /// ratatui named color.
    pub color: Color,
    pub bold: bool,
    pub habitats: &'static [Habitat],
}

const fn ab(
    strength: i32,
    dexterity: i32,
    constitution: i32,
    intelligence: i32,
    wisdom: i32,
    charisma: i32,
) -> AbilityScores {
    AbilityScores {
        strength,
        dexterity,
        constitution,
        intelligence,
        wisdom,
        charisma,
    }
}

const fn dice(count: u8, sides: u8, bonus: i32, damage_type: DamageType) -> DamageDice {
    DamageDice {
        count,
        sides,
        bonus,
        damage_type,
    }
}

/// The complete bestiary. Built at call time because [`feet_to_meters`] is not `const`.
pub fn bestiary() -> Vec<CreatureSpec> {
    use DamageType::{Bludgeoning, Piercing, Slashing};
    use Habitat::{Caverns, Cliffs, Urban, Water, Wilderness};

    vec![
        // ---- Wilderness ------------------------------------------------------
        CreatureSpec {
            template: squirrel_template(),
            glyph: 'r',
            color: Color::Gray,
            bold: false,
            habitats: &[Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Wolf",
                abilities: ab(12, 15, 12, 3, 12, 6),
                max_hp: 11,
                speed_m: feet_to_meters(40),
                armor_class: 13,
                attack_bonus: 4,
                damage: dice(2, 4, 2, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 'd',
            color: Color::Gray,
            bold: false,
            habitats: &[Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Dire Wolf",
                abilities: ab(17, 15, 15, 3, 12, 7),
                max_hp: 37,
                speed_m: feet_to_meters(50),
                armor_class: 14,
                attack_bonus: 5,
                damage: dice(2, 6, 3, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 'd',
            color: Color::DarkGray,
            bold: false,
            habitats: &[Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Boar",
                abilities: ab(13, 11, 12, 2, 9, 5),
                max_hp: 11,
                speed_m: feet_to_meters(40),
                armor_class: 11,
                attack_bonus: 3,
                damage: dice(1, 6, 1, Slashing),
                attack_name: "Tusk",
                reach_m: 1,
            },
            glyph: 'q',
            color: Color::Yellow,
            bold: false,
            habitats: &[Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Brown Bear",
                abilities: ab(19, 10, 16, 2, 13, 7),
                max_hp: 34,
                speed_m: feet_to_meters(40),
                armor_class: 11,
                attack_bonus: 6,
                damage: dice(2, 6, 4, Slashing),
                attack_name: "Claws",
                reach_m: 1,
            },
            glyph: 'q',
            color: Color::Yellow,
            bold: true,
            habitats: &[Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Goblin",
                abilities: ab(8, 14, 10, 10, 8, 8),
                max_hp: 7,
                speed_m: feet_to_meters(30),
                armor_class: 15,
                attack_bonus: 4,
                damage: dice(1, 6, 2, Slashing),
                attack_name: "Scimitar",
                reach_m: 1,
            },
            glyph: 'o',
            color: Color::Green,
            bold: false,
            habitats: &[Wilderness, Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Orc",
                abilities: ab(16, 12, 16, 7, 11, 10),
                max_hp: 15,
                speed_m: feet_to_meters(30),
                armor_class: 13,
                attack_bonus: 5,
                damage: dice(1, 12, 3, Slashing),
                attack_name: "Greataxe",
                reach_m: 1,
            },
            glyph: 'o',
            color: Color::LightGreen,
            bold: false,
            habitats: &[Wilderness, Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Hobgoblin",
                abilities: ab(13, 12, 12, 10, 10, 9),
                max_hp: 11,
                speed_m: feet_to_meters(30),
                armor_class: 18,
                attack_bonus: 3,
                damage: dice(1, 8, 1, Slashing),
                attack_name: "Longsword",
                reach_m: 1,
            },
            glyph: 'o',
            color: Color::LightRed,
            bold: false,
            habitats: &[Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Giant Wolf Spider",
                abilities: ab(12, 16, 13, 3, 12, 4),
                max_hp: 11,
                speed_m: feet_to_meters(40),
                armor_class: 13,
                attack_bonus: 3,
                damage: dice(1, 6, 1, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 's',
            color: Color::Yellow,
            bold: false,
            habitats: &[Wilderness, Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Pseudodragon",
                abilities: ab(6, 15, 13, 10, 12, 10),
                max_hp: 7,
                speed_m: feet_to_meters(15),
                armor_class: 13,
                attack_bonus: 4,
                damage: dice(1, 4, 2, Piercing),
                attack_name: "Sting",
                reach_m: 1,
            },
            glyph: 'D',
            color: Color::Red,
            bold: false,
            habitats: &[Wilderness, Cliffs],
        },
        // ---- Caverns / catacombs --------------------------------------------
        CreatureSpec {
            template: CreatureTemplate {
                name: "Giant Rat",
                abilities: ab(7, 15, 11, 2, 10, 4),
                max_hp: 7,
                speed_m: feet_to_meters(30),
                armor_class: 12,
                attack_bonus: 4,
                damage: dice(1, 4, 2, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 'r',
            color: Color::Yellow,
            bold: false,
            habitats: &[Caverns, Urban],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Kobold",
                abilities: ab(7, 15, 9, 8, 7, 8),
                max_hp: 5,
                speed_m: feet_to_meters(30),
                armor_class: 12,
                attack_bonus: 4,
                damage: dice(1, 4, 2, Piercing),
                attack_name: "Dagger",
                reach_m: 1,
            },
            glyph: 'k',
            color: Color::Red,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Skeleton",
                abilities: ab(10, 14, 15, 6, 8, 5),
                max_hp: 13,
                speed_m: feet_to_meters(30),
                armor_class: 13,
                attack_bonus: 4,
                damage: dice(1, 6, 2, Piercing),
                attack_name: "Shortsword",
                reach_m: 1,
            },
            glyph: 'Z',
            color: Color::White,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Zombie",
                abilities: ab(13, 6, 16, 3, 6, 5),
                max_hp: 22,
                speed_m: feet_to_meters(20),
                armor_class: 8,
                attack_bonus: 3,
                damage: dice(1, 6, 1, Bludgeoning),
                attack_name: "Slam",
                reach_m: 1,
            },
            glyph: 'Z',
            color: Color::Green,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Ghoul",
                abilities: ab(13, 15, 10, 7, 10, 6),
                max_hp: 22,
                speed_m: feet_to_meters(30),
                armor_class: 12,
                attack_bonus: 4,
                damage: dice(2, 4, 2, Slashing),
                attack_name: "Claws",
                reach_m: 1,
            },
            glyph: 'Z',
            color: Color::Gray,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Giant Spider",
                abilities: ab(14, 16, 12, 2, 11, 4),
                max_hp: 26,
                speed_m: feet_to_meters(30),
                armor_class: 14,
                attack_bonus: 5,
                damage: dice(1, 8, 3, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 's',
            color: Color::Magenta,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Giant Centipede",
                abilities: ab(5, 14, 12, 1, 7, 3),
                max_hp: 4,
                speed_m: feet_to_meters(30),
                armor_class: 13,
                attack_bonus: 4,
                damage: dice(1, 4, 2, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 's',
            color: Color::LightRed,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Cockatrice",
                abilities: ab(6, 12, 12, 2, 13, 5),
                max_hp: 27,
                speed_m: feet_to_meters(20),
                armor_class: 11,
                attack_bonus: 3,
                damage: dice(1, 4, 1, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 'c',
            color: Color::Yellow,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Violet Fungus",
                abilities: ab(3, 1, 10, 1, 3, 1),
                max_hp: 18,
                speed_m: feet_to_meters(5),
                armor_class: 5,
                attack_bonus: 2,
                damage: dice(1, 8, 0, Bludgeoning),
                attack_name: "Rotting Touch",
                reach_m: 3,
            },
            glyph: 'F',
            color: Color::Magenta,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Gray Ooze",
                abilities: ab(12, 6, 16, 1, 6, 2),
                max_hp: 22,
                speed_m: feet_to_meters(10),
                armor_class: 8,
                attack_bonus: 3,
                damage: dice(1, 6, 1, Bludgeoning),
                attack_name: "Pseudopod",
                reach_m: 1,
            },
            glyph: 'P',
            color: Color::Gray,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Gelatinous Cube",
                abilities: ab(14, 3, 20, 1, 6, 1),
                max_hp: 84,
                speed_m: feet_to_meters(15),
                armor_class: 6,
                attack_bonus: 4,
                damage: dice(3, 6, 0, Bludgeoning),
                attack_name: "Pseudopod",
                reach_m: 1,
            },
            glyph: 'b',
            color: Color::Cyan,
            bold: false,
            habitats: &[Caverns],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Rust Monster",
                abilities: ab(13, 12, 13, 2, 13, 6),
                max_hp: 27,
                speed_m: feet_to_meters(40),
                armor_class: 14,
                attack_bonus: 3,
                damage: dice(1, 8, 1, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 'R',
            color: Color::LightRed,
            bold: false,
            habitats: &[Caverns],
        },
        // ---- Urban -----------------------------------------------------------
        CreatureSpec {
            template: CreatureTemplate {
                name: "Cat",
                abilities: ab(3, 15, 10, 3, 12, 7),
                max_hp: 2,
                speed_m: feet_to_meters(40),
                armor_class: 12,
                attack_bonus: 0,
                damage: dice(1, 1, 0, Slashing),
                attack_name: "Claws",
                reach_m: 1,
            },
            glyph: 'f',
            color: Color::White,
            bold: false,
            habitats: &[Urban],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Bandit",
                abilities: ab(11, 12, 12, 10, 10, 10),
                max_hp: 11,
                speed_m: feet_to_meters(30),
                armor_class: 12,
                attack_bonus: 3,
                damage: dice(1, 6, 1, Slashing),
                attack_name: "Scimitar",
                reach_m: 1,
            },
            glyph: '@',
            color: Color::Red,
            bold: false,
            habitats: &[Urban, Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Guard",
                abilities: ab(13, 12, 12, 10, 11, 10),
                max_hp: 11,
                speed_m: feet_to_meters(30),
                armor_class: 16,
                attack_bonus: 3,
                damage: dice(1, 6, 1, Piercing),
                attack_name: "Spear",
                reach_m: 1,
            },
            glyph: '@',
            color: Color::LightBlue,
            bold: false,
            habitats: &[Urban],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Cultist",
                abilities: ab(11, 12, 10, 10, 11, 10),
                max_hp: 9,
                speed_m: feet_to_meters(30),
                armor_class: 12,
                attack_bonus: 3,
                damage: dice(1, 6, 1, Slashing),
                attack_name: "Scimitar",
                reach_m: 1,
            },
            glyph: '@',
            color: Color::DarkGray,
            bold: false,
            habitats: &[Urban],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Thug",
                abilities: ab(15, 11, 14, 10, 10, 11),
                max_hp: 32,
                speed_m: feet_to_meters(30),
                armor_class: 11,
                attack_bonus: 4,
                damage: dice(1, 6, 2, Bludgeoning),
                attack_name: "Mace",
                reach_m: 1,
            },
            glyph: '@',
            color: Color::LightRed,
            bold: false,
            habitats: &[Urban],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Wererat",
                abilities: ab(10, 15, 12, 11, 10, 8),
                max_hp: 33,
                speed_m: feet_to_meters(30),
                armor_class: 12,
                attack_bonus: 4,
                damage: dice(1, 4, 2, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 'r',
            color: Color::LightRed,
            bold: true,
            habitats: &[Urban, Caverns],
        },
        // ---- Water -----------------------------------------------------------
        CreatureSpec {
            template: CreatureTemplate {
                name: "Giant Frog",
                abilities: ab(12, 13, 11, 2, 10, 3),
                max_hp: 18,
                speed_m: feet_to_meters(30),
                armor_class: 11,
                attack_bonus: 3,
                damage: dice(1, 6, 1, Bludgeoning),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: ':',
            color: Color::Green,
            bold: false,
            habitats: &[Water],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Crocodile",
                abilities: ab(15, 10, 13, 2, 10, 5),
                max_hp: 19,
                speed_m: feet_to_meters(20),
                armor_class: 12,
                attack_bonus: 4,
                damage: dice(1, 10, 2, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: ':',
            color: Color::Green,
            bold: true,
            habitats: &[Water],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Constrictor Snake",
                abilities: ab(15, 14, 12, 1, 10, 3),
                max_hp: 13,
                speed_m: feet_to_meters(30),
                armor_class: 12,
                attack_bonus: 4,
                damage: dice(1, 6, 2, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 'S',
            color: Color::Green,
            bold: false,
            habitats: &[Water, Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Poisonous Snake",
                abilities: ab(2, 16, 11, 1, 10, 3),
                max_hp: 2,
                speed_m: feet_to_meters(30),
                armor_class: 13,
                attack_bonus: 5,
                damage: dice(1, 1, 0, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: 'S',
            color: Color::LightGreen,
            bold: false,
            habitats: &[Water, Wilderness],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Reef Shark",
                abilities: ab(14, 13, 13, 1, 10, 4),
                max_hp: 22,
                speed_m: feet_to_meters(0),
                armor_class: 12,
                attack_bonus: 4,
                damage: dice(1, 8, 2, Piercing),
                attack_name: "Bite",
                reach_m: 1,
            },
            glyph: ';',
            color: Color::Blue,
            bold: false,
            habitats: &[Water],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Sahuagin",
                abilities: ab(13, 11, 12, 12, 13, 9),
                max_hp: 22,
                speed_m: feet_to_meters(30),
                armor_class: 12,
                attack_bonus: 3,
                damage: dice(1, 4, 1, Slashing),
                attack_name: "Claws",
                reach_m: 1,
            },
            glyph: ';',
            color: Color::Cyan,
            bold: false,
            habitats: &[Water],
        },
        // ---- Cliffs / heights ------------------------------------------------
        CreatureSpec {
            template: CreatureTemplate {
                name: "Giant Goat",
                abilities: ab(17, 11, 12, 3, 12, 6),
                max_hp: 19,
                speed_m: feet_to_meters(40),
                armor_class: 11,
                attack_bonus: 5,
                damage: dice(2, 4, 3, Bludgeoning),
                attack_name: "Ram",
                reach_m: 1,
            },
            glyph: 'q',
            color: Color::White,
            bold: false,
            habitats: &[Cliffs],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Hawk",
                abilities: ab(5, 16, 8, 2, 14, 6),
                max_hp: 1,
                speed_m: feet_to_meters(10),
                armor_class: 13,
                attack_bonus: 5,
                damage: dice(1, 1, 0, Slashing),
                attack_name: "Talons",
                reach_m: 1,
            },
            glyph: 'B',
            color: Color::Yellow,
            bold: false,
            habitats: &[Cliffs],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Blood Hawk",
                abilities: ab(6, 14, 10, 3, 14, 5),
                max_hp: 7,
                speed_m: feet_to_meters(10),
                armor_class: 12,
                attack_bonus: 4,
                damage: dice(1, 4, 2, Piercing),
                attack_name: "Beak",
                reach_m: 1,
            },
            glyph: 'B',
            color: Color::Red,
            bold: false,
            habitats: &[Cliffs],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Giant Eagle",
                abilities: ab(16, 17, 13, 8, 14, 10),
                max_hp: 26,
                speed_m: feet_to_meters(10),
                armor_class: 13,
                attack_bonus: 5,
                damage: dice(2, 6, 3, Slashing),
                attack_name: "Talons",
                reach_m: 1,
            },
            glyph: 'B',
            color: Color::White,
            bold: false,
            habitats: &[Cliffs],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Giant Vulture",
                abilities: ab(15, 10, 15, 6, 12, 7),
                max_hp: 22,
                speed_m: feet_to_meters(10),
                armor_class: 10,
                attack_bonus: 3,
                damage: dice(2, 6, 2, Slashing),
                attack_name: "Talons",
                reach_m: 1,
            },
            glyph: 'B',
            color: Color::DarkGray,
            bold: false,
            habitats: &[Cliffs],
        },
        CreatureSpec {
            template: CreatureTemplate {
                name: "Gargoyle",
                abilities: ab(15, 11, 16, 6, 11, 7),
                max_hp: 52,
                speed_m: feet_to_meters(30),
                armor_class: 15,
                attack_bonus: 4,
                damage: dice(1, 6, 2, Slashing),
                attack_name: "Claws",
                reach_m: 1,
            },
            glyph: 'g',
            color: Color::Gray,
            bold: false,
            habitats: &[Cliffs, Urban],
        },
    ]
}

/// All creatures whose habitat list includes `habitat`.
pub fn creatures_in(habitat: Habitat) -> Vec<CreatureSpec> {
    bestiary()
        .into_iter()
        .filter(|spec| spec.habitats.contains(&habitat))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ALL_HABITATS, bestiary, creatures_in};
    use ratatui::style::Color;

    #[test]
    fn bestiary_has_forty_creatures() {
        assert_eq!(bestiary().len(), 40);
    }

    #[test]
    fn every_habitat_is_populated() {
        for habitat in ALL_HABITATS {
            assert!(
                !creatures_in(habitat).is_empty(),
                "habitat {habitat:?} has no creatures"
            );
        }
    }

    #[test]
    fn specs_are_sane() {
        for spec in bestiary() {
            assert!(
                spec.template.max_hp >= 1,
                "{} has non-positive HP",
                spec.template.name
            );
            assert!(
                !spec.glyph.is_whitespace(),
                "{} has a whitespace glyph",
                spec.template.name
            );
            assert!(
                !spec.habitats.is_empty(),
                "{} has no habitat",
                spec.template.name
            );
        }
    }

    #[test]
    fn spot_checks_match_nethack_conventions() {
        let all = bestiary();
        let find = |name: &str| all.iter().find(|s| s.template.name == name).copied();

        let squirrel = find("Squirrel").expect("squirrel present");
        assert_eq!(squirrel.glyph, 'r');
        assert_eq!(squirrel.color, Color::Gray);

        let skeleton = find("Skeleton").expect("skeleton present");
        assert_eq!(skeleton.glyph, 'Z');
        assert_eq!(skeleton.color, Color::White);

        let crocodile = find("Crocodile").expect("crocodile present");
        assert_eq!(crocodile.glyph, ':');
    }
}
