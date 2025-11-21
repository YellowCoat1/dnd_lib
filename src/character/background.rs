use super::choice::PresentedOption;
use super::features::Feature;
use super::items::Item;
use super::stats::SkillType;
use serde::{Deserialize, Serialize};

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
    pub equipment: Vec<(Item, usize)>,
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageOption {
    /// A fixed, given language for the background. One chosen LanguageOption collapses into this.
    Fixed(String),
    /// A choice between multiple presented language options, e.g. "Choose one of: Common, Elvish, Dwarvish".
    NamedChoice(PresentedOption<String>),
    /// A choice of languages without a specific name, e.g. "Choose two languages".
    UnnamedChoice,
}

impl PartialEq for Background {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
