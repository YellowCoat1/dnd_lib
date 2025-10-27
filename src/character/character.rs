use serde::{Serialize, Deserialize};

use crate::character::class::ItemCategory;
use crate::character::items::{is_proficient_with, ArmorCategory};

use super::background::Background;
use super::race::Race;
use super::stats::{EquipmentProficiencies, Modifiers, Saves, SkillModifiers, SkillProficiencies, SkillType, StatType, Stats, PROFICIENCY_BY_LEVEL};
use super::features::{AbilityScoreIncrease, Feature, FeatureEffect, PresentedOption};
use super::choice::chosen;
use super::items::{Item, ItemType, Weapon, WeaponAction, WeaponType};
use super::spells::{PactSlots, Spell, SpellAction, SpellCasterType, SpellSlots, Spellcasting, CASTER_SLOTS, PACT_CASTING_SLOTS};
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
///  and 1 level in fighter, they'll have a level 3 SpeccedClass wizard at `classes[0]` and a level 1
/// SpeccedClass fighter at `classes[1]`.
///
/// Most customization is done through choosing an option in a [PresentedOption].
///
/// The choice for subraces is available through the [Character::race] field, and then
/// [Race::subraces].
///
/// The choice for subclasses is available through `Character.classes[n].subclass`.
///
/// It's also important to select the items available through your starting class. Those are
/// available through `Character.classes[0].items`.
#[derive(Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    /// Individual classes that the character has specced into.
    pub classes: Vec<SpeccedClass>,
    pub race: Race,
    pub background: Background,
    /// Lists active spell slots. These can be spent.
    pub available_spell_slots: Option<SpellSlots>,
    /// Lists active pact magic slots. These can be spent. Seperate from regular spell slots.
    pub availible_pact_slots: Option<PactSlots>,
    base_stats: Stats,
    /// Extra features from etc sources that aren't listed otherwise. Feel free to append on any
    /// extra feature you want your character to have.
    pub bonus_features: Vec<Feature>,
    /// The first field is the item, second is count, and 3rd is if it's equipped or not.
    pub items: Vec<(Item, usize, bool)>, 
    equipment_proficiencies: EquipmentProficiencies,
    pub class_skill_proficiencies: Vec<PresentedOption<SkillType>>,
    class_saving_throw_proficiencies: Vec<StatType>,
    pub hp: usize,
    pub temp_hp: usize,
}

impl Character {


