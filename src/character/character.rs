use serde::{Serialize, Deserialize};

use crate::character::class::ItemCategory;

use super::background::Background;
use super::race::Race;
use super::stats::{Modifiers, Saves, SkillModifiers, SkillProficiencies, SkillType, StatType, Stats, PROFICIENCY_BY_LEVEL};
use super::features::{AbilityScoreIncrease, Feature, FeatureEffect, PresentedOption};
use super::choice::chosen;
use super::items::{Item, ItemType};
use super::spells::{SpellSlots, Spellcasting};
use super::class::{Class, Subclass};


/// A struct to represent a Dungeons and Dragons character.
///
/// In order to build a character, you need a [Class], a [Background], and a [Race].
/// To get one of these, you can either get them from the api using the [get](super::super::get) module,
/// or build them from scratch in the case of homebrew.
///
/// ```
/// #[tokio::main]
/// async fn main() {
///     use dnd_lib::get::{get_class, get_race, get_background};
///     use dnd_lib::character::{stats::Stats, Character};
/// 
///     let fighter = get_class("fighter").await.unwrap();
///     let human = get_race("human").await.unwrap();
///     let acolyte = get_background("acolyte").await.unwrap();
///
///     let john = Character::new(String::from("john"), &fighter, &acolyte, &human, Stats::default());
/// }
/// ```
///
/// Each class the character uses is represented by a [SpeccedClass] instance. If a character has 3 levels in wizard
///  and 1 level in fighter, they'll have a level 3 SpeccedClass wizard at classes\[0\] and a level 1
/// SpeccedClass fighter at classes\[1\].
///
#[derive(Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    /// Individual classes that the character has specced into.
    pub classes: Vec<SpeccedClass>,
    pub race: Race,
    pub background: Background,
    pub availible_spell_slots: Option<SpellSlots>,
    base_stats: Stats,
    /// Extra features from etc sources that aren't listed otherwise. Feel free to append on any
    /// extra feature you want your character to have.
    pub bonus_features: Vec<Feature>,
    /// The first field is the item, second is count, and 3rd is if it's equipped or not.
    pub items: Vec<(Item, usize, bool)>, 
    pub class_skill_proficiencies: Vec<PresentedOption<SkillType>>,
    class_saving_throw_proficiencies: Vec<StatType>,
    pub hp: usize,
    pub temp_hp: usize,
}

impl Character {
    pub fn new(name: String, class: &Class, background: &Background, race: &Race, base_stats: Stats) -> Character {
        
        let mut new_character = Character {
            name,
            classes: vec![SpeccedClass::from_class(class, 1)],
            background: background.clone(),
            items: vec![],
            race: race.clone(),
            base_stats,
            bonus_features: vec![],
            availible_spell_slots: None,
            class_skill_proficiencies: vec![class.skill_proficiency_choices.1.clone(); class.skill_proficiency_choices.0],
            class_saving_throw_proficiencies: class.saving_throw_proficiencies.clone(),
            hp: 1,
            temp_hp: 0,
        };

        // add background items
        let mut items = vec![];
        items.append(&mut new_character.background.equipment);
        new_character.add_item_list(items);
        new_character.hp = new_character.max_hp();

        new_character
    }

    fn add_item_list(&mut self, item_list: Vec<(Item, usize)>) {
        let mut new_item_list = item_list
            .into_iter()
            .map(|(i, c)| (i, c, false)).collect();
        self.items.append(&mut new_item_list)
    }

    /// Adds the class's items to the character, and removes those items from their [SpeccedClass] entry.
    /// ignores unchosen items.
    pub fn add_class_items(&mut self) {
        let mut items: Vec<(Item, usize)> = vec![];
        self.classes[0].items = self.classes[0].items.iter().filter(|o| {
            if let PresentedOption::Base(v) = o  {
                items.extend_from_slice(&Character::selected_items(v));
                false
            } else {true} 
        }).cloned().collect();
        self.add_item_list(items);
    }

    fn selected_items(items: &Vec<(ItemCategory, usize)>) -> Vec<(Item, usize)> {
        items.iter().filter_map(|v| if let ItemCategory::Item(i) = &v.0 {Some((i.clone(), v.1))} else {None}).collect()
    }

    /// Gets the character's total level by summing up all their class levels.
    pub fn level(&self) -> usize {
        self.classes.iter().map(|class| class.level).sum()
    }

    // in some cases level can be over 20.
    // This isn't officially supported, but it's nice to have a fallback
    // so it doesn't come crashing down.
    fn clamped_level(&self) -> usize {
        let l = self.level();
        if l > 20 {20} else {l}
    }

