//! Defines stats, saving throws, skills, and proficieny.

use std::{
        collections::HashSet, 
        ops::{Add, Deref, DerefMut, Sub}
};
use strum::{EnumIter, IntoEnumIterator};

use serde::{Serialize, Deserialize};

// proficiency bonus values for each level
pub const PROFICIENCY_BY_LEVEL: [isize; 20] = 
    [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6];


/// Base ability scores.
/// These are total scores, not modifiers.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub strength: isize,
    pub dexterity: isize,
    pub constitution: isize,
    pub wisdom: isize,
    pub intelligence: isize,
    pub charisma: isize,
}

impl From<&[isize; 6]> for Stats {
    fn from(arr: &[isize; 6]) -> Self {
        Self {
            strength:       arr[0],
            dexterity:      arr[1],
            constitution:   arr[2],
            intelligence:   arr[3],
            wisdom:         arr[4],
            charisma:       arr[5],
        }
    }
}

impl From<Stats> for Vec<isize> {
    fn from(value: Stats) -> Self {
        vec![value.strength, value.dexterity, value.constitution, value.intelligence, value.wisdom, value.charisma]
    }
}

impl Stats {
    /// Returns the modifier for each ability score.
    ///
    /// Modifiers are computed as floor((score - 10) / 2)
    pub fn modifiers(&self) -> Modifiers {

        fn calc_mod(stat: isize) -> isize {
            ( ((stat as f64)-10.0)/2.0 ).floor() as isize
        }

        Modifiers{ stats: Stats {
            strength: calc_mod(self.strength),
            dexterity: calc_mod(self.dexterity),
            constitution: calc_mod(self.constitution),
            wisdom: calc_mod(self.wisdom),
            intelligence: calc_mod(self.intelligence),
            charisma: calc_mod(self.charisma),
        }}
    }
    
    /// Returns a mutable refrence to the value of the given stat type.
    pub fn get_stat_type_mut (&mut self, stat_type: &StatType) -> &mut isize {
        match stat_type {
           StatType::Strength => &mut self.strength,
           StatType::Dexterity => &mut self.dexterity,
           StatType::Constitution => &mut self.constitution,
           StatType::Intelligence => &mut self.intelligence,
           StatType::Wisdom => &mut self.wisdom,
           StatType::Charisma => &mut self.charisma,
        }
    }

    /// Returns a refrence to the value of the given stat type.
    pub fn get_stat_type(&self, stat_type: &StatType) -> &isize {
        match stat_type {
            StatType::Strength => &self.strength,
            StatType::Dexterity => &self.dexterity,
            StatType::Constitution => &self.constitution,
            StatType::Intelligence => &self.intelligence,
            StatType::Wisdom => &self.wisdom,
            StatType::Charisma => &self.charisma,
        }
    }

}

impl Add for Stats {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            strength: self.strength + rhs.strength,
            dexterity: self.dexterity + rhs.dexterity,
            constitution: self.constitution + rhs.constitution,
            wisdom: self.wisdom + rhs.wisdom,
            intelligence: self.intelligence + rhs.intelligence,
            charisma: self.charisma + rhs.charisma,
        }
    }
}

impl Add<isize> for Stats {
    type Output = Self;
    fn add(self, rhs: isize) -> Self::Output {
        Self {
            strength: self.strength + rhs,
            dexterity: self.dexterity + rhs,
            constitution: self.constitution + rhs,
            wisdom: self.wisdom + rhs,
            intelligence: self.intelligence + rhs,
            charisma: self.charisma + rhs,
        }
    }
}

impl Sub for Stats {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            strength: self.strength - rhs.strength,
            dexterity: self.dexterity - rhs.dexterity,
            constitution: self.constitution - rhs.constitution,
            wisdom: self.wisdom - rhs.wisdom,
            intelligence: self.intelligence - rhs.intelligence,
            charisma: self.charisma - rhs.charisma,
        }
    }
}

impl Sub<isize> for Stats {
    type Output = Self;
    fn sub(self, rhs: isize) -> Self::Output {
        self.add(-rhs)
    }
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            wisdom: 10,
            intelligence: 10,
            charisma: 10
        }
    }
}

// A wrapper for [Stats] where each field is a modifier instead of a base score.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Modifiers {pub stats: Stats}

impl Deref for Modifiers {
    type Target = Stats;
    fn deref(&self) -> &Self::Target {
        &self.stats
    }
}
impl DerefMut for Modifiers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stats
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self { stats: Stats { 
            strength: 0, 
            dexterity: 0, 
            constitution: 0, 
            wisdom: 0, 
            intelligence: 0, 
            charisma: 0 
        }}
    }
}

