#![cfg_attr(doc, feature(doc_auto_cfg))]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::character::class::ItemCategory;
use crate::character::items::{is_proficient_with, ArmorCategory};
use crate::character::spells::SpellCastingPreperation;

use super::background::Background;
use super::choice::chosen;
use super::class::{Class, TrackedField, Subclass, UNARMORED_MOVEMENT};
use super::features::{
    AbilityScoreIncrease, ComputedCustomAction, CustomAction, Feature, FeatureEffect,
    PresentedOption,
};
use super::items::{DamageRoll, DamageType, Item, ItemType, Weapon, WeaponAction, WeaponType};
use super::race::Race;
use super::spells::{
    PactSlots, Spell, SpellAction, SpellCasterType, SpellSlots, Spellcasting, CASTER_SLOTS,
    PACT_CASTING_SLOTS,
};
use super::stats::{
    EquipmentProficiencies, Modifiers, Saves, SkillModifiers, SkillProficiencies, SkillType,
    Speeds, StatType, Stats, PROFICIENCY_BY_LEVEL,
};
use super::{CharacterDescriptors, CharacterStory};

/// A struct to represent a Dungeons and Dragons character.
///
/// In order to build a character, you need a [Class], a [Background], and a [Race].
/// To get one of these, you can either get them from the api using the [get](super::super::get) module,
/// or build them from scratch in the case of homebrew.
///
/// ```
/// # #[cfg(feature = "dnd5eapi")] {
/// #[tokio::main]
/// async fn main() {
///     use dnd_lib::prelude::*;
///
///     let provider = Dnd5eapigetter::new();
///     let fighter = provider.get_class("fighter").await.unwrap();
///     let human = provider.get_race("human").await.unwrap();
///     let acolyte = provider.get_background("acolyte").await.unwrap();
///
///     let john = CharacterBuilder::new("john")
///         .class(&fighter)
///         .background(&acolyte)
///         .race(&human)
///         .stats(Stats::default())
///         .build().unwrap();
/// }
/// # }
/// ```
///
/// Each class the character uses is represented by a [SpeccedClass] instance. If a character has 3 levels in wizard
///  and 1 level in fighter, they'll have a level 3 SpeccedClass wizard at `classes[0]` and a level 1
/// SpeccedClass fighter at `classes[1]`.
///
/// Some methods have a field of "class_index". This is just an index of the `Character.classes` vec, at the class the method wants to target.
///
/// ## Customization
///
/// Here, we talk about customization as in the choices you make for the character. Things like an
/// ability score increase, a possible subrace or subclass, or a language option.
///
/// Most customization is done through choosing an option in a [PresentedOption], which is often
/// done by using
/// [PresentedOption::choose_in_place].
///
/// There's also occasionally a regular [Option]. A `None` value would mean an unfilled
/// choice, which you can fill into a `Some(T)` to "choose" it's value. This is usually used for
/// open ended choices, such as langauges.
///
/// Languages can be really anything, so instead of a concrete list of choices, you can just fill
/// it with any string.
///
/// The following can be found manually, but is provided as a convinience.
///
/// #### Major features
/// The choice for subraces is available through the [Character::race] field, and then
/// [Race::subraces].
///
/// If a race or subrace can choose an ability score, (like how variant human can chose any two to
/// add +1), then that can be found from the `ability_bonuses` field, at `ability_bonuses[n].0`
/// where n is the index of the ability score increase. An unchosen ability bonus is represented by
/// a `None` value, and filling it with a `Some(StatType)` will "choose" it.
///
/// The choice for subclasses is available through `Character.classes[n].subclass`.
///
/// Beginning item choices are available through `Character.classes[0].items`.
///
/// #### Spells
///
/// Spells are per-class. You can find the prepared/known spells of a class in the character at
/// `Character.classes[n].spellcasting.unwrap().1`. You can push or change spells here. This isn't
/// logically bounded; There's nothing in the crate stopping you from pushing 100 spells.
///
/// The amount of spells the character can prepare in that class can be found by
/// `Character.num_spells(n)` where n is the index of the class.
///
/// #### Minor features
///
/// Various strings for describing the character's story are availiable at [Character::story].
/// Similarly, strings about the character's personality and appearance are at
/// [Character::descriptors].
///
/// Character alignment is also available at [Character::descriptors].
///

#[derive(Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    /// Individual classes that the character has specced into.
    pub classes: Vec<SpeccedClass>,
    pub race: Race,
    /// Lists active spell slots. These can be spent.
    pub available_spell_slots: Option<SpellSlots>,
    /// Lists active pact magic slots. These can be spent. Seperate from regular spell slots.
    pub available_pact_slots: Option<PactSlots>,
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

    /// The name of the character's background
    pub background: String,
    /// The proficiencies granted by the background
    pub background_proficiencies: Vec<PresentedOption<SkillType>>,

    /// The character can choose 2 personality traits.
    pub personality_traits: (PresentedOption<String>, PresentedOption<String>),
    pub ideal: PresentedOption<String>,
    pub bond: PresentedOption<String>,
    pub flaw: PresentedOption<String>,

    /// Etc field for describing the character's story, enemies, personality, etc
    pub story: CharacterStory,
    /// Etc field for describing the character's personal traits (eye color, height, alignment)
    pub descriptors: CharacterDescriptors,

    /// heroic inspiration
    pub inspiration: bool,

    /// hit dice. This is the amount spent. The total amount is equal to the level, or
    /// [Character::level()]
    pub spent_hit_dice: usize,
}

