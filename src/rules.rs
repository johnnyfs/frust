//! Small hand-authored subset of SRD 5.2.1 rules data.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeciesKind {
    Human,
    Elf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassKind {
    Fighter,
    Rogue,
    Cleric,
    Wizard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityScores {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl AbilityScores {
    pub fn modifier(self, ability: Ability) -> i32 {
        ability_modifier(match ability {
            Ability::Strength => self.strength,
            Ability::Dexterity => self.dexterity,
            Ability::Constitution => self.constitution,
            Ability::Intelligence => self.intelligence,
            Ability::Wisdom => self.wisdom,
            Ability::Charisma => self.charisma,
        })
    }
}

pub fn ability_modifier(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

pub fn feet_to_meters(feet: i32) -> i32 {
    ((feet as f32) * 0.3048).round() as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Bludgeoning,
    Piercing,
    Slashing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageDice {
    pub count: u8,
    pub sides: u8,
    pub bonus: i32,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Longsword,
    Dagger,
    Mace,
    Quarterstaff,
    Bite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Weapon {
    pub kind: WeaponKind,
    pub name: &'static str,
    pub damage: DamageDice,
    pub attack_ability: Ability,
    pub proficient: bool,
}

impl Weapon {
    pub fn for_kind(kind: WeaponKind) -> Self {
        match kind {
            WeaponKind::Longsword => Self {
                kind,
                name: "Longsword",
                damage: DamageDice {
                    count: 1,
                    sides: 8,
                    bonus: 0,
                    damage_type: DamageType::Slashing,
                },
                attack_ability: Ability::Strength,
                proficient: true,
            },
            WeaponKind::Dagger => Self {
                kind,
                name: "Dagger",
                damage: DamageDice {
                    count: 1,
                    sides: 4,
                    bonus: 0,
                    damage_type: DamageType::Piercing,
                },
                attack_ability: Ability::Dexterity,
                proficient: true,
            },
            WeaponKind::Mace => Self {
                kind,
                name: "Mace",
                damage: DamageDice {
                    count: 1,
                    sides: 6,
                    bonus: 0,
                    damage_type: DamageType::Bludgeoning,
                },
                attack_ability: Ability::Strength,
                proficient: true,
            },
            WeaponKind::Quarterstaff => Self {
                kind,
                name: "Quarterstaff",
                damage: DamageDice {
                    count: 1,
                    sides: 6,
                    bonus: 0,
                    damage_type: DamageType::Bludgeoning,
                },
                attack_ability: Ability::Strength,
                proficient: true,
            },
            WeaponKind::Bite => Self {
                kind,
                name: "Bite",
                damage: DamageDice {
                    count: 1,
                    sides: 1,
                    bonus: 0,
                    damage_type: DamageType::Piercing,
                },
                attack_ability: Ability::Dexterity,
                proficient: false,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorKind {
    None,
    Leather,
    ChainMail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Armor {
    pub kind: ArmorKind,
    pub name: &'static str,
    pub base_ac: i32,
    pub dexterity_limit: Option<i32>,
}

impl Armor {
    pub fn for_kind(kind: ArmorKind) -> Self {
        match kind {
            ArmorKind::None => Self {
                kind,
                name: "None",
                base_ac: 10,
                dexterity_limit: None,
            },
            ArmorKind::Leather => Self {
                kind,
                name: "Leather Armor",
                base_ac: 11,
                dexterity_limit: None,
            },
            ArmorKind::ChainMail => Self {
                kind,
                name: "Chain Mail",
                base_ac: 16,
                dexterity_limit: Some(0),
            },
        }
    }
}

pub fn armor_class(abilities: AbilityScores, armor: ArmorKind, shield: bool) -> i32 {
    let armor = Armor::for_kind(armor);
    let dexterity = abilities.modifier(Ability::Dexterity);
    let dexterity =
        armor.dexterity_limit.map_or(
            dexterity,
            |limit| {
                if limit == 0 { 0 } else { dexterity.min(limit) }
            },
        );
    armor.base_ac + dexterity + if shield { 2 } else { 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterTemplate {
    pub name: &'static str,
    pub species: SpeciesKind,
    pub class: ClassKind,
    pub abilities: AbilityScores,
    pub max_hp: i32,
    pub speed_m: i32,
    pub armor: ArmorKind,
    pub shield: bool,
    pub weapon: WeaponKind,
}

impl CharacterTemplate {
    pub fn armor_class(self) -> i32 {
        armor_class(self.abilities, self.armor, self.shield)
    }

    pub fn attack_bonus(self) -> i32 {
        let weapon = Weapon::for_kind(self.weapon);
        self.abilities.modifier(weapon.attack_ability)
            + if weapon.proficient {
                proficiency_bonus(1)
            } else {
                0
            }
    }

    pub fn damage_dice(self) -> DamageDice {
        let weapon = Weapon::for_kind(self.weapon);
        DamageDice {
            bonus: self.abilities.modifier(weapon.attack_ability),
            ..weapon.damage
        }
    }
}

pub fn proficiency_bonus(_level: u8) -> i32 {
    2
}

pub fn starter_party() -> [CharacterTemplate; 4] {
    [
        CharacterTemplate {
            name: "Mara",
            species: SpeciesKind::Human,
            class: ClassKind::Fighter,
            abilities: AbilityScores {
                strength: 16,
                dexterity: 12,
                constitution: 14,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            max_hp: 12,
            speed_m: feet_to_meters(30),
            armor: ArmorKind::ChainMail,
            shield: false,
            weapon: WeaponKind::Longsword,
        },
        CharacterTemplate {
            name: "Ilyra",
            species: SpeciesKind::Elf,
            class: ClassKind::Rogue,
            abilities: AbilityScores {
                strength: 10,
                dexterity: 16,
                constitution: 12,
                intelligence: 12,
                wisdom: 10,
                charisma: 13,
            },
            max_hp: 9,
            speed_m: feet_to_meters(30),
            armor: ArmorKind::Leather,
            shield: false,
            weapon: WeaponKind::Dagger,
        },
        CharacterTemplate {
            name: "Tovin",
            species: SpeciesKind::Human,
            class: ClassKind::Cleric,
            abilities: AbilityScores {
                strength: 14,
                dexterity: 10,
                constitution: 14,
                intelligence: 10,
                wisdom: 16,
                charisma: 11,
            },
            max_hp: 10,
            speed_m: feet_to_meters(30),
            armor: ArmorKind::ChainMail,
            shield: true,
            weapon: WeaponKind::Mace,
        },
        CharacterTemplate {
            name: "Sable",
            species: SpeciesKind::Elf,
            class: ClassKind::Wizard,
            abilities: AbilityScores {
                strength: 8,
                dexterity: 14,
                constitution: 14,
                intelligence: 16,
                wisdom: 12,
                charisma: 10,
            },
            max_hp: 8,
            speed_m: feet_to_meters(30),
            armor: ArmorKind::None,
            shield: false,
            weapon: WeaponKind::Quarterstaff,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureTemplate {
    pub name: &'static str,
    pub abilities: AbilityScores,
    pub max_hp: i32,
    pub speed_m: i32,
    pub armor_class: i32,
    pub attack_bonus: i32,
    pub damage: DamageDice,
    /// Name of the creature's natural attack, e.g. "Bite", "Claws", "Slam".
    pub attack_name: &'static str,
    /// Reach of the attack in meters (1 for most Small/Medium creatures).
    pub reach_m: i32,
}

pub fn squirrel_template() -> CreatureTemplate {
    CreatureTemplate {
        name: "Squirrel",
        abilities: AbilityScores {
            strength: 2,
            dexterity: 11,
            constitution: 9,
            intelligence: 2,
            wisdom: 10,
            charisma: 4,
        },
        max_hp: 1,
        speed_m: feet_to_meters(20),
        attack_name: "Bite",
        reach_m: 1,
        armor_class: 10,
        attack_bonus: 2,
        damage: DamageDice {
            count: 1,
            sides: 1,
            bonus: 0,
            damage_type: DamageType::Piercing,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Ability, ArmorKind, ability_modifier, armor_class, feet_to_meters, squirrel_template,
    };

    #[test]
    fn ability_modifiers_use_srd_flooring() {
        assert_eq!(ability_modifier(16), 3);
        assert_eq!(ability_modifier(10), 0);
        assert_eq!(ability_modifier(9), -1);
        assert_eq!(ability_modifier(1), -5);
    }

    #[test]
    fn speed_feet_rounds_to_meters() {
        assert_eq!(feet_to_meters(30), 9);
        assert_eq!(feet_to_meters(20), 6);
    }

    #[test]
    fn armor_class_accounts_for_armor_shield_and_dexterity() {
        let abilities = super::AbilityScores {
            strength: 10,
            dexterity: 16,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };

        assert_eq!(armor_class(abilities, ArmorKind::None, false), 13);
        assert_eq!(armor_class(abilities, ArmorKind::Leather, false), 14);
        assert_eq!(armor_class(abilities, ArmorKind::ChainMail, true), 18);
    }

    #[test]
    fn squirrel_uses_rat_like_basics() {
        let squirrel = squirrel_template();

        assert_eq!(squirrel.max_hp, 1);
        assert_eq!(squirrel.speed_m, 6);
        assert_eq!(squirrel.armor_class, 10);
        assert_eq!(squirrel.attack_bonus, 2);
        assert_eq!(squirrel.attack_name, "Bite");
        assert_eq!(squirrel.reach_m, 1);
        assert_eq!(squirrel.abilities.modifier(Ability::Dexterity), 0);
    }
}
