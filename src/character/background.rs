use crate::character::items::Item;

use super::choice::PresentedOption;
use super::features::Feature;
use super::items::ItemCount;
use super::stats::SkillType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A D&D Character background.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Background {
    pub name: String,
    /// Skill proficiencies granted by the background.
    ///
    /// ## Proficiency Representation
    /// [Classes](crate::character::class::Class), which you may be more familiar with, represent
    /// skill proficiencies as a tuple of `(usize, PresentedOption<SkillType>)`, where the `usize`
    /// indicates how many skills you may choose from the presented options. This is because
    /// classes have a list to choose n skills from, but do not provide any base proficiencies.
    ///
    /// This is a `Vec<PresentedOption<SkillType>>` instead, as backgrounds can provide a fixed set
    /// or a choice of skill proficiencies. For example, the "Acolyte" background provides
    /// proficiency in two specific skills (Insight and Religion), while the "Charlatan" background
    /// allows you to choose two skills from a list of options.
    pub proficiencies: Vec<PresentedOption<SkillType>>,
    //pub languages: Vec<String>,
    /// A static list of starting equipment. Each item is paired with its count.
    pub equipment: Vec<ItemCount>,
    /// Features granted by this background.
    pub features: Vec<Feature>,
    /// Language options granted by this background.
    pub language_options: Vec<LanguageOption>,

    pub personality_traits: Vec<String>,
    pub ideals: Vec<String>,
    pub bonds: Vec<String>,
    pub flaws: Vec<String>,
}