impl Character {
    /// Builds a level 1 character with base equipment from the class and background.
    ///
    /// Typically using [CharacterBuilder](crate::character::CharacterBuilder) is preferred over this.
    pub fn new(
        name: String,
        class: &Class,
        background: &Background,
        race: &Race,
        base_stats: Stats,
    ) -> Character {
        let mut new_character = Character {
            name,
            classes: vec![SpeccedClass::from_class(class, 1)],
            items: vec![],
            equipment_proficiencies: class.equipment_proficiencies.clone(),
            race: race.clone(),
            base_stats,
            bonus_features: vec![],
            available_spell_slots: None,
            available_pact_slots: None,
            class_skill_proficiencies: vec![
                class.skill_proficiency_choices.1.clone();
                class.skill_proficiency_choices.0
            ],
            class_saving_throw_proficiencies: class.saving_throw_proficiencies.clone(),

            background: background.name.clone(),
            background_proficiencies: background.proficiencies.clone(),
            personality_traits: (
                PresentedOption::Choice(background.personality_traits.clone()),
                PresentedOption::Choice(background.personality_traits.clone()),
            ),
            ideal: PresentedOption::Choice(background.ideals.clone()),
            bond: PresentedOption::Choice(background.bonds.clone()),
            flaw: PresentedOption::Choice(background.flaws.clone()),

            hp: 1,
            temp_hp: 0,
            story: CharacterStory::default(),
            descriptors: CharacterDescriptors::default(),
            inspiration: false,
            spent_hit_dice: 0,
        };

        // add background items
        new_character.add_item_list(background.equipment.clone());

        // set hp
        new_character.hp = new_character.max_hp();

        // add class items
        new_character.add_class_items();

        // set the correct size
        new_character.descriptors.size = new_character.race.size;

        new_character.available_spell_slots = new_character.spell_slots();
        new_character.available_pact_slots = new_character.pact_slots();

        new_character
    }

