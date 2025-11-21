use crate::character::{
    features::{Feature, PresentedOption},
    stats::StatType,
};
use serde::{Deserialize, Serialize};

use super::stats::Size;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub name: String,
    pub speed: usize,
    /// Lists ability bonus by stat and the amount of the bonus.
    ///
    /// If the `Option<StatType>` is [None], then this means that the bonus can be chosen from any
    /// stat.
    pub ability_bonuses: Vec<(Option<StatType>, isize)>,
    pub size: Size,
    pub traits: Vec<PresentedOption<Feature>>,
    pub subraces: PresentedOption<Subrace>,
    pub languages: Vec<String>,
    pub wildcard_languages: Vec<Option<String>>,
}

impl PartialEq for Race {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Race {
    pub fn add_subrace(&mut self, subrace: Subrace) {
        match &mut self.subraces {
            PresentedOption::Base(b) => {
                let old = std::mem::replace(b, subrace.clone());
                self.subraces = PresentedOption::Choice(vec![old, subrace]);
            }
            PresentedOption::Choice(v) => v.push(subrace),
        }
    }
}

pub struct RaceBuilder {
    pub name: String,
    pub speed: usize,
    pub ability_bonuses: Vec<(Option<StatType>, isize)>,
    pub size: Size,
    pub traits: Vec<PresentedOption<Feature>>,
    pub subraces: PresentedOption<Subrace>,
    pub languages: Vec<String>,
    pub wildcard_languages: Vec<Option<String>>,
}

impl RaceBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            speed: 30,
            ability_bonuses: Vec::new(),
            size: Size::Medium,
            traits: Vec::new(),
            subraces: PresentedOption::Choice(vec![]),
            languages: Vec::new(),
            wildcard_languages: Vec::new(),
        }
    }

    pub fn push_ability_bonus(mut self, stat: Option<StatType>, bonus: isize) -> Self {
        self.ability_bonuses.push((stat, bonus));
        self
    }

    pub fn push_trait(mut self, trait_: PresentedOption<Feature>) -> Self {
        self.traits.push(trait_);
        self
    }

    pub fn push_language(mut self, language: String) -> Self {
        self.languages.push(language);
        self
    }

    pub fn push_subrace(mut self, subrace: Subrace) -> Self {
        match &mut self.subraces {
            PresentedOption::Base(b) => {
                let old = std::mem::replace(b, subrace.clone());
                self.subraces = PresentedOption::Choice(vec![old, subrace]);
            }
            PresentedOption::Choice(v) => v.push(subrace),
        }
        self
    }

    pub fn add_wildcard_language(mut self, num: usize) -> Self {
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