/// Represents a single option between languages for a background.
///
/// Prefer using [LanguageOption::new_fixed] and [LanguageOption::new_named_choice] to construct
/// this, as it handles the string formatting.
///
///
/// ```
/// use dnd_lib::character::background::LanguageOption;
///
/// let choices = vec![
///   "Elvish".to_string(),
///   "dwArVish".to_string(),
///   "HALFLING".to_string(),
/// ];
/// let mut lang_option = LanguageOption::new_named_choice(choices);
///
/// assert_eq!(lang_option.set_to("Draconic".to_string()), false); // not in choices
///
/// lang_option.set_to("dwarvish".to_string());
/// assert_eq!(lang_option, LanguageOption::new_fixed("Dwarvish".to_string()));
///
/// ```

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageOption {
    /// A fixed, given language for the background.
    Fixed(String),
    /// A choice between multiple presented language options, e.g. "Choose one of: Common, Elvish, Dwarvish".
    NamedChoice(Vec<String>),
    /// A choice of languages without a specific name, e.g. "Choose two languages".
    UnnamedChoice,
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => s.to_string(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

impl LanguageOption {
    /// Sets the language option to a fixed language, returning true on success and false
    /// otherwise.
    ///
    /// This is useful for converting a choice option into a fixed option after the player has made
    /// their selection.
    ///
    /// # Errors
    /// Returns false if `self` is `Fixed`, if `s` is not in the list of choices for `NamedChoice`,
    /// or if `self` is `UnnamedChoice`. The value is not set in these cases.
    pub fn set_to(&mut self, s: String) -> bool {
        let s = capitalize_first(&s.to_lowercase());
        match self {
            LanguageOption::Fixed(_) => return false,
            LanguageOption::NamedChoice(choices) => {
                if !choices.contains(&s) {
                    return false;
                }
            }
            LanguageOption::UnnamedChoice => (),
        }
        let f = Self::new_fixed(s);
        *self = f;
        true
    }

    /// Constructs a fixed language option with proper capitalization.
    pub fn new_fixed(s: String) -> Self {
        LanguageOption::Fixed(capitalize_first(&s.to_lowercase()))
    }
    /// Constructs a named choice language option with proper capitalization.
    pub fn new_named_choice(choices: Vec<String>) -> Self {
        let choices: Vec<String> = choices
            .into_iter()
            .map(|s| capitalize_first(&s.to_lowercase()))
            .collect();
        LanguageOption::NamedChoice(choices)
    }
}

impl PartialEq for Background {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// An error in building a [Background] with a [BackgroundBuilder].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum BackgroundBuildError {
    #[error("Background must have at least one proficiency")]
    EmptyProficiencies,
    #[error("Background must have at least one ideal")]
    EmptyIdeals,
    #[error("Background must have at least one bond")]
    EmptyBonds,
    #[error("Background must have at least one flaw")]
    EmptyFlaws,
    #[error("Background must have at least two personality traits")]
    NotEnoughPersonalityTraits,
}

/// Builds a [Background] with a builder pattern.
///
/// ```rust
/// use dnd_lib::character::{
///     background::BackgroundBuilder,
///     background::LanguageOption,
///     stats::SkillType,
///     features::PresentedOption,
/// };
///
/// // a fixed language option that the background always provides
/// let elvish = LanguageOption::Fixed("Elvish".to_string());
/// // similarly, a fixed atheltics skill proficiency
/// let athletics = PresentedOption::Base(SkillType::Athletics);
/// // and a fixed perception skill proficiency
/// let perception = PresentedOption::Base(SkillType::Perception);
///
/// let bg_result = BackgroundBuilder::new("Test Background")
///     .add_proficiency(athletics)
///     .add_proficiency(perception)
///     .add_language_option(elvish)
///     .add_personality_trait("I am always calm under pressure.".to_string())
///     .add_personality_trait("I enjoy helping others.".to_string())
///     .add_ideal("Charity: I believe in helping those in need.".to_string())
///     .add_bond("I owe my life to the priest who saved me.".to_string())
///     .add_flaw("I have a quick temper.".to_string())
///     .build();
///
/// assert!(bg_result.is_ok());
/// ```
///  The following fields are required:
///  - At least 1 proficiency.
///  - At least 1 ideal.
///  - At least 1 bond.
///  - At least 1 flaw.
///  - At least 2 personality traits.
pub struct BackgroundBuilder {
    background: Background,
}

impl BackgroundBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            background: Background {
                name: name.to_string(),
                proficiencies: Vec::new(),
                equipment: Vec::new(),
                features: Vec::new(),
                language_options: Vec::new(),
                personality_traits: Vec::new(),
                ideals: Vec::new(),
                bonds: Vec::new(),
                flaws: Vec::new(),
            },
        }
    }

    pub fn add_proficiency(mut self, proficiency: PresentedOption<SkillType>) -> Self {
        self.background.proficiencies.push(proficiency);
        self
    }

    /// Adds equipment to the background's starting equipment.
    pub fn add_equipment(mut self, item: Item, count: usize) -> Self {
        self.background.equipment.push(ItemCount { item, count });
        self
    }

    /// Adds equipment to the background's starting equipment. Taking an [ItemCount].
    ///
    /// See also: [BackgroundBuilder::add_equipment]
    pub fn add_equipment_count(mut self, item_count: ItemCount) -> Self {
        self.background.equipment.push(item_count);
        self
    }

    pub fn add_feature(mut self, feature: Feature) -> Self {
        self.background.features.push(feature);
        self
    }

    pub fn add_language_option(mut self, option: LanguageOption) -> Self {
        self.background.language_options.push(option);
        self
    }

    pub fn add_personality_trait(mut self, trait_desc: String) -> Self {
        self.background.personality_traits.push(trait_desc);
        self
    }
    pub fn add_ideal(mut self, ideal: String) -> Self {
        self.background.ideals.push(ideal);
        self
    }
    pub fn add_bond(mut self, bond: String) -> Self {
        self.background.bonds.push(bond);
        self
    }
    pub fn add_flaw(mut self, flaw: String) -> Self {
        self.background.flaws.push(flaw);
        self
    }

    ///  Builds the background, consuming the builder.
    ///
    ///  # Errors
    ///  Returns a [BackgroundBuildError] if any required fields are missing or invalid.
    ///
    pub fn build(self) -> Result<Background, BackgroundBuildError> {
        use BackgroundBuildError::*;
        let bg = &self.background;

        if bg.proficiencies.is_empty() {
            return Err(EmptyProficiencies);
        }
        if bg.flaws.is_empty() {
            return Err(EmptyFlaws);
        }
        if bg.bonds.is_empty() {
            return Err(EmptyBonds);
        }
        if bg.ideals.is_empty() {
            return Err(EmptyIdeals);
        }
        if bg.personality_traits.len() < 2 {
            return Err(NotEnoughPersonalityTraits);
        }

        Ok(self.background)
    }
}