    fn add_item_list(&mut self, item_list: Vec<(Item, usize)>) {
        let mut new_item_list = item_list.into_iter().map(|(i, c)| (i, c, false)).collect();
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
    ///
    /// ```
    /// # #[cfg(feature = "dnd5eapi")] {
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use dnd_lib::prelude::*;
    /// # let provider = Dnd5eapigetter::new();
    /// # let fighter = provider.get_class("fighter").await.unwrap();
    /// # let human = provider.get_race("human").await.unwrap();
    /// # let acolyte = provider.get_background("acolyte").await.unwrap();
    /// # let mut john = CharacterBuilder::new("john")
    /// #   .class(&fighter)
    /// #   .background(&acolyte)
    /// #   .race(&human)
    /// #   .stats(Stats::default())
    /// #   .build().unwrap();
    /// // select the first option for the first item choice
    /// john.classes[0].items[0].choose_in_place(0);
    /// // adds the selected item to john's inventory
    /// john.add_class_items();
    /// // now, john.classes[0].items[0] is what items[1] was before, since items[0] was removed.
    /// // john.items now contains the item that was selected.
    /// # }
    /// # }
    /// ```
    pub fn add_class_items(&mut self) {
        let mut items: Vec<(Item, usize)> = vec![];
        self.classes[0].items = self.classes[0]
            .items
            .iter()
            .filter(|o| {
                if let PresentedOption::Base(v) = o {
                    items.extend_from_slice(&Character::selected_items(v));
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        self.add_item_list(items);
    }

    fn selected_items(items: &[(ItemCategory, usize)]) -> Vec<(Item, usize)> {
        items
            .iter()
            .filter_map(|v| {
                if let ItemCategory::Item(i) = &v.0 {
                    Some((i.clone(), v.1))
                } else {
                    None
                }
            })
            .collect()
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
        PROFICIENCY_BY_LEVEL[self.clamped_level() - 1]
    }

    /// Returns the character's ability scores.
    ///
    /// Note that this isn't modifiers, but rather base scores.
    ///
    /// This takes the character's base stats, adds any increase from racial bonuses, and finally
    /// adds on any bonus from class ability score increases.
    pub fn stats(&self) -> Stats {
        let mut new_stats = self.base_stats;

        for (race_stat_change, amount) in self.race.ability_bonuses.iter() {
            if let Some(s) = race_stat_change {
                *new_stats.get_stat_type_mut(s) += amount;
            }
        }

        if let PresentedOption::Base(ref chosen_subrace) = self.race.subraces {
            for (subrace_stat, amount) in chosen_subrace.ability_bonuses.iter() {
                if let Some(s) = subrace_stat {
                    *new_stats.get_stat_type_mut(s) += amount;
                }
            }
        }

        // We also want to get features from the class.
        // This accounts for things like ability score increases.
        // We're also adding bonus features just in case.
        let feature_effects = self
            .class_features()
            .into_iter()
            .chain(self.bonus_features.iter())
            .flat_map(|v| &v.effects);

        // ability score increase macro
        macro_rules! apply_ability_score_increase {
            ($s1: expr) => {
                if let Some(s) = $s1 {
                    // if the ability score is under 20, we add 1.
                    // we don't want to go over 20 through this.
                    if *new_stats.get_stat_type(s) < 20 {
                        *new_stats.get_stat_type_mut(s) += 1;
                    }
                }
            };
        }

        for feature in feature_effects {
            match feature {
                FeatureEffect::AddModifier(stat, amount) => {
                    let stat = new_stats.get_stat_type_mut(stat);
                    // add it, while making sure it's bounded by 20
                    *stat = (*stat + amount).min(20);
                }
                FeatureEffect::AbilityScoreIncrease(AbilityScoreIncrease::StatIncrease(s1, s2)) => {
                    apply_ability_score_increase!(s1);
                    apply_ability_score_increase!(s2);
                }
                _ => (),
            }
        }

        // used for features that can increase the total over 20
        let extra_modifiers = self
            .bonus_features
            .iter()
            .chain(self.item_features())
            .flat_map(|v| &v.effects)
            .filter_map(|v| match v {
                FeatureEffect::AddModifierUncapped(stat, amount) => Some((stat, amount)),
                _ => None,
            });

        for (stat, amount) in extra_modifiers {
            let stat = new_stats.get_stat_type_mut(stat);
            *stat += amount;
        }

        new_stats
    }

    /// Returns the proficiencies the character has in each saving throw.
    ///
    /// This is not saving throw modifiers. For that, see [Character::save_mods].
    /// 
    /// ```
    /// # #[cfg(feature = "dnd5eapi")] {
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use dnd_lib::prelude::*;
    /// # use dnd_lib::character::stats::StatType;
    /// # let provider = Dnd5eapigetter::new();
    /// # let fighter = provider.get_class("fighter").await.unwrap();
    /// # let human = provider.get_race("human").await.unwrap();
    /// # let acolyte = provider.get_background("acolyte").await.unwrap();
    /// let john = CharacterBuilder::new("john")
    ///   .class(&fighter)
    ///   .background(&acolyte)
    ///   .race(&human)
    ///   .stats(Stats::default())
    ///   .build().unwrap();
    /// // get the saving throw proficiencies
    /// let saves = john.saves();
    /// // fighter gives proficiency in str and con saves
    /// assert!(saves.is_proficient(StatType::Strength));
    /// assert!(saves.is_proficient(StatType::Constitution)); 
    /// # }
    /// # }
    pub fn saves(&self) -> Saves {
        let mut base = Saves::default();

        for save in self.class_saving_throw_proficiencies.iter() {
            base.add_proficiency_from_type(*save);
        }

        for effect in self
            .total_features()
            .into_iter()
            .flat_map(|t| t.effects.iter())
        {
            if let FeatureEffect::AddSaveProficiency(s) = effect {
                base.add_proficiency_from_type(*s);
            }
        }

        base
    }

    /// Returns the modifiers the character has in each saving throw.
    pub fn save_mods(&self) -> Modifiers {
        let mut modifiers = self
            .saves()
            .modifiers(&self.stats(), self.proficiency_bonus());

        for effect in self
            .total_features()
            .into_iter()
            .flat_map(|t| t.effects.iter())
        {
            if let FeatureEffect::AddSaveModifier(t, m) = effect {
                *modifiers.get_stat_type_mut(t) += m;
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
        let background_skills: Vec<&SkillType> = chosen(&self.background_proficiencies);

        for skill in chosen_class_skills.iter().chain(background_skills.iter()) {
            let cloned_skill = *(*skill);
            base.add_proficiency_from_type(cloned_skill);
        }

        for effect in self.total_features().iter().flat_map(|t| t.effects.iter()) {
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
                _ => (),
            }
        }

        base
    }

    /// Returns the modifiers the character has in each skill.
    ///
    /// This calculates the base modifiers using the character's ability scores, finds the skills that the character are proficient in with [Character::skills], and adds the proficiency bonus to a skill if the character is proficient in it. (Proficiency is added twice if the character has proficiency and expertise)
    pub fn skill_modifiers(&self) -> SkillModifiers {
        let mut modifiers = self
            .skills()
            .modifiers(&self.stats(), self.proficiency_bonus());

        for effect in self.total_features().iter().flat_map(|t| t.effects.iter()) {
            if let FeatureEffect::AddSkillModifier(t, n) = effect {
                *modifiers.get_skill_type_mut(*t) += *n
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
        self.items
            .iter()
            .filter_map(|(i, c, h)| if *h { Some((i, c)) } else { None })
            .collect()
    }

    // ---------- SPELLS ----------

    /// gets the spell save dc and spell attack modifier of the specified class.
    ///
    /// The first field of the tuple is the spell save dc, and the second is the spell attack
    /// modifier.
    ///
    /// Returns a [None] if the character is not a spellcaster.
    ///
    /// ```rust
    ///     # #[cfg(feature = "dnd5eapi")] {
    ///     # #[tokio::main]
    ///     # async fn main() {
    ///     # use dnd_lib::prelude::*;
    ///     # let provider = Dnd5eapigetter::new();
    ///     # let wizard = provider.get_class("wizard").await.unwrap();
    ///     # let acolyte = provider.get_background("acolyte").await.unwrap();
    ///     # let elf = provider.get_race("elf").await.unwrap();
    ///     // John has a base int score of 13, and john is a high elf.
    ///     // His int should be 14.
    ///     let stats = Stats::from(&[10, 10, 10, 13, 10, 10]);
    ///     let mut john = CharacterBuilder::new("john")
    ///         .class(&wizard)
    ///         .background(&acolyte)
    ///         .race(&elf)
    ///         .stats(stats)
    ///         .build().unwrap();
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
    ///     # }
    ///     # }
    /// ```
    pub fn spellcasting_scores(&self, class_index: usize) -> Option<(isize, isize)> {
        let modifiers = self.stats().modifiers();
        self.spellcasting_scores_with_modifiers(class_index, &modifiers)
    }

    fn spellcasting_scores_with_modifiers(
        &self,
        class_index: usize,
        modifiers: &Modifiers,
    ) -> Option<(isize, isize)> {
        let spellcasting_ability = &self
            .classes
            .get(class_index)?
            .spellcasting
            .as_ref()?
            .0
            .spellcasting_ability;
        let spellcasting_mod = *modifiers.get_stat_type(spellcasting_ability);

        let spell_save_dc = 8 + self.proficiency_bonus() + spellcasting_mod;
        let spell_attack_mod = self.proficiency_bonus() + spellcasting_mod;

        Some((spell_save_dc, spell_attack_mod))
    }

    /// Gets every spell availiable to the character.
    /// Returns a list of spells, and the indexes to the [SpeccedClass]es that they come from.
    ///
    /// If the character is not a spellcaster, this returns an empty [Vec].
    pub fn spells(&self) -> Vec<(&Spell, usize)> {
        self.classes
            .iter()
            .enumerate()
            .filter_map(|(n, v)| v.spellcasting.as_ref().map(|v| (&v.1, n)))
            .flat_map(|(v, n)| v.iter().zip(vec![n; v.len()]))
            .collect()
    }

    /// Gets total spell slots, the base spell slots the class has access to after a long rest.
    pub fn spell_slots(&self) -> Option<SpellSlots> {
        let caster_classes = self.classes.iter().filter_map(|v| {
            v.spellcasting
                .as_ref()
                .map(|s| (s.0.spellcaster_type, v.level))
        });

        let slots_level: usize = caster_classes
            .map(|(caster_type, level)| match caster_type {
                SpellCasterType::Full => level,
                SpellCasterType::Half => level / 2,
                SpellCasterType::Quarter => level / 3,
                SpellCasterType::Warlock => 0,
            })
            .sum();

        if slots_level == 0 {
            return None;
        }

        Some(SpellSlots(CASTER_SLOTS[slots_level - 1]))
    }

    /// Gets total pact magic slots, the base pact magic slots the class has access to after a
    /// short or long rest.
    ///
    /// Pact slots are treated differenty than spell slots. For regular spell slots, see
    /// [Character::spell_slots].
    pub fn pact_slots(&self) -> Option<PactSlots> {
        let (_, slots_level) = self
            .classes
            .iter()
            .filter_map(|v| {
                v.spellcasting
                    .as_ref()
                    .map(|s| (s.0.spellcaster_type, v.level))
            })
            .find(|(s, _)| matches!(s, SpellCasterType::Warlock))?;

        if slots_level == 0 {
            return None;
        }

        Some(PactSlots::from(PACT_CASTING_SLOTS[slots_level - 1]))
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
    pub fn cast<T: Castable>(&mut self, casted: &T, spell_list: Option<bool>) -> bool {
        if spell_list.is_none() {
            let v = self
                .classes
                .iter()
                .find(|c| c.spellcasting.is_some())
                .and_then(|v| v.spellcasting.as_ref())
                .map(|v| v.0.spellcaster_type);

            match v {
                None => false,
                Some(SpellCasterType::Warlock) => self.cast_with_pact(casted.level()),
                Some(_) => self.cast_with_slots(casted.level()),
            }
        } else if let Some(b) = spell_list {
            if b {
                self.cast_with_pact(casted.level())
            } else {
                self.cast_with_slots(casted.level())
            }
        } else {
            false
        }
    }

    fn cast_with_slots(&mut self, level: usize) -> bool {
        if level == 0 {
            return true;
        }
        let spell_slots = match &mut self.available_spell_slots {
            Some(s) => s,
            _ => return false,
        };

        let spell_slot = &mut spell_slots.0[level - 1];

        if *spell_slot < 1 {
            return false;
        }

        *spell_slot -= 1;
        true
    }

    fn cast_with_pact(&mut self, level: usize) -> bool {
        if level == 0 {
            return true;
        }
        let spell_slot = match &mut self.available_pact_slots {
            Some(s) => s,
            _ => return false,
        };

        if spell_slot.level < level {
            return false;
        }

        if spell_slot.num < 1 {
            return false;
        }

        spell_slot.num -= 1;
        true
    }

    // ----------- FEATURES ------------

    /// Every feature currently granted by any items the character has equipped.
    pub fn item_features(&self) -> Vec<&Feature> {
        self.items
            .iter()
            .filter_map(|v| if v.2 { Some(v.0.features.iter()) } else { None })
            .flatten()
            .collect()
    }

    /// Every feature currently granted by the character's class, not including subclass features.
    pub fn class_features(&self) -> Vec<&Feature> {
        self.classes
            .iter()
            .flat_map(|specced_class| specced_class.get_features())
            .collect()
    }

    /// Every feature currently granted by the character's subclass(es).
    pub fn subclass_features(&self) -> Vec<&Feature> {
        self.classes
            .iter()
            .filter_map(|specced_class| specced_class.get_subclass_features())
            .flatten()
            .collect()
    }

    /// Every feature currently granted by the character's race.
    /// These are typically called traits, though in this library everything is under a [Feature].
    ///
    /// This does not include subrace features.
    pub fn race_features(&self) -> Vec<&Feature> {
        self.race
            .traits
            .iter()
            .filter_map(|race_trait| race_trait.as_base())
            .collect()
    }

    /// Every feature granted by the subrace of the character's race.
    ///
    /// This does not include main race features.
    pub fn subrace_features(&self) -> Vec<&Feature> {
        let subrace = match self.race.subraces.as_base() {
            Some(v) => v,
            _ => return vec![],
        };

        subrace
            .traits
            .iter()
            .filter_map(|race_trait| race_trait.as_base())
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

        let feature_effects = self
            .class_features()
            .into_iter()
            .chain(self.item_features())
            .flat_map(|v| v.effects.iter());

        // TODO: pick the unarmored defense that grants the most ac.
        let unarmored_defense = feature_effects.clone().find_map(|v| match v {
            FeatureEffect::UnarmoredDefense(base, stat1, stat2) => Some((base, stat1, stat2)),
            _ => None,
        });

        // finds the first armor equipped. We're assuming there's only one.
        let armor = equipped_items.iter().find_map(|i| {
            if let ItemType::Armor(armor) = &i.0.item_type {
                Some(armor)
            } else {
                None
            }
        });

        let mut ac: isize = match (armor, unarmored_defense) {
            (Some(a), _) => a.total_ac(stats.dexterity),
            (None, Some((base, stat1, Some(stat2)))) => {
                *base + stats.get_stat_type(stat1) + stats.get_stat_type(stat2)
            }
            (None, Some((base, stat, None))) => *base + stats.get_stat_type(stat),
            (None, None) => 10 + stats.dexterity,
        };

        for effect in feature_effects {
            if let FeatureEffect::ACBonus(n) = effect {
                ac += n;
            }
        }

        // If there's a shield equipped, add 2, otherwise add 0
        let shield_bonus = equipped_items
            .iter()
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
        let hit_die = self
            .classes
            .first()
            .expect("Character should have a class")
            .hit_die;
        let hit_die_avg = (((hit_die as f32) + 1.0) / 2.0).ceil() as usize;
        let con = self.stats().modifiers().constitution.max(1) as usize;

        let mut hp = hit_die + con + (level - 1) * (hit_die_avg + con);

        // some features
        for effect in self.race_features().iter().flat_map(|v| v.effects.iter()) {
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
                self.hp == 0
            }
            None => {
                self.hp = 0;
                true
            }
        }
    }

    /// Gets the walking speed of the character
    pub fn speed(&self) -> usize {
        let speed_bonus: usize = self
            .race_features()
            .into_iter()
            .chain(self.class_features())
            .chain(self.bonus_features.iter())
            .flat_map(|v| v.effects.iter())
            .map(|effect| match effect {
                FeatureEffect::SpeedBonus(n) => *n,
                FeatureEffect::UnarmoredMovement => self.unarmored_movement(),
                _ => 0,
            })
            .sum();

        self.race.speed + speed_bonus
    }

    /// Returns the different speeds of the character, e.g. flying and climbing.
    ///
    /// Most of these speeds, besides walking, is rare for a character to have.
    pub fn speeds(&self) -> Speeds {
        let mut speeds = Speeds {
            walking: Some(self.speed()),
            flying: None,
            hovering: None,
            burrowing: None,
            climbing: None,
            swimming: None,
        };

        let effects = self
            .race_features()
            .into_iter()
            .chain(self.bonus_features.iter())
            .flat_map(|v| v.effects.iter());

        macro_rules! add_speed {
            ($speed_field: expr, $applying: expr) => {
                $speed_field = $speed_field.map(|s| s.max($applying))
            };
        }

        for effect in effects {
            match effect {
                FeatureEffect::FlyingSpeed(s) => add_speed!(speeds.flying, *s),
                FeatureEffect::HoveringSpeed(s) => add_speed!(speeds.hovering, *s),
                FeatureEffect::BurrowingSpeed(s) => add_speed!(speeds.burrowing, *s),
                FeatureEffect::ClimbingSpeed(s) => add_speed!(speeds.climbing, *s),
                FeatureEffect::SwimmingSpeed(s) => add_speed!(speeds.swimming, *s),
                _ => panic!(),
            };
        }

        speeds
    }

    fn unarmored_movement(&self) -> usize {
        let level = self
            .classes
            .iter()
            .find(|v| v.class == "Monk")
            .expect("Unarmored defense without monk levels. Did you add it manually?")
            .level;
        UNARMORED_MOVEMENT[level - 1]
    }

    /// Attempts to increase the character's level by 1 in the given class.
    ///
    /// Returns the character's current level in that class, or [None] if the level would exceed
    /// 20.
    pub fn level_up(&mut self, class: &Class) -> Option<usize> {
        // get the spell slots before leveling up. This is usefule for recalculating spell slots.
        let spell_slots_before = self.spell_slots();
        let pact_slots_before = self.pact_slots();
        let stats = self.stats();

        // actually level up
        let v = self.level_up_inner(class, &stats)?;

        self.hp = self.max_hp();

        // if the class has spellcasting, the new spell slots need to be calculated.
        self.level_up_spellslots(spell_slots_before);
        self.level_up_warlock_pactslots(pact_slots_before);

        Some(v)
    }

    fn level_up_etc_specific(&mut self, class: &Class) {
        for specced_class in self.classes.iter_mut() {
            let level_before = specced_class.level - 1;
            let level_after = specced_class.level;
            dbg!((&level_before, &level_after));
            for etc_field in specced_class.tracked_fields.iter_mut() {
                let max_before = self::get_etc_field_max(
                    &etc_field.0,
                    &class.class_specific_leveled,
                    level_before,
                );
                let max_after = self::get_etc_field_max(
                    &etc_field.0,
                    &class.class_specific_leveled,
                    level_after,
                );
                match (max_before, max_after) {
                    (Some(b), Some(a)) => etc_field.1 += a.saturating_sub(b),
                    _ => continue,
                }
            }
        }
    }
    /// Inner function for leveling up without recalculating etc info.
    fn level_up_inner(&mut self, class: &Class, stats: &Stats) -> Option<usize> {
        let class_name: &String = &class.name;
        // checking if the character is already specced into that class
        let current_class = self
            .classes
            .iter_mut()
            .find(|specced_class| specced_class.class == *class_name);

        match current_class {
            Some(specced_class) => {
                specced_class.add_level(class);
                // add things like extra rages or ki points
                let v = Some(specced_class.level);
                self.level_up_etc_specific(class);
                v
            }
            None => self.level_multiclass(class, stats),
        }
    }

    // When a character tries to multiclass into a new class.
    // Returns Some(1) if succeeds, or None if the character doesn't have the correct requirements.
    fn level_multiclass(&mut self, class: &Class, stats: &Stats) -> Option<usize> {
        let or = class.multiclassing_prerequisites_or;

        let mut able_to_multiclass = !or;

        for (stat, min_value) in class.multiclassing_prerequisites.iter() {
            let condition = *stats.get_stat_type(stat) >= *min_value as isize;
            dbg!((
                &stat,
                &min_value,
                &able_to_multiclass,
                stats.get_stat_type(stat)
            ));
            match or {
                false => able_to_multiclass = able_to_multiclass && condition,
                true => able_to_multiclass = able_to_multiclass || condition,
            }
        }

        if able_to_multiclass {
            self.classes.push(SpeccedClass::from_class(class, 1));
            self.equipment_proficiencies += class.multiclassing_proficiency_gain.clone();
            Some(1)
        } else {
            None
        }
    }

    // when leveling up, spell new spell slots are added, but existing spent spell slots remain spent.
    fn level_up_spellslots(&mut self, slots_before: Option<SpellSlots>) {
        let slots_after = self.spell_slots();
        match (slots_before, slots_after) {
            (Some(before), Some(after)) => {
                let new_slots = after
                    .0
                    .iter()
                    .zip(before.0.iter())
                    .map(|(a, b)| a.saturating_sub(*b))
                    .collect::<Vec<usize>>();

                if let Some(current_slots) = &mut self.available_spell_slots {
                    for (i, v) in new_slots.iter().enumerate() {
                        current_slots.0[i] += *v;
                    }
                } else {
                    self.available_spell_slots = Some(SpellSlots(new_slots.try_into().unwrap()));
                }
            }
            (None, Some(after)) => {
                self.available_spell_slots = Some(after);
            }
            _ => {}
        }
    }

    /// same as level_up_spellslots but for warlock pact slots.
    fn level_up_warlock_pactslots(&mut self, slots_before: Option<PactSlots>) {
        let slots_after = self.pact_slots();
        match (slots_before, slots_after) {
            (Some(before), Some(after)) => {
                let new_num = after.num.saturating_sub(before.num);
                if let Some(current_slots) = &mut self.available_pact_slots {
                    current_slots.num += new_num;
                } else {
                    self.available_pact_slots = Some(PactSlots {
                        level: after.level,
                        num: new_num,
                    });
                }
                self.available_pact_slots.as_mut().unwrap().level = after.level;
            }
            (None, Some(after)) => {
                self.available_pact_slots = Some(after);
            }
            _ => {}
        }
    }

    /// Level up multiple times. Same as calling [Character::level_up] repeatedly.
    pub fn level_up_multiple(&mut self, class: &Class, times: usize) -> Option<usize> {
        if times <= 1 {
            return self.level_up(class);
        }

        let stats = self.stats();
        let spell_slots_before = self.spell_slots();
        let pact_slots_before = self.pact_slots();
        for _ in 0..times - 1 {
            self.level_up_inner(class, &stats)?;
        }
        let new_class_level = self.level_up_inner(class, &stats)?;

        self.hp = self.max_hp();
        self.level_up_spellslots(spell_slots_before);
        self.level_up_warlock_pactslots(pact_slots_before);
        Some(new_class_level)
    }

    /// Level up until the total level (not class level) is equal to the given number.
    pub fn level_up_to_level(&mut self, class: &Class, level: usize) -> Option<usize> {
        if level > 20 {
            return None;
        }
        let level_offset = (level as isize) - (self.level() as isize);
        if level_offset < 1 {
            return None;
        }

        self.level_up_multiple(class, level_offset as usize)
    }

    /// Returns the total equipment proficiencies for the character.
    ///
    /// This aggregates proficiencies from the class, possible race features, and
    ///  [Character::bonus_features].
    pub fn equipment_proficiencies(&self) -> EquipmentProficiencies {
        let feature_effects = self
            .race_features()
            .into_iter()
            .chain(self.subrace_features())
            .chain(self.bonus_features.iter())
            .flat_map(|v| v.effects.iter());

        let mut equipment_proficiencies = self.equipment_proficiencies.clone();

        for feature_effect in feature_effects {
            match feature_effect {
                FeatureEffect::WeaponProficiency(w) => match w {
                    WeaponType::Simple => equipment_proficiencies.simple_weapons = true,
                    WeaponType::SimpleRanged => equipment_proficiencies.simple_weapons = true,
                    WeaponType::Martial => equipment_proficiencies.martial_weapons = true,
                    WeaponType::MartialRanged => equipment_proficiencies.martial_weapons = true,
                },
                FeatureEffect::ArmorProficiency(a) => match a {
                    ArmorCategory::Light => equipment_proficiencies.light_armor = true,
                    ArmorCategory::Medium => equipment_proficiencies.medium_armor = true,
                    ArmorCategory::Heavy => equipment_proficiencies.heavy_armor = true,
                },
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
        let mut weapon_actions: Vec<_> = self
            .equipped_items()
            .into_iter()
            .filter_map(|v| match &v.0.item_type {
                ItemType::Weapon(w) => Some((&v.0.name, w)),
                _ => None,
            })
            .flat_map(|(name, weapon)| {
                weapon_actions(
                    name,
                    weapon,
                    &modifiers,
                    &equipment_proficiencies,
                    proficiency_modifier,
                )
                .into_iter()
            })
            .collect();

        // Unarmed Strike
        weapon_actions.push(WeaponAction {
            name: "Unarmed Strike".to_string(),
            attack_bonus: self.proficiency_bonus(),
            damage_roll: DamageRoll::new(0, 4, DamageType::Bludgeoning),
            damage_roll_bonus: modifiers.strength + self.proficiency_bonus(),
            two_handed: false,
            second_attack: false,
        });

        weapon_actions
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
    /// one of many types, so it would have many spell actions.
    ///
    /// If the character is a warlock, it still returns everthing up until their maximum spell
    /// level, since they're still able to downcast below their spell slot's level.
    pub fn spell_actions(&self) -> Vec<SpellAction> {
        let modifiers = self.stats().modifiers();

        let max_slot_level = match self.max_slot_level() {
            Some(v) => v,
            None => return vec![],
        };

        let mut char_spell_actions = vec![];
        for (index, class) in self
            .classes
            .iter()
            .enumerate()
            .filter(|v| v.1.spellcasting.is_some())
        {
            let spellcasting_stuff = self
                .spellcasting_scores_with_modifiers(index, &modifiers)
                .zip(class.spellcasting.as_ref());
            let ((_, attack_mod), (_, spells)) = match spellcasting_stuff {
                Some(s) => s,
                _ => continue,
            };

            let class_spell_actions = spells
                .iter()
                .filter_map(|s| spell_actions(s, attack_mod, max_slot_level, self.level()))
                .flat_map(|v| v.into_iter())
                .collect::<Vec<_>>();
            char_spell_actions.extend(class_spell_actions);
        }
        char_spell_actions
    }

    fn max_slot_level(&self) -> Option<usize> {
        let spell_slots = self
            .spell_slots()
            .map(|v| v.0.into_iter().position(|v| v == 0).unwrap_or(8) + 1);
        let pact_slots = self.pact_slots().map(|v| v.level + 1);

        match (spell_slots, pact_slots) {
            (Some(s), Some(p)) => Some(s.max(p)),
            (Some(s), None) => Some(s),
            (None, Some(p)) => Some(p),
            (None, None) => None,
        }
    }

    /// Gets the extra attacks granted by any feature(s) that do so.
    /// The resulting [ComputedCustomAction] has the final calculations needed to preform an
    /// attack.
    ///
    /// This may represent any extra feature that deals damage. Maybe your race has claws. Maybe
    /// your class adds 1d6 to every melee attack. Maybe a magical item allows you to make a
    /// special attack with it. Anything that isn't a regular attack with weapons or spells will
    /// fit here.
    pub fn ect_actions(&self) -> Vec<ComputedCustomAction> {
        self.total_features()
            .into_iter()
            .flat_map(|v| v.effects.iter())
            .filter_map(|v| match v {
                FeatureEffect::CustomAction(a) => Some(a),
                _ => None,
            })
            .map(|c| self.parse_custom_action(c))
            .collect()
    }

    fn parse_custom_action(&self, c: &CustomAction) -> ComputedCustomAction {
        let modifiers = self.stats().modifiers();
        let stats_attack_bonus = c
            .attack_bonus_stats
            .iter()
            .map(|v| modifiers.get_stat_type(v))
            .sum::<isize>();
        let attack_bonus = (c.static_attack_bonus as isize + stats_attack_bonus).max(0);

        let stats_damage_bonus = c
            .damage_bonus_stats
            .iter()
            .map(|v| modifiers.get_stat_type(v))
            .sum::<isize>();
        let damage_roll_bonus = (c.static_damage_bonus as isize + stats_damage_bonus).max(0);

        ComputedCustomAction {
            name: c.name.clone(),
            attack_bonus,
            damage_roll: c.damage_roll,
            damage_roll_bonus,
        }
    }

    /// A short rest.
    ///
    /// The 1st argument is the amount of hit die to spend.
    ///
    /// The 2nd argument is an optional manual override of the hit die rolls, which otherwise are
    /// just the averages. This is before constitution is added, so it's just the base dice rolls.
    ///
    /// Returns a bool of it it succeeded or not. The function fails if the amount of hit die are
    /// more than what's available, or if the hit die override has a different length than the amount of hit die spent.
    pub fn short_rest(&mut self, die_amount: usize, manual_hit_die: Option<Vec<usize>>) -> bool {
        let hit_die = self
            .classes
            .first()
            .expect("Character should have a class")
            .hit_die;

        if die_amount > self.level() - self.spent_hit_dice {
            return false;
        }

        let con_mod = if die_amount == 0 {
            0
        } else {
            self.stats().modifiers().constitution.max(0) as usize
        };

        let hit_die_rolls = match manual_hit_die {
            None => (die_average_max(hit_die) + con_mod) * die_amount,
            Some(v) => {
                if v.len() != die_amount {
                    return false;
                }
                v.into_iter().map(|n| n + con_mod).sum()
            }
        };

        let max_hp = self.max_hp();
        self.hp = (self.hp + hit_die_rolls).min(max_hp);

        self.spent_hit_dice += die_amount;

        // if there's warlock spell slots, they're replenished.
        if self.available_pact_slots.is_some() {
            self.available_pact_slots = self.pact_slots();
        }

        true
    }

    /// Calculates and applies the effects of taking a long rest.
    pub fn long_rest(&mut self) {
        // regain all hp
        self.hp = self.max_hp();

        // if there are spell slots, regain them
        if self.available_spell_slots.is_some() {
            self.available_spell_slots = self.spell_slots();
        }

        // if there's warlock spell slots, they're replenished.
        if self.available_pact_slots.is_some() {
            self.available_pact_slots = self.pact_slots();
        }

        // regain spent hit dice
        self.spent_hit_dice = self.spent_hit_dice.min(self.level()); // make sure it's valid
        let regained = (self.level() as f32 / 2.0).ceil() as usize;
        self.spent_hit_dice = self.spent_hit_dice.saturating_sub(regained);

        // regain features
        for class in self.classes.iter_mut() {
            let (specific_fields, etc_fields) = (&mut class.class_specific, &mut class.tracked_fields);
            for v in etc_fields {
                if !v.0.long_rest {
                    continue;
                }
                let class_specific_max: Option<usize> =
                    v.0.class_specific_max
                        .clone()
                        .and_then(|ref v| specific_fields.get(v)?.parse().ok());
                let max = v.0.hard_max.or(class_specific_max);
                if let Some(s) = max {
                    v.1 = s
                }
            }
        }
    }

    /// Returns the information necessary to select spells for each spellcasting class after a long rest. (or after creating
    /// the character.)
    ///
    /// The first field is the index of the [SpeccedClass] of the spellcasting class.
    ///
    /// The second field is the prepared spell list of that spellcaster. If you want to access it directly
    /// instead, just do `character.classes[index].spellcasting.unwrap().1` with the index
    /// provided.
    ///
    /// The third field is the amount of spells (1st level and onward) the character can prepare.
    ///
    /// The fourth field is the amount of cantrips the character can prepare.
    ///
    /// This function has no secondary effects, and is purely for retrieving data easily.
    pub fn prepare_spells(&mut self) -> Vec<(usize, &mut Vec<Spell>, usize, usize)> {
        let mut return_vector = vec![];
        let modifiers = self.stats().modifiers();

        for (n, class) in self.classes.iter_mut().enumerate() {
            let class_level = class.level;
            let casting = match class.spellcasting.as_mut() {
                Some(c) => c,
                _ => continue,
            };

            // if it isn't a class that prepares it's spells, continue to the next instance
            if !matches!(
                casting.0.preperation_type,
                SpellCastingPreperation::Prepared
            ) {
                continue;
            }

            let cantrips_num = casting.0.cantrips_per_level[class_level - 1];

            let ability = *modifiers.get_stat_type(&casting.0.spellcasting_ability);
            let spells_num = (class.level as isize + ability).max(0) as usize;
            return_vector.push((n, &mut casting.1, spells_num, cantrips_num));
        }

        return_vector
    }

    /// Gets the amount of spells the class at the index can prepare or know.
    ///
    /// Returns [None] if the class does not exist, or if the class is not a spellcaster.
    pub fn num_spells(&mut self, class_index: usize) -> Option<(usize, usize)> {
        let class_level = self.classes.get(class_index)?.level;
        if class_level == 0 {
            return None;
        }
        let casting = &self.classes.get(class_index)?.spellcasting.as_ref()?.0;
        let spellcasting_ability = casting.spellcasting_ability;
        let modifier = *self
            .stats()
            .modifiers()
            .get_stat_type(&spellcasting_ability);
        let cantrips_num = casting.cantrips_per_level[class_level - 1];
        let spells_num = (class_level as isize + modifier).max(0) as usize;

        Some((spells_num, cantrips_num))
    }
}

fn die_average_max(d: usize) -> usize {
    ((d as f32 + 1.0) / 2.0).ceil() as usize
}

fn spell_actions(
    spell: &Spell,
    spell_attack_mod: isize,
    max_slot_level: usize,
    character_level: usize,
) -> Option<Vec<SpellAction>> {
    if spell.level == 0 {
        return Some(vec![spell_action_cantrip(
            spell,
            spell_attack_mod,
            character_level,
        )?]);
    }
    Some(
        spell
            .damage
            .as_ref()?
            .iter()
            .enumerate()
            // filter out everything over what the spellcaster can cast
            .filter(|(n, _)| n + spell.level < max_slot_level)
            .flat_map(|(n, dv)| dv.iter().map(move |d| (n + spell.level, d)))
            .map(|(spell_level, damage)| SpellAction {
                spell_level: spell_level as isize,
                name: spell.name.clone(),
                spell_attack_mod,
                damage_roll: *damage,
            })
            .collect(),
    )
}

fn spell_action_cantrip(
    spell: &Spell,
    spell_attack_mod: isize,
    character_level: usize,
) -> Option<SpellAction> {
    let mut damage = spell.leveled_damage.as_ref()?.clone();
    // make sure damage is sorted by level
    damage.sort_by(|a, b| a.0.cmp(&b.0));
    // find the rightmost version we can use
    let position = damage
        .iter()
        .rposition(|(level, _)| *level <= character_level)
        .expect("Couldn't find a cantrip damage for this level.");

    Some(SpellAction {
        name: spell.name.clone(),
        spell_level: 0,
        spell_attack_mod,
        damage_roll: damage[position].1,
    })
}

fn weapon_actions(
    name: &String,
    w: &Weapon,
    m: &Modifiers,
    p: &EquipmentProficiencies,
    proficiency_mod: isize,
) -> Vec<WeaponAction> {
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

    let bonus = if proficient { proficiency_mod } else { 0 };

    let attack_bonus = modifier + bonus + (w.attack_roll_bonus as isize);
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
            attack_bonus,
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
/// Classes are bulky, and are the largest datastructure in the crate, aside from characters. A
/// SpeccedClass is like a fragment of that class, which has only the relevant information for a
/// character.
///
/// For example, if the character is a level 3 fighter, a [Class] would have every feature you
/// can take up until level 20, but the [SpeccedClass] only contains features from levels 1 to 3.
///
/// SpeccedClasses are internally built from a `Class`. Many of the fields are the same.
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

    /// A list of extra class fields that need to be actively tracked. See [TrackedField].
    ///
    /// The first field in the tuple is the [TrackedField], and the second field is the current
    /// amount the character has.
    pub tracked_fields: Vec<(TrackedField, usize)>,

    class_specific: HashMap<String, String>,
}

impl SpeccedClass {
    /// Get a specced class from a class, up to the specified level.
    fn from_class(class: &Class, level: usize) -> SpeccedClass {
        let subclass = PresentedOption::Choice(class.subclasses.to_vec());

        let base_tracked_fields= class.tracked_fields.clone();
        let tracked_fields = base_tracked_fields
            .into_iter()
            .map(|v| {
                let base_max = v.get_base_max(class).unwrap_or(1);
                (v, base_max)
            })
            .collect::<Vec<_>>();

        SpeccedClass {
            class: class.name.clone(),
            level,
            current_class_features: class
                .features
                .get(0..level)
                .expect("class doesn't have proper features!")
                .to_vec(),
            items: class.beginning_items.to_vec(),
            subclass,
            spellcasting: class.spellcasting.clone().map(|v| (v, vec![])),
            hit_die: class.hit_die,
            tracked_fields,
            class_specific: class
                .class_specific_leveled
                .iter()
                .map(|(k, arr)| (k.clone(), arr[0].clone()))
                .collect(),
        }
    }

    /// Get the total class's features.
    fn get_features(&self) -> Vec<&Feature> {
        self.current_class_features
            .iter()
            .flat_map(|level_features| chosen(level_features))
            .collect()
    }

    fn get_subclass_features(&self) -> Option<Vec<&Feature>> {
        let subclass = self.subclass.as_base()?;
        let features: Vec<_> = subclass.features[0..self.level]
            .iter()
            .flat_map(|v| v.iter())
            .filter_map(|v| v.as_base())
            .collect();
        Some(features)
    }

    /// Increments the character's level by 1 in that class.
    fn add_level(&mut self, class: &Class) {
        if self.level >= 20 {
            return;
        }
        self.current_class_features
            .push(class.features[self.level].clone());
        self.class_specific = class
            .class_specific_leveled
            .iter()
            .map(|(k, arr)| (k.clone(), arr[self.level].clone()))
            .collect();
        self.level += 1;
    }

    /// gets the etc class specific fields for the level. This is the same as [Class::class_specific_leveled], but specifically for the level that the current class is at.
    pub fn get_class_specific(&self) -> &HashMap<String, String> {
        &self.class_specific
    }
}

/// Represents something you can cast.
///
/// This is mainly used for [Character::cast].
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
        self.spell_level
            .try_into()
            .expect("spell level was negative")
    }
}

fn get_etc_field_max(
    etc_field: &TrackedField,
    class_specific: &HashMap<String, [String; 20]>,
    level: usize,
) -> Option<usize> {
    etc_field.hard_max.or(etc_field
        .class_specific_max
        .clone()
        .and_then(|v| class_specific.get(&v)?[level - 1].parse::<usize>().ok()))
}
