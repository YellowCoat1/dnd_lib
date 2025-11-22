use crate::character::{
    features::{Feature, PresentedOption},
    stats::StatType,
};
use heck::ToTitleCase;
use serde::{Deserialize, Serialize};

use super::stats::Size;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    name: String,
    speed: usize,
    ability_bonuses: Vec<(Option<StatType>, isize)>,
    size: Size,
    traits: Vec<PresentedOption<Feature>>,
    subraces: Vec<Subrace>,
    languages: Vec<String>,
    wildcard_languages: Vec<Option<String>>,
}

impl PartialEq for Race {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Race {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn speed(&self) -> usize {
        self.speed
    }
    /// Lists ability bonus by stat and the amount of the bonus.
    ///
    /// If the `Option<StatType>` is [None], then this means that the bonus can be chosen from any
    /// stat.
    pub fn ability_bonuses(&self) -> &Vec<(Option<StatType>, isize)> {
        &self.ability_bonuses
    }
    pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn traits(&self) -> &Vec<PresentedOption<Feature>> {
        &self.traits
    }
    pub fn subraces(&self) -> &Vec<Subrace> {
        &self.subraces
    }
    pub fn languages(&self) -> &Vec<String> {
        &self.languages
    }
    pub fn wildcard_languages(&self) -> &Vec<Option<String>> {
        &self.wildcard_languages
    }
    pub fn add_subrace(&mut self, subrace: Subrace) {
        self.subraces.push(subrace);
    }
}

pub struct RaceBuilder {
    pub name: String,
    pub speed: usize,
    pub ability_bonuses: Vec<(Option<StatType>, isize)>,
    pub size: Size,
    pub traits: Vec<PresentedOption<Feature>>,
    pub subraces: Vec<Subrace>,
    pub languages: Vec<String>,
    pub wildcard_languages: Vec<Option<String>>,
}

impl RaceBuilder {
    pub fn new(n: &str) -> Self {
        Self {
            name: n.to_title_case(),
            speed: 30,
            ability_bonuses: Vec::new(),
            size: Size::Medium,
            traits: Vec::new(),
            subraces: vec![],
            languages: Vec::new(),
            wildcard_languages: Vec::new(),
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name.to_title_case();
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
    pub fn speed(mut self, speed: usize) -> Self {
        self.speed = speed;
        self
    }

    pub fn add_ability_bonus(mut self, stat: Option<StatType>, bonus: isize) -> Self {
        self.ability_bonuses.push((stat, bonus));
        self
    }

    pub fn add_ability_bonuses<I>(mut self, bonuses: I) -> Self
    where
        I: IntoIterator<Item = (Option<StatType>, isize)>,
    {
        self.ability_bonuses.extend(bonuses);
        self
    }

    pub fn add_trait(mut self, trait_: PresentedOption<Feature>) -> Self {
        self.traits.push(trait_);
        self
    }

    pub fn add_traits<I>(mut self, traits: I) -> Self
    where
        I: IntoIterator<Item = PresentedOption<Feature>>,
    {
        self.traits.extend(traits);
        self
    }

    pub fn add_language(mut self, language: String) -> Self {
        self.languages.push(language);
        self
    }

    pub fn add_languages<I>(mut self, languages: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.languages.extend(languages);
        self
    }

    pub fn add_subrace(mut self, subrace: Subrace) -> Self {
        self.subraces.push(subrace);
        self
    }

    pub fn add_subraces<I>(mut self, subraces: I) -> Self
    where
        I: IntoIterator<Item = Subrace>,
    {
        self.subraces.extend(subraces);
        self
    }

    pub fn add_wildcard_language(mut self) -> Self {
        self.wildcard_languages.push(None);
        self
    }
    pub fn add_wildcard_languages(mut self, num: usize) -> Self {
        self.wildcard_languages.extend((0..num).map(|_| None));
        self
    }

    pub fn build(self) -> Race {
        Race {
            name: self.name,
            speed: self.speed,
            ability_bonuses: self.ability_bonuses,
            size: self.size,
            traits: self.traits,
            subraces: self.subraces,
            languages: self.languages,
            wildcard_languages: self.wildcard_languages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subrace {
    pub name: String,
    pub description: String,
    /// Lists ability bonuses.
    /// See [Race::ability_bonuses]
    pub ability_bonuses: Vec<(Option<StatType>, isize)>,
    pub traits: Vec<PresentedOption<Feature>>,
}

impl PartialEq for Subrace {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Subrace {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            ability_bonuses: Vec::new(),
            traits: Vec::new(),
        }
    }

    pub fn push_ability_bonus(&mut self, stat: Option<StatType>, bonus: isize) {
        self.ability_bonuses.push((stat, bonus));
    }

    pub fn push_trait(&mut self, race_trait: PresentedOption<Feature>) {
        self.traits.push(race_trait);
    }
}