/// Enumerates all six core ability score types. 
#[derive(Clone, Copy, Debug, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum StatType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl <'a>StatType {
    /// Getting a StatType from it's shorthand (e.g. "dex")
    /// if you want to get a StatType from it's full name, just take the first three characters.
    ///
    /// Returns the associated [StatType] of a 3 character string, or otherwise returns an `Err(())`.
    pub fn from_shorthand(shorthand: &str) -> Option<StatType> {
        match shorthand.to_lowercase().as_str() {
            "str" => Some(StatType::Strength),
            "dex" => Some(StatType::Dexterity),
            "con" => Some(StatType::Constitution),
            "int" => Some(StatType::Intelligence),
            "wis" => Some(StatType::Wisdom),
            "cha" => Some(StatType::Charisma),
            _ => None
        }
    }

    /// Get the string name of a stat type.
    pub fn get_name(&'a self) -> &'a str {
        match self {
            StatType::Strength => "Strength",
            StatType::Dexterity => "Dexterity",
            StatType::Constitution => "Constitution",
            StatType::Intelligence => "Intelligence",
            StatType::Wisdom => "Wisdom",
            StatType::Charisma => "Charisma",
        }
    }

    /// Get the shorthand of a stat type.
    ///
    /// this is equivalent to `get_name()[..3]`, and can be used as a deterministic name for the
    /// stat.
    pub fn get_shorthand(&'a self) -> &'a str {
        &self.get_name()[..3]
    }
}


/// Boolean flags indicating proficiency in each saving throw.
///
/// A "true" field means the character is proficient in that save.
///
/// If you need the numeric modfier of a save, use [modifiers](Saves::modifiers).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Saves {
    pub strength: bool,
    pub dexterity: bool,
    pub constitution: bool,
    pub intelligence: bool,
    pub wisdom: bool,
    pub charisma: bool,
}

impl Saves {
    /// Returns the total saving throw modifiers, combining ability modifiers and any proficieny
    /// bonuses.
    pub fn modifiers(&self, stats: &Stats, proficiency_bonus: isize) -> Modifiers {

        // essentially just takes the base stats, converts it to the modfiers, then adds
        // proficiency to the fields that are proficient.


        let calc_bonus = |b| if b {proficiency_bonus} else {0};
            
        let base_modifiers = stats.modifiers();

        Modifiers{ stats: Stats { 
            strength: base_modifiers.strength + calc_bonus(self.strength),
            dexterity: base_modifiers.dexterity + calc_bonus(self.dexterity),
            constitution: base_modifiers.constitution + calc_bonus(self.constitution),
            intelligence: base_modifiers.intelligence + calc_bonus(self.intelligence),
            wisdom: base_modifiers.wisdom + calc_bonus(self.wisdom),
            charisma: base_modifiers.charisma + calc_bonus(self.charisma),
        }}
    }

    /// Add a saving throw proficiency from an ability score type
    pub fn add_proficiency_from_type(&mut self, stat_type: StatType) {
       match stat_type {
           StatType::Strength => self.strength = true,
           StatType::Dexterity => self.dexterity = true,
           StatType::Constitution => self.constitution = true,
           StatType::Intelligence => self.intelligence = true,
           StatType::Wisdom => self.wisdom = true,
           StatType::Charisma => self.charisma = true,
       }
    }
}


/// Tracks proficiency and expertise for every skill.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SkillProficiencies {
    pub acrobatics: Skill,
    pub animal_handling: Skill,
    pub arcana: Skill,
    pub athletics: Skill,
    pub deception: Skill,
    pub history: Skill,
    pub insight: Skill,
    pub intimidation: Skill,
    pub investigation: Skill,
    pub medicine: Skill,
    pub nature: Skill,
    pub perception: Skill,
    pub performance: Skill,
    pub persuasion: Skill,
    pub religion: Skill,
    pub sleight_of_hand: Skill,
    pub stealth: Skill,
    pub survival: Skill,
}
 
/// Enumerates the different skills a character has. (e.g. Deception, Religion, Medicine)
#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
#[derive(EnumIter)]
#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum SkillType {
    /// Uses dexterity
    Acrobatics,
    /// Uses wisdom
    AnimalHandling,
    /// Uses intelligence
    Arcana,
    /// Uses strength
    Athletics,
    /// Uses charisma
    Deception,
    /// Uses intelligence
    History,
    /// Uses wisdom
    Insight,
    /// Uses charisma
    Intimidation,
    /// Uses intelligence
    Investigation,
    /// Uses wisdom
    Medicine,
    /// Uses intelligence
    Nature,
    /// Uses wisdom
    Perception,
    /// Uses charisma
    Performance,
    /// Uses charisma
    Persuasion,
    /// Uses intelligence
    Religion,
    /// Uses dexterity
    SleightOfHand,
    /// Uses dexterity
    Stealth,
    /// Uses wisdom
    Survival,
}


