//! Stats, Skills, and other number-based data for characters.
use std::{collections::HashSet, ops::{Add, Deref, DerefMut, Sub}};
use strum::{EnumIter, IntoEnumIterator};

use serde::{Serialize, Deserialize};

// proficiency bonus values for each level
pub const PROFICIENCY_BY_LEVEL: [isize; 20] = [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6];


/// Base stats.
/// These are total scores, not modifiers.
/// 
#[derive(PartialEq)]
#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
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
        Stats {
            strength:       arr[0],
            dexterity:      arr[1],
            constitution:   arr[2],
            intelligence:   arr[3],
            wisdom:         arr[4],
            charisma:       arr[5],
        }
    }
}

impl Into<Vec<isize>> for Stats {
    fn into(self) -> Vec<isize> {
        vec![self.strength, self.dexterity, self.constitution, self.intelligence, self.wisdom, self.charisma]
    }
}

impl Stats {
    pub fn from_arr(arr: &[isize; 6]) -> Stats{
        arr.into()
    }

    pub fn to_vec(&self) -> Vec<isize> {
        self.clone().into()
    }

    /// Gets the stat modifiers from base scores.
    pub fn modifiers(&self) -> Modifiers {

        fn calc_mod(stat: isize) -> isize {
            ( ((stat as f64)-10.0)/2.0 ).floor() as isize
        }

        Modifiers(Stats {
            strength: calc_mod(self.strength),
            dexterity: calc_mod(self.dexterity),
            constitution: calc_mod(self.constitution),
            wisdom: calc_mod(self.wisdom),
            intelligence: calc_mod(self.intelligence),
            charisma: calc_mod(self.charisma),
        })
    }
    
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

/// A struct holding modifiers for stats.
///
/// It's just a wrapper around a Stats instance, since modifiers are essentially just stats on a different scale.
pub struct Modifiers(pub Stats);

impl Deref for Modifiers {
    type Target = Stats;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Modifiers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// An individual stat type.
#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
#[derive(EnumIter)]
#[derive(Debug)]
#[derive(Clone, Copy)]
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
    pub fn from_shorthand(shorthand: &str) -> Result<StatType, ()> {
        match shorthand.to_lowercase().as_str() {
            "str" => Ok(StatType::Strength),
            "dex" => Ok(StatType::Dexterity),
            "con" => Ok(StatType::Constitution),
            "int" => Ok(StatType::Intelligence),
            "wis" => Ok(StatType::Wisdom),
            "cha" => Ok(StatType::Charisma),
            _ => Err(())
        }
    }

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

    pub fn get_shorthand(&'a self) -> &'a str {
        &self.get_name()[..3]
    }
}


/// A character's saving throw proficiencies.
///
/// If you need the number modfier of a save, use [modifiers](Saves::modifiers).
#[derive(Default)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct Saves {
    pub strength: bool,
    pub dexterity: bool,
    pub constitution: bool,
    pub intelligence: bool,
    pub wisdom: bool,
    pub charisma: bool,
}

impl Saves {
    /// Gets the individual save modifiers.
    ///
    /// An individual Saves struct only shows if a character has proficieny or not in each skill,
    /// this calculates the actual modifiers used for the save.
    pub fn modifiers(&self, stats: &Stats, proficiency_bonus: isize) -> Modifiers {

        // essentially just takes the base stats, converts it to the modfiers, then adds
        // proficiency to the fields that are proficient.

        let calc_bonus = |b| if b {proficiency_bonus} else {0};
            
        let base_modifiers = stats.modifiers();
        Modifiers(Stats {
            strength: base_modifiers.strength + calc_bonus(self.strength),
            dexterity: base_modifiers.dexterity + calc_bonus(self.dexterity),
            constitution: base_modifiers.constitution + calc_bonus(self.constitution),
            intelligence: base_modifiers.intelligence + calc_bonus(self.intelligence),
            wisdom: base_modifiers.wisdom + calc_bonus(self.wisdom),
            charisma: base_modifiers.charisma + calc_bonus(self.charisma),
        })
    }

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


/// Stores the proficieny/mastery for all skills 
#[derive(Default)]
#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
#[derive(EnumIter)]
#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum SkillType {
    Acrobatics,
    AnimalHandling,
    Arcana,
    Athletics,
    Deception,
    History,
    Insight,
    Intimidation,
    Investigation,
    Medicine,
    Nature,
    Perception,
    Performance,
    Persuasion,
    Religion,
    SleightOfHand,
    Stealth,
    Survival,
}


impl SkillType {
    // converts from string name to skill type. non case sensitive.
    pub fn from_name(name: &str) -> Result<SkillType, ()> {
        match name.to_lowercase().as_str() {
            "acrobatics" => Ok(SkillType::Acrobatics),
            "animal handling" => Ok(SkillType::AnimalHandling),
            "arcana" => Ok(SkillType::Arcana),
            "athletics" => Ok(SkillType::Athletics),
            "deception" => Ok(SkillType::Deception),
            "history" => Ok(SkillType::History),
            "insight" => Ok(SkillType::Insight),
            "intimidation" => Ok(SkillType::Intimidation),
            "investigation" => Ok(SkillType::Investigation),
            "medicine" => Ok(SkillType::Medicine),
            "nature" => Ok(SkillType::Nature),
            "perception" => Ok(SkillType::Perception),
            "performance" => Ok(SkillType::Performance),
            "persuasion" => Ok(SkillType::Persuasion),
            "religion" => Ok(SkillType::Religion),
            "sleight of hand" => Ok(SkillType::SleightOfHand),
            "stealth" => Ok(SkillType::Stealth),
            "survival" => Ok(SkillType::Survival),
            _ => Err(())
        }
    }
}


/// Stores the proficiency/mastery of a single skill type.
#[derive(Default)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Skill {
    pub proficiency: bool,
    pub expertise: bool,
}


/// Calculated modifiers for skills
#[derive(Debug)]
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


impl PartialEq for SkillModifiers {
    fn eq(&self, other: &Self) -> bool {
        self.acrobatics == other.acrobatics
        && self.animal_handling == other.animal_handling
        && self.arcana == other.arcana
        && self.athletics == other.athletics
        && self.deception == other.deception
        && self.history == other.history
        && self.insight == other.insight
        && self.intimidation == other.intimidation
        && self.investigation == other.investigation
        && self.medicine == other.medicine
        && self.nature == other.nature
        && self.perception == other.perception
        && self.performance == other.performance
        && self.persuasion == other.persuasion
        && self.religion == other.religion
        && self.sleight_of_hand == other.sleight_of_hand
        && self.stealth == other.stealth
        && self.survival == other.survival
    }
}

impl SkillProficiencies {
    pub fn modifiers(&self, stats: &Stats, proficiency_bonus: isize) -> SkillModifiers{
        // stat modifiers. Shorthanded name since it's a very short lived and highly used var.
        let sm = stats.modifiers();
        // proficiency modifier
        // calculates how much is added due to the proficiency bonus and mastery, if any
        let pm = |skill: &Skill| {
            let mut total = 0;
            if skill.proficiency {
                total += proficiency_bonus;
            }
            if skill.expertise {
                total += proficiency_bonus;
            }
            total
        };

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
