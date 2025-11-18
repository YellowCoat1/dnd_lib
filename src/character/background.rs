use super::choice::PresentedOption;
use super::features::Feature;
use super::items::Item;
use super::stats::SkillType;
use serde::{Deserialize, Serialize};

/// A D&D Character background.
#[derive(Serialize, Deserialize, Clone)]
pub struct Background {
    pub name: String,
    /// Skill proficiencies granted by the background.
    ///
    /// This is a `Vec<PresentedOption<SkillType>>` instead of a `(usize,
    /// PresentedOption<SkillType>)` like [Class](crate::character::class::Class), as this field can have base proficiencies. A background might
    /// make you choose 2 from a list, or provide 2 static proficiencies.
    pub proficiencies: Vec<PresentedOption<SkillType>>,
    //pub languages: Vec<String>,
    /// Starting equipment. Each item is paired with its count.
    pub equipment: Vec<(Item, usize)>,
    /// Features granted by this background.
    pub features: Vec<Feature>,

    pub personality_traits: Vec<String>,
    pub ideals: Vec<String>,
    pub bonds: Vec<String>,
    pub flaws: Vec<String>,
}