    /// Gets the character's proficiency bonus based on their level.
    pub fn proficiency_bonus(&self) -> isize {
        PROFICIENCY_BY_LEVEL[self.clamped_level()-1]
    }

    /// Gets the character's ability scores. 
    ///
    /// Note that this isn't modifiers, but rather base scores.
    pub fn stats(&self) -> Stats {
        let mut new_stats = self.base_stats.clone();

        for (race_stat_change, amount) in self.race.ability_bonuses.iter() {
            *new_stats.get_stat_type_mut(race_stat_change) += amount;
        }

        if let PresentedOption::Base(ref chosen_subrace) = self.race.subraces {
            for (subrace_stat, amount) in chosen_subrace.ability_bonuses.iter() {
                *new_stats.get_stat_type_mut(subrace_stat) += amount;
            }
        }

        // We also want to get features from the class.
        // This accounts for things like ability score increases.
        // We're also adding bonus features just in case.
        let feature_effects = self.class_features()
            .into_iter()
            .chain(self.bonus_features.iter())
            .map(|v| &v.effects)
            .flatten();

        for feature in feature_effects {
            match feature {
                FeatureEffect::AddModifier(stat, amount) => *new_stats.get_stat_type_mut(stat) += amount,
                FeatureEffect::AbilityScoreIncrease(AbilityScoreIncrease::StatIncrease(s1, s2)) => {
                    if let Some(s) = s1 {
                        *new_stats.get_stat_type_mut(s) += 1;
                    }
                    if let Some(s) = s2 {
                        *new_stats.get_stat_type_mut(s) += 1;
                    }
                }
                _ => (),
            }

        }


        new_stats
    }

    /// Saving throw proficiencies 
    pub fn saves(&self) -> Saves {
        let mut base = Saves::default();

        for save in self.class_saving_throw_proficiencies.iter() {
            base.add_proficiency_from_type(save.clone());
        }

        for effect in self.total_features().into_iter().map(|t| t.effects.iter()).flatten() {
            if let FeatureEffect::AddSaveProficiency(s) = effect {
                base.add_proficiency_from_type(*s);
            }
        }

        base
    }

    /// Saving throw modifiers
    pub fn save_mods(&self) -> Modifiers {
        let mut modifiers = self.saves().modifiers(&self.base_stats, self.proficiency_bonus());

        for effect in self.total_features().into_iter().map(|t| t.effects.iter()).flatten() {
            if let FeatureEffect::AddSaveModifier(t, m) = effect {
                *modifiers.0.get_stat_type_mut(&t) += m;
            }
        }

        modifiers
    }

    /// Skill proficiencies
    pub fn skills(&self) -> SkillProficiencies {
        let mut base = SkillProficiencies::default();
        let chosen_class_skills: Vec<&SkillType> = chosen(&self.class_skill_proficiencies);
        let background_skills: Vec<&SkillType> = chosen(&self.background.proficiencies);

        for skill in chosen_class_skills.iter().chain(background_skills.iter()) {
            let cloned_skill = (*skill).clone();
            base.add_proficiency_from_type(cloned_skill);
        }

        for effect in self.total_features().iter().map(|t| t.effects.iter()).flatten() { 
            match effect {
                FeatureEffect::AddSkillProficiency(s) => base.add_proficiency_from_type(*s),
                FeatureEffect::Expertise([s1, s2]) => {
                    if let Some(v1) = s1 {
                        base.add_expertise_from_type(*v1);
                    }
                    if let Some(v2) = s2 {
                        base.add_expertise_from_type(*v2);
                    }
                }
                _ => ()
            }
        }

        base
    }

    pub fn skill_modifiers(&self) -> SkillModifiers {
        let mut modifiers = self.skills().modifiers(&self.stats(), self.proficiency_bonus());

        for effect in self.total_features().iter().map(|t| t.effects.iter()).flatten() {
            match effect {
                FeatureEffect::AddSkillModifier(t, n) => *modifiers.get_skill_type_mut(*t) += *n,
                _ => (),
            }
        }

        modifiers
    }

    /// Gets refrences to every item marked as held
    pub fn equipped_items(&self) -> Vec<(&Item, &usize)> {
        // gets the items that are selected as held
        self.items.iter().filter_map(|(i, c, h)| if *h {Some((i,c))} else {None}).collect()
    }