    /// Builds a level 1 character with base equipment from the class and background.
    pub fn new(name: String, class: &Class, background: &Background, race: &Race, base_stats: Stats) -> Character {
        
        let mut new_character = Character {
            name,
            classes: vec![SpeccedClass::from_class(class, 1)],
            background: background.clone(),
            items: vec![],
            equipment_proficiencies: class.equipment_proficiencies.clone(),
            race: race.clone(),
            base_stats,
            bonus_features: vec![],
            available_spell_slots: None,
            availible_pact_slots: None,
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

        // add class items
        new_character.add_class_items();

        new_character
    }

    fn add_item_list(&mut self, item_list: Vec<(Item, usize)>) {
        let mut new_item_list = item_list
            .into_iter()
            .map(|(i, c)| (i, c, false))
            .collect();
        self.items.append(&mut new_item_list)
    }

    /// Adds the class's items to the character, and removes those items from their [SpeccedClass] entry.
    /// ignores unchosen items.
    ///
    /// When a character is created, the items that are added are only the base, given items. Any
    /// choices (e.g. "A shortsword or any simple weapon") must be selected from the
    /// [SpeccedClass]'s list. This function will add every selected item, and remove them from
    /// that list to avoid double-adding.
    ///
    /// For selecting options, see [PresentedOption::choose_in_place]
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



    // ---------- STATS ----------

    /// Gets the character's total level by summing up all their class levels.
    pub fn level(&self) -> usize {
        self.classes.iter().map(|class| class.level).sum()
    }

    // in some cases level can be over 20.
    // This isn't officially supported, but it's nice to have a fallback
    // so it doesn't come crashing down.
    fn clamped_level(&self) -> usize {
        self.level().min(20)
    }

    /// Gets the character's proficiency bonus based on their level.
    pub fn proficiency_bonus(&self) -> isize {
        PROFICIENCY_BY_LEVEL[self.clamped_level()-1]
    }

    /// Returns the character's ability scores. 
    ///
    /// Note that this isn't modifiers, but rather base scores.
    /// 
    /// This takes the character's base stats, adds any increase from racial bonuses, and finally
    /// adds on any bonus from class ability score increases.
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

    /// Returns the proficiencies the character has in each saving throw.
    ///
    /// This is not saving throw modifiers. For that, see [Character::save_mods].
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

    /// Returns the modifiers the character has in each saving throw.
    pub fn save_mods(&self) -> Modifiers {
        let mut modifiers = self.saves().modifiers(&self.base_stats, self.proficiency_bonus());

        for effect in self.total_features().into_iter().map(|t| t.effects.iter()).flatten() {
            if let FeatureEffect::AddSaveModifier(t, m) = effect {
                *modifiers.get_stat_type_mut(&t) += m;
            }
        }

        modifiers
    }

    /// Returns the proficiencies and expertise the character has in each skill.
    ///
    /// This is not the modifiers for each skill. For that, see [Character::skill_modifiers]
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

    /// Returns the modifiers the character has in each skill.
    ///
    /// This calculates the base modifiers using the character's ability scores, finds the skills that the character are proficient in with [Character::skills], and adds the proficiency bonus to a skill if the character is proficient in it. (Proficiency is added twice if the character has proficiency and expertise)
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

    /// Returns a vector of references to every item marked as held.
    ///
    /// Just like for [Character::items], the first field in the tuple is the item, and the second
    /// field is the count of the item. (How many.)
    pub fn equipped_items(&self) -> Vec<(&Item, &usize)> {
        // gets the items that are selected as held
        self.items.iter().filter_map(|(i, c, h)| if *h {Some((i,c))} else {None}).collect()
    }

    // ---------- SPELLS ----------

    /// gets the spell save dc and spell attack modifier of the specified class.
    /// 
    /// The first field of the tuple is the spell save dc, and the second is the spell attack
    /// modifier.
    ///
    /// Returns a [None] if the character is not a spellcaster.
    ///
    /// ```ignore
    ///     use dnd_lib::get::{get_class, get_race, get_background};
    ///     use dnd_lib::character::{stats::Stats, Character};
    ///
    ///     // this is john. john has a base int score of 13, and john is a high elf. His int should be 14.
    ///     let stats = Stats::from(&[10, 10, 10, 13, 10, 10]);
    ///     let mut john = Character::new(String::from("john"), &wizard, &acolyte, &elf, stats);
    ///     john.race.subraces.choose_in_place(0);
    ///
    ///     // An int of 14 is a modifier of 2.
    ///     assert_eq!(john.stats().modifiers().intelligence, 2);
    ///
    ///     // john should have a spell save dc of 12, and a spell attack modifier of 4.
    ///     let (spell_save, spell_mod) = john.spellcasting_scores(0)
    ///         .expect("wizard character should be a spellcaster");
    ///     assert_eq!(spell_save, 12);
    ///     assert_eq!(spell_mod, 4);
    /// ```
    pub fn spellcasting_scores(&self, class_index: usize) -> Option<(isize, isize)> {
        let modifiers = self.stats().modifiers();
        self.spellcasting_scores_with_modifiers(class_index, &modifiers)
    }

    fn spellcasting_scores_with_modifiers(&self, class_index: usize, modifiers: &Modifiers) -> Option<(isize, isize)> {
        let spellcasting_ability = &self.classes.get(class_index)?
            .spellcasting.as_ref()?
            .0
            .spellcasting_ability;
        let spellcasting_mod = modifiers.get_stat_type(&spellcasting_ability).clone();

        let spell_save_dc = 8 + self.proficiency_bonus() + spellcasting_mod;
        let spell_attack_mod = self.proficiency_bonus() + spellcasting_mod;

        Some((spell_save_dc, spell_attack_mod))
    }

    /// Gets every spell availiable to the character.
    /// Returns a list of spells, and the indexes to the [SpeccedClass]es that they come from.
    ///
    /// If the character is not a spellcaster, this returns an empty [Vec].
    pub fn spells(&self) -> Vec<(&Spell, usize)> {
        self.classes.iter()
            .enumerate()
            .filter_map(|(n, v)| v.spellcasting.as_ref().map(|v| (&v.1, n)))
            .map(|(v, n)| {
                v.iter().zip(vec![n; v.len()].into_iter())
            })
            .flatten()
            .collect()
    }

    /// Gets total spell slots, the base spell slots the class has access to after a long rest.
    pub fn spell_slots(&self) -> Option<SpellSlots> {
        let caster_classes = self.classes.iter()
            .filter_map(|v| v.spellcasting.as_ref().map(|s| (s.0.spellcaster_type, v.level)));

        let slots_level: usize = caster_classes.map(|(caster_type, level)| {
            match caster_type {
                SpellCasterType::Full => level,
                SpellCasterType::Half => level / 2,
                SpellCasterType::Quarter => level / 3,
                SpellCasterType::Warlock => 0,
            }
        }).sum();

        if slots_level == 0 {
            return None;
        }

        Some(SpellSlots(CASTER_SLOTS[slots_level-1]))
    }

    /// Gets total pact magic slots, the base pact magic slots the class has access to after a
    /// short or long rest.
    ///
    /// Pact slots are treated differenty than spell slots. For regular spell slots, see
    /// [Character::spell_slots].
    pub fn pact_slots(&self) -> Option<PactSlots> {

        let (_, slots_level) = self.classes.iter()
            .filter_map(|v| v.spellcasting.as_ref().map(|s| (s.0.spellcaster_type, v.level)))
            .find(|(s, _)| matches!(s, SpellCasterType::Warlock))?;

        if slots_level == 0 {
            return None;
        }

        Some(PactSlots::from(PACT_CASTING_SLOTS[slots_level-1]))
    }

    /// Cast the spell, expending a spell slot.
    ///
    /// Returns false if the spell could not be cast. For example, if the character is not a
    /// spellcaster, or if there are no spell slots left of the specified type.
    ///
    /// the third argument is which spell list to pull from. Set this to None regularly. 
    ///
    /// Spells can be casted either with regular spell slots or warlock pact magic. Typically this doesn't come into effect, though
    /// if you're multiclassing you can choose which to use. If spell_list is none, then it uses
    /// whichever came first. Spell slots if you were another spellcaster first, pact magic if you
    /// were a warlock first. If it's a Some(false), it uses Spell slots, and if it's Some(True),
    /// it uses pact magic.
    ///
    /// Note that this only decrements the spell slot at the spell's level.
    pub fn cast<T: Castable>(&mut self, casted: T, spell_list: Option<bool>) -> bool {
        
        if let None = spell_list {
            let v = self.classes
                .iter()
                .find(|c| matches!(c.spellcasting, Some(_)))
                .map(|v| v.spellcasting.as_ref())
                .flatten()
                .map(|v| v.0.spellcaster_type);

            match v {
                None => false,
                Some(SpellCasterType::Warlock) => self.cast_with_pact(casted.level()),
                Some(_) => self.cast_with_slots(casted.level()),
            }
        } else {
            let b = spell_list.unwrap();
            if b {
                self.cast_with_pact(casted.level())
            } else {
                self.cast_with_slots(casted.level())
            }
        }
        
    }

    fn cast_with_slots(&mut self, level: usize) -> bool {
        let spell_slots = match &mut self.available_spell_slots {
            Some(s) => s,
            _ => return false,
        };

        let spell_slot = &mut spell_slots.0[level-1];
        
        if *spell_slot < 1 {
            return false
        }

        *spell_slot -= 1;
        true
    }
    
    fn cast_with_pact(&mut self, level: usize) -> bool {
        let spell_slot = match &mut self.availible_pact_slots {
            Some(s) => s,
            _ => return false,
        };

        if spell_slot.level < level {
            return false
        }

        if spell_slot.num < 1 {
            return false
        }

        spell_slot.num -= 1;
        true
    }

    // ----------- FEATURES ------------

    /// Every feature currently granted by any items the character has equipped.
    pub fn item_features(&self) -> Vec<&Feature> {
        self.items.iter()
            .filter_map(|v| if v.2 {Some(v.0.features.iter())} else {None})
            .flatten()
            .collect()
    }

    /// Every feature currently granted by the character's class, not including subclass features.
    pub fn class_features(&self) -> Vec<&Feature> {
        self.classes.iter()
            .flat_map(|specced_class| specced_class.get_features())
            .collect()
    }

    /// Every feature currently granted by the character's subclass(es).
    pub fn subclass_features(&self) -> Vec<&Feature> {
        self.classes.iter()
            .filter_map(|specced_class| specced_class.get_subclass_features())
            .flatten()
            .collect()
    }

    /// Every feature currently granted by the character's race.
    /// These are typically called traits, though in this library everything is under a [Feature].
    /// 
    /// This does not include subrace features.
    pub fn race_features(&self) -> Vec<&Feature> {
        self.race.traits
            .iter()
            .filter_map(|race_trait| race_trait.is_base())
            .collect()
    }

    /// Every feature granted by the subrace of the character's race.
    ///
    /// This does not include main race features.
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
    ///
    /// The features are collected from the character's race, subrace, class, subclass, items, and
    /// any extra bonus features the character may have as listed in [Character::bonus_features].
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

    /// Returns the current ac of the character based off features and equipped items.
    ///
    /// If the character has armor equipped, it uses the ac of that armor, plus any dex bonus that
    /// armor would get.
    /// If the character has no armor and an unarmored defense feature, the character has the ac of
    /// that unarmored defense.
    /// If the character has no armor an no unarmoed defense, as per D&D 5e rules the character's
    /// ac is 10 plus their dexterity modifier.
    ///
    /// Afterwards, bonuses from other features (and a shield, if any) are added.
    pub fn ac(&self) -> isize {
        let stats = self.stats().modifiers();
        self.ac_with_modifiers(&stats)
    }


    /// Getting the ac, with inputted modifiers. This is intended to be a more efficient version of
    /// [Character::ac] if you already have the stats on-hand.
    pub fn ac_with_modifiers(&self, stats: &Modifiers) -> isize {
        let equipped_items = self.equipped_items();


        let feature_effects = self.class_features()
            .into_iter()
            .chain(self.item_features())
            .flat_map(|v| v.effects.iter());

        // TODO: pick the unarmored defense that grants the most ac.
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
            (Some(a), _) => a.total_ac(stats.dexterity),
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

    /// Processes the character taking damage.
    ///
    /// Returns true if the characted dropped to zero hp, or false otherwise.
    pub fn damage(&mut self, damage: usize) -> bool {
        let o = self.hp.checked_sub(damage);
        match o {
            Some(s) => {
                self.hp = s;
                if self.hp != 0 {
                    false
                } else {
                    true
                }
            }
            None => {
                self.hp = 0;
                true
            },
        }
    }
    


    /// Attempts to increase the character's level by 1 in the given class.
    ///
    /// Returns the character's current level in that class, or [None] if the level would exceed
    /// 20.
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
        for _ in 0..times-1 {
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

    /// Returns the total equipment proficiencies for the character.
    ///
    /// This aggregates proficiencies from the class, possible race features, and
    ///  [Character::bonus_features]. 
    pub fn equipment_proficiencies(&self) -> EquipmentProficiencies {
        let feature_effects = self.race_features()
            .into_iter()
            .chain(self.subrace_features().into_iter())
            .chain(self.bonus_features.iter())
            .flat_map(|v| v.effects.iter());

        let mut equipment_proficiencies = self.equipment_proficiencies.clone();


        for feature_effect in feature_effects {
            match feature_effect {
                FeatureEffect::WeaponProficiency(w) => {
                    match w {
                        WeaponType::Simple => equipment_proficiencies.simple_weapons = true,
                        WeaponType::SimpleRanged => equipment_proficiencies.simple_weapons = true,
                        WeaponType::Martial => equipment_proficiencies.martial_weapons = true,
                        WeaponType::MartialRanged => equipment_proficiencies.martial_weapons = true,
                    }
                }
                FeatureEffect::ArmorProficiency(a) => {
                    match a {
                        ArmorCategory::Light => equipment_proficiencies.light_armor = true,
                        ArmorCategory::Medium => equipment_proficiencies.medium_armor = true,
                        ArmorCategory::Heavy => equipment_proficiencies.heavy_armor = true,
                    }
                }
                _ => (),
            }
        }

        equipment_proficiencies
    }

    /// Gets the attacks possible from all weapon sources with the character. The resulting
    /// [WeaponAction] has the final calculated attack modifier and damage roll needed to preform
    /// an attack.
    ///
    /// A weapon may represent multiple [WeaponAction]s. Light weapons have both a [WeaponAction] for
    /// their main attack, and a [WeaponAction] for their second attack, which will be marked as
    /// such and will not have the ability modifer added to the damage of the roll.
    ///
    /// Versitile weapons will also represent multiple [WeaponAction]s, one for one-handed and
    /// another for two-handed.
    ///
    /// If the weapon is versitile, it will use whichever is highest between strength and
    /// dexterity.
    pub fn weapon_actions(&self) -> Vec<WeaponAction> {
        let modifiers = self.stats().modifiers();
        let equipment_proficiencies = self.equipment_proficiencies();
        let proficiency_modifier = self.proficiency_bonus();
        self.equipped_items()
            .into_iter()
            .filter_map(|v| {
                match &v.0.item_type {
                    ItemType::Weapon(w) => Some((&v.0.name, w)),
                    _ => None,
                }
            })
            .flat_map(|(name, weapon)| {
                weapon_actions(name, weapon, &modifiers, &equipment_proficiencies, proficiency_modifier).into_iter()
            })
            .collect()
        //TODO: add unarmed strike
    }

    /// Gets the attacks possible from all spells prepared in any class. The resulting
    /// [SpellAction] has the final calculated attack modifer and damage roll needed to preform an
    ///  attack.
    ///
    /// Each spell that can deal damage will represent one [SpellAction] for each level it can be
    /// casted. For example, fireball will have a SpellAction for 3rd level, another for 4th, and
    /// another for 5th, and so on.
    ///
    /// Spells may also have multiple for each level. Chromatic orb, for example, can attack with
    /// one of many types, so it has many spell actions.
    pub fn spell_actions(&self) -> Vec<SpellAction> {
        //TODO: Limit spell levels up to the maximum the spellcaster can cast
        let modifiers = self.stats().modifiers();
        let mut char_spell_actions = vec![];
        for (index, class) in self.classes.iter().enumerate().filter(|v| v.1.spellcasting.is_some()) {
            let spellcasting_stuff = self.spellcasting_scores_with_modifiers(index, &modifiers).zip(class.spellcasting.as_ref());
            let ((_, attack_mod), (_, spells)) = match spellcasting_stuff {
                Some(s) => s,
                _ => continue,
            };

            let class_spell_actions =  spells.into_iter() 
                .filter_map(|s| spell_actions(s, attack_mod))
                .flat_map(|v| v.into_iter())
                .collect::<Vec<_>>();
            char_spell_actions.extend(class_spell_actions);

        }
        char_spell_actions
    }


}

fn spell_actions(spell: &Spell, spell_attack_mod: isize) -> Option<Vec<SpellAction>>  {
    Some(spell.damage.as_ref()?.into_iter()
        .enumerate()
        .flat_map(|(n, dv)| {
            dv.into_iter().map(move |d| (n + spell.level, d))
        })
        .map(|(spell_level, damage)| {
            SpellAction {
                spell_level: spell_level as isize,
                name: spell.name.clone(), 
                spell_attack_mod, 
                damage_roll: damage.clone(), 
            }
        })
        .collect())
}


fn weapon_actions(name: &String, w: &Weapon, m: &Modifiers, p: &EquipmentProficiencies, proficiency_mod: isize) -> Vec<WeaponAction> {
    let finesse = w.properties.finesse;
    let versatile = w.properties.versatile;
    let two_handed = w.properties.two_handed;
    let light = w.properties.light;

    let modifier = if finesse && m.dexterity > m.strength {
        m.dexterity
    } else {
        m.strength
    };

    let proficient = is_proficient_with(&w.weapon_type, p) || p.other.contains(name);

    let bonus = if proficient {proficiency_mod} else {0};

    let attack_bonus = modifier+bonus+(w.attack_roll_bonus as isize);
    let damage_roll = w.damage;
    let damage_roll_bonus = modifier + bonus;

    let base_attack = WeaponAction {
        name: name.clone(),
        attack_bonus,
        damage_roll,
        damage_roll_bonus,
        two_handed,
        second_attack: false,
    };

    let mut attacks = vec![base_attack];

    // add second attack
    if light {
        attacks.push(WeaponAction {
            name: name.clone(),
            attack_bonus:  attack_bonus,
            damage_roll,
            damage_roll_bonus: modifier,
            two_handed: false,
            second_attack: true,
        });
    }

    // add possible two-handed attack
    if let Some(d) = versatile {
        attacks.push(WeaponAction {
            name: name.clone(),
            attack_bonus,
            damage_roll: d,
            damage_roll_bonus,
            two_handed: true,
            second_attack: false,
        });
    }

    attacks
}

/// A class as it's used for a character. This contains all the relevant information from a class
/// for a character at their level.
///
/// A lot of the details about a SpeccedClass's fields are the same as a [Class].
#[derive(Clone, Serialize, Deserialize)]
pub struct SpeccedClass {
    /// The name of the class the [SpeccedClass] is from.
    pub class: String,
    /// The level the character has in the class.
    pub level: usize,
    /// List of feature options for each level the character has in this class.
    /// 
    /// Note that this only stores features up to the character's current level. In order to get the
    /// features of a new level, it needs to be grabbed from a [Class]. This is why
    /// [Character::level_up] requires a class.
    pub current_class_features: Vec<Vec<PresentedOption<Feature>>>,
    /// Items given by the class at level 1. This is only relevant for the first class.
    ///
    /// This field is a list of options to select, of which can each be a list of items.
    ///
    /// The first field in the tuple is an [ItemCategory], which needs to be a [ItemCategory::Item]
    /// in order to be "Chosen". The second field in the tuple is the count of the item, or how
    /// many there are.
    pub items: Vec<PresentedOption<Vec<(ItemCategory, usize)>>>,
    /// The subclasses that can be chosen. The character must choose a subclass before gaining any
    /// features from it.
    pub subclass: PresentedOption<Subclass>,
    /// The class's spellcasting. Each class in D&D has it's own prepared spell list and spell
    /// DC/attack modifier, so they're listed in the [SpeccedClass] instead of a field in the
    /// [Character]. Spell slots are dependent on the character rather than the indiviudal class,
    /// so they're listed as a [Character] field in [Character::available_spell_slots].
    ///
    /// The [Option] is [None] if the class isn't a spellcaster. The first field is the
    /// [Spellcasting], which contains information about the spellcasting ability and spell list,
    /// and the second field has the prepared or known spells.
    pub spellcasting: Option<(Spellcasting, Vec<Spell>)>,
    /// The class's hit die. This is the number of faces, so an 8 is a 1d8.
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
            spellcasting: class.spellcasting.clone().map(|v| (v, vec![])),
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

pub trait Castable {
    fn level(&self) -> usize;
}

impl Castable for Spell {
    fn level(&self) -> usize {
        self.level
    }
}

impl Castable for SpellAction {
    fn level(&self) -> usize {
        self.spell_level.try_into().expect("spell level was negative")
    }
}

