use serde::{Deserialize, Serialize};
use super::features::Feature;
use super::stats::SkillType;
use super::items::Item;
use super::choice::PresentedOption;

/// A D&D Character background.
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct Background {
    /// Skill proficiencies granted by the background
    pub proficiencies: Vec<PresentedOption<SkillType>>,
    //pub languages: Vec<String>,
    /// Item and count
    pub equipment: Vec<(Item, usize)>,
    /// Total features the background adds.
    pub features: Vec<Feature>,

    pub personality_traits: PresentedOption<String>,
    pub ideals: PresentedOption<String>,
    pub bonds: PresentedOption<String>,
    pub flaws: PresentedOption<String>,
}