    /// gets the spell save dc and spell attack modifier of the specified class. if class_index is
    /// 0, this is just the starting class.
    /// ```
    ///     use dnd_lib::get::{get_class, get_race, get_background};
    ///     use dnd_lib::character::{stats::Stats, Character};
    ///     #[tokio::main]
    ///     async fn main() {
    ///         let wizard =  get_class("wizard").await.unwrap();
    ///         let acolyte = get_background("acolyte").await.unwrap();
    ///         let elf = get_race("elf").await.unwrap();
    ///
    ///         // this is john. john has a base int score of 13, and john is a high elf. His int should be 14.
    ///         let stats = Stats::from_arr(&[10, 10, 10, 13, 10, 10]);
    ///         let mut john = Character::new(String::from("john"), &wizard, &acolyte, &elf, stats);
    ///         john.race.subraces.choose_in_place(0);
    ///
    ///         // An int of 14 is a modifier of 2.
    ///         assert_eq!(john.stats().modifiers().intelligence, 2);
    ///
    ///         // john should have a spell save dc of 12, and a spell attack modifier of 4.
    ///         let (spell_save, spell_mod) = john.spellcasting_scores(0).expect("wizard character should be a spellcaster");
    ///         assert_eq!(spell_save, 12);
    ///         assert_eq!(spell_mod, 4);
    ///     }
    /// ```
    pub fn spellcasting_scores(&self, class_index: usize) -> Option<(isize, isize)> {
        let spellcasting_ability = &self.classes.get(class_index)?
            .spellcasting.as_ref()?
            .spellcasting_ability;
        let spellcasting_mod = self.stats().modifiers().get_stat_type(&spellcasting_ability).clone();

        let spell_save_dc = 8 + self.proficiency_bonus() + spellcasting_mod;
        let spell_attack_mod = self.proficiency_bonus() + spellcasting_mod;

        Some((spell_save_dc, spell_attack_mod))
    }

    pub fn item_features(&self) -> Vec<&Feature> {
        self.items.iter()
            .map(|v| v.0.features.iter())
            .flatten()
            .collect()
    }

    /// Every feature currently granted by the character's class, not including subclass.
    pub fn class_features(&self) -> Vec<&Feature> {
        self.classes.iter()
            .map(|specced_class| specced_class.get_features())
            .flatten()
            .collect()
    }

    /// Every feature currently granted by the character's subclass(es).
    pub fn subclass_features(&self) -> Vec<&Feature> {
        self.classes.iter()
            .map(|specced_class| specced_class.get_subclass_features())
            .filter_map(|f| f)
            .flatten()
            .collect()
    }

    pub fn race_features(&self) -> Vec<&Feature> {
        self.race.traits
            .iter()
            .filter_map(|race_trait| race_trait.is_base())
            .collect()
    }

    pub fn subrace_features(&self) -> Vec<&Feature> {
        let subrace = match self.race.subraces.is_base() {
            Some(v) => v,
            _ => return vec![],
        };

        subrace.traits
            .iter()
            .filter_map(|race_trait| race_trait.is_base())
            .collect()
    }

    /// Every feature from all sources in effect on the character.
    pub fn total_features(&self) -> Vec<&Feature> {
        let bonus_features = self.bonus_features.iter();
        let item_features = self.item_features().into_iter();
        let class_features = self.class_features().into_iter();
        let subclass_features = self.subclass_features().into_iter();
        let race_features = self.race_features();
        let subrace_features = self.subrace_features();

        class_features
            .chain(item_features)
            .chain(subclass_features)
            .chain(race_features)
            .chain(subrace_features)
            .chain(bonus_features)
            .collect()
    }

    /// Gets the current ac of the character based off equipped items and features.
    pub fn ac(&self) -> isize {
        let stats = self.stats().modifiers();
        self.ac_with_modifiers(&stats)
    }


    /// Getting the ac, with inputted modifiers. This is intended to be a more efficienct version of
    /// [Character::ac] if you already have the stats on-hand.
    pub fn ac_with_modifiers(&self, stats: &Modifiers) -> isize {
        let equipped_items = self.equipped_items();


        let feature_effects = self.class_features()
            .into_iter()
            .chain(self.item_features())
            .map(|v| v.effects.iter())
            .flatten();

        let unarmored_defense = feature_effects.clone().find_map(|v| match v {
            FeatureEffect::UnarmoredDefense(base, stat1, stat2) => Some((base, stat1, stat2)),
            _ => None
        });

        // finds the first armor equipped. We're assuming there's only one.
        let armor = equipped_items.iter()
            .find_map(|i| {
                if let ItemType::Armor(armor)  = &i.0.item_type {
                    Some(armor)
                } else {
                    None
                }
            });

        let mut ac: isize = match (armor, unarmored_defense) {
            (Some(a), _) => a.get_ac(stats.dexterity),
            (None, Some((base, stat1, Some(stat2)))) => *base+stats.get_stat_type(stat1)+stats.get_stat_type(stat2),
            (None, Some((base, stat, None))) => *base+stats.get_stat_type(stat),
            (None, None) => 10 + stats.dexterity,
        };

        for effect in feature_effects {
            if let FeatureEffect::ACBonus(n) = effect {
                ac += n;
            }
        }
 
        // If there's a shield equipped, add 2, otherwise add 0
        let shield_bonus = equipped_items.iter()
            .find_map(|i| match &i.0.item_type {
                ItemType::Shield => Some(2),
                _ => None,
            })
            .unwrap_or(0);

        ac += shield_bonus;

       
        ac
    }