impl SkillType {
    // converts from string name to skill type. non case sensitive.
    pub fn from_name(name: &str) -> Option<SkillType> {
        match name.to_lowercase().as_str() {
            "acrobatics" => Some(SkillType::Acrobatics),
            "animal handling" => Some(SkillType::AnimalHandling),
            "arcana" => Some(SkillType::Arcana),
            "athletics" => Some(SkillType::Athletics),
            "deception" => Some(SkillType::Deception),
            "history" => Some(SkillType::History),
            "insight" => Some(SkillType::Insight),
            "intimidation" => Some(SkillType::Intimidation),
            "investigation" => Some(SkillType::Investigation),
            "medicine" => Some(SkillType::Medicine),
            "nature" => Some(SkillType::Nature),
            "perception" => Some(SkillType::Perception),
            "performance" => Some(SkillType::Performance),
            "persuasion" => Some(SkillType::Persuasion),
            "religion" => Some(SkillType::Religion),
            "sleight of hand" => Some(SkillType::SleightOfHand),
            "stealth" => Some(SkillType::Stealth),
            "survival" => Some(SkillType::Survival),
            _ => None,
        }
    }
}


/// Stores the proficiency/mastery of a single skill type.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Skill {
    pub proficiency: bool,
    pub expertise: bool,
}


/// Calculated modifiers for skills
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillModifiers {
    pub acrobatics: isize,
    pub animal_handling: isize,
    pub arcana: isize,
    pub athletics: isize,
    pub deception: isize,
    pub history: isize,
    pub insight: isize,
    pub intimidation: isize,
    pub investigation: isize,
    pub medicine: isize,
    pub nature: isize,
    pub perception: isize,
    pub performance: isize,
    pub persuasion: isize,
    pub religion: isize,
    pub sleight_of_hand: isize,
    pub stealth: isize,
    pub survival: isize,
}

impl SkillModifiers {
    pub fn get_skill_type(&self, skill_type: SkillType) -> &isize {
        match skill_type {
            SkillType::Acrobatics => &self.acrobatics,
            SkillType::AnimalHandling => &self.animal_handling,
            SkillType::Arcana => &self.arcana,
            SkillType::Athletics => &self.athletics,
            SkillType::Deception => &self.deception,
            SkillType::History => &self.history,
            SkillType::Insight => &self.insight,
            SkillType::Intimidation => &self.intimidation,
            SkillType::Investigation => &self.investigation,
            SkillType::Medicine => &self.medicine,
            SkillType::Nature => &self.nature,
            SkillType::Perception => &self.perception,
            SkillType::Performance => &self.performance,
            SkillType::Persuasion => &self.persuasion,
            SkillType::Religion => &self.religion,
            SkillType::SleightOfHand => &self.sleight_of_hand,
            SkillType::Stealth => &self.stealth,
            SkillType::Survival => &self.survival,
        }
    }
    pub fn get_skill_type_mut(&mut self, skill_type: SkillType) -> &mut isize {
        match skill_type {
            SkillType::Acrobatics => &mut self.acrobatics,
            SkillType::AnimalHandling => &mut self.animal_handling,
            SkillType::Arcana => &mut self.arcana,
            SkillType::Athletics => &mut self.athletics,
            SkillType::Deception => &mut self.deception,
            SkillType::History => &mut self.history,
            SkillType::Insight => &mut self.insight,
            SkillType::Intimidation => &mut self.intimidation,
            SkillType::Investigation => &mut self.investigation,
            SkillType::Medicine => &mut self.medicine,
            SkillType::Nature => &mut self.nature,
            SkillType::Perception => &mut self.perception,
            SkillType::Performance => &mut self.performance,
            SkillType::Persuasion => &mut self.persuasion,
            SkillType::Religion => &mut self.religion,
            SkillType::SleightOfHand => &mut self.sleight_of_hand,
            SkillType::Stealth => &mut self.stealth,
            SkillType::Survival => &mut self.survival,
        }
    }

}

