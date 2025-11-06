use serde::{Deserialize, Serialize};
use super::features::Feature;
use super::stats::SkillType;
use super::items::Item;
use super::choice::PresentedOption;

/// A D&D Character background.
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
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

    pub personality_traits: PresentedOption<String>,
    pub ideals: PresentedOption<String>,
    pub bonds: PresentedOption<String>,
    pub flaws: PresentedOption<String>,
}