    /// This finds the hp of the character, assuming that you took the average value.
    pub fn max_hp(&self) -> usize {
        let level = self.level();
        let hit_die = self.classes.get(0).expect("Character should have a class").hit_die;
        let hit_die_avg = (((hit_die as f32)+1.0)/2.0).ceil() as usize;
        let con = self.stats().modifiers().constitution.max(1) as usize;

        let mut hp = hit_die + con + (level-1)*(hit_die_avg + con);

        // some features 
        for effect in self.race_features().iter().map(|v| v.effects.iter()).flatten() {
            if let FeatureEffect::LeveledHpIncrease = effect {
                hp += level;
            }
        }

        hp
    }
    

    /// level up a character by 1 in the given class.
    ///
    /// Returns the character's current level in that class, or nothing if you're trying to level
    /// up past 20.
    pub fn level_up(&mut self, class: &Class) -> Option<usize> {
        let class_name: &String = &class.name;
        // checking if the character is already specced into that class
        let current_class = self.classes.iter_mut().find(|specced_class| specced_class.class == *class_name );
        let current_class_ref = match current_class {
            Some(specced_class) => {
                specced_class.add_level(class);
                specced_class
            }
            None => {
                self.classes.push(SpeccedClass::from_class(class, 1));
                &self.classes[self.classes.len()-1] 
            }
        };
        let v = current_class_ref.level;
        self.hp = self.max_hp();
        Some(v)
    }

    /// Level up multiple times. Same as calling [Character::level_up] repeatedly.
    pub fn level_up_multiple(&mut self, class: &Class, times: usize) -> Option<usize> {
        if times <= 1 {
            return self.level_up(&class)
        }
        for _ in [0..times-1] {
            self.level_up(&class)?;
        }

        self.level_up(&class)
    }

    /// Level up until the total level (not class level) is equal to the given number.
    pub fn level_up_to_level(&mut self, class: &Class, level: usize) -> Option<usize> {
        if level > 20 {
            return None
        }
        let level_offset = (level as isize) - (self.level() as isize);
        if level_offset < 1 {
            return None
        }

        self.level_up_multiple(class, level_offset as usize) 
    }

}

/// A class as it's used for a character. This contains all the relevant information from a class
/// for a character at their level.
///
/// Note that this only stores features up to the character's current level. In order to get the
/// features of a new level, it needs to be grabbed from a [Class]. This is why
/// [Character::level_up] requires a class.
///
/// A lot of the details about a SpeccedClass's fields are the same as a [Class].
#[derive(Serialize, Deserialize)]
pub struct SpeccedClass {
    pub class: String,
    pub level: usize,
    pub current_class_features: Vec<Vec<PresentedOption<Feature>>>,
    pub items: Vec<PresentedOption<Vec<(ItemCategory, usize)>>>,
    pub subclass: PresentedOption<Subclass>,
    pub spellcasting: Option<Spellcasting>,
    pub hit_die: usize,
}

impl SpeccedClass {
    
    /// Get a specced class from a class, up to the specified level.
    fn from_class(class: &Class, level: usize) -> SpeccedClass {

        let subclass_option_vec = class.subclasses.iter().map(|v| PresentedOption::Base(v.clone())).collect();
        let subclass = PresentedOption::Choice(subclass_option_vec);

        SpeccedClass {
            class: class.name.clone(),
            level: level,
            current_class_features: class.features[0..level].iter().cloned().collect(),
            items: class.beginning_items.iter().cloned().collect(),
            subclass,
            spellcasting: class.spellcasting.clone(),
            hit_die: class.hit_die,
        }
    }

    /// Get the total class's features.
    fn get_features(&self) -> Vec<&Feature> {
        self.current_class_features.iter()
            .map(|level_features| chosen(level_features))
            .flatten()
            .collect()
    }

    fn get_subclass_features(&self) -> Option<Vec<&Feature>> {
        let subclass = self.subclass.is_base()?;
        let features: Vec<_>  = subclass
            .features[0..self.level as usize].iter()
            .map(|v| v.iter())
            .flatten()
            .filter_map(|v| v.is_base())
            .collect();
        Some(features)
    }

    /// Increments the character's level by 1 in that class.
    fn add_level(&mut self, class: &Class) {
        if self.level >= 20 {return}
        self.current_class_features.push(class.features[self.level].clone());
        self.level += 1;
    }
}