impl SkillProficiencies {
    /// Computes total modifiers for all skills based on ability modifiers and proficiency bonuses.
    ///
    /// Proficency in a skill adds proficiency once. Expertise adds the proficency bonus again.
    pub fn modifiers(&self, stats: &Stats, proficiency_bonus: isize) -> SkillModifiers{
        // stat modifiers. Shorthanded name since it's a very short lived and highly used var.
        let sm = stats.modifiers();
        // proficiency modifier
        // calculates how much is added due to the proficiency bonus and mastery, if any
        let pm = |s: &Skill| proficiency_bonus * (s.proficiency as isize + s.expertise as isize);

        SkillModifiers {
            acrobatics: sm.dexterity + pm(&self.acrobatics),
            animal_handling: sm.wisdom + pm(&self.animal_handling),
            arcana: sm.intelligence + pm(&self.arcana),
            athletics: sm.strength + pm(&self.athletics),
            deception: sm.charisma + pm(&self.deception),
            history: sm.intelligence + pm(&self.history),
            insight: sm.wisdom + pm(&self.insight),
            intimidation: sm.charisma + pm(&self.intimidation),
            investigation: sm.intelligence + pm(&self.investigation),
            medicine: sm.wisdom + pm(&self.medicine),
            nature: sm.intelligence + pm(&self.nature),
            perception: sm.wisdom + pm(&self.perception),
            performance: sm.charisma + pm(&self.performance),
            persuasion: sm.charisma + pm(&self.persuasion),
            religion: sm.intelligence + pm(&self.religion),
            sleight_of_hand: sm.dexterity + pm(&self.sleight_of_hand),
            stealth: sm.dexterity + pm(&self.stealth),
            survival: sm.wisdom + pm(&self.survival),
        }
    }


    /// Gets a reference to the skill data for the specified skill type
    pub fn get_from_type(&self, stat_type: SkillType) -> &Skill {
        match stat_type {
            SkillType::Acrobatics => &self.acrobatics,
            SkillType::AnimalHandling => &self.animal_handling,
            SkillType::Arcana => &self.arcana,
            SkillType::Athletics => &self.athletics,
            SkillType::Deception => &self.deception,
            SkillType::History => &self.history,
            SkillType::Insight => &self.insight,
            SkillType::Intimidation => &self.intimidation,
            SkillType::Investigation => &self.investigation,
            SkillType::Medicine => &self.medicine,
            SkillType::Nature => &self.nature,
            SkillType::Perception => &self.perception,
            SkillType::Performance => &self.performance,
            SkillType::Persuasion => &self.persuasion,
            SkillType::Religion => &self.religion,
            SkillType::SleightOfHand => &self.sleight_of_hand,
            SkillType::Stealth => &self.stealth,
            SkillType::Survival => &self.survival,
        }
    }

    /// Gets a reference to the skill data for the specified skill type
    pub fn get_mut_from_type(&mut self, stat_type: SkillType) -> &mut Skill {
        match stat_type {
            SkillType::Acrobatics => &mut self.acrobatics,
            SkillType::AnimalHandling => &mut self.animal_handling,
            SkillType::Arcana => &mut self.arcana,
            SkillType::Athletics => &mut self.athletics,
            SkillType::Deception => &mut self.deception,
            SkillType::History => &mut self.history,
            SkillType::Insight => &mut self.insight,
            SkillType::Intimidation => &mut self.intimidation,
            SkillType::Investigation => &mut self.investigation,
            SkillType::Medicine => &mut self.medicine,
            SkillType::Nature => &mut self.nature,
            SkillType::Perception => &mut self.perception,
            SkillType::Performance => &mut self.performance,
            SkillType::Persuasion => &mut self.persuasion,
            SkillType::Religion => &mut self.religion,
            SkillType::SleightOfHand => &mut self.sleight_of_hand,
            SkillType::Stealth => &mut self.stealth,
            SkillType::Survival => &mut self.survival,
        }
    }

    pub fn add_proficiency_from_type(&mut self, stat_type: SkillType) {
        self.get_mut_from_type(stat_type).proficiency = true;
    }

    pub fn add_expertise_from_type(&mut self, stat_type: SkillType) {
        self.get_mut_from_type(stat_type).expertise = true;
    }

    /// Returns a vector of the skills that have proficiency.
    pub fn skills_with_proficiency(&self) -> Vec<(SkillType, &Skill)> {
        let mut v = vec![];
        for t in SkillType::iter() {
            let x = self.get_from_type(t);
            if x.proficiency {v.push((t,x))}
        }
        v
    }
}


/// Proficiencies in armor and weapons.
///
/// The other field holds other etc proficiencies, like shortswords and land vehicles.
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(Default)]
pub struct EquipmentProficiencies {
    pub simple_weapons: bool,
    pub martial_weapons: bool,
    pub light_armor: bool,
    pub medium_armor: bool,
    pub heavy_armor: bool,
    pub shields: bool,
    pub other: HashSet<String>,
}

/// Represents the different types of speed any creature can have. E.g. hovering, climbing,
/// swimming
/// 
/// Most of these are only used in rare cases. The walking speed is almost always a given.
pub struct Speeds {
    pub walking: Option<usize>,
    pub flying: Option<usize>,
    pub hovering: Option<usize>,
    pub burrowing: Option<usize>,
    pub climbing: Option<usize>,
    pub swimming: Option<usize>,
}
