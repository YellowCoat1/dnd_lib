use crate::character::{
    features::{PresentedOption, Feature}, 
    stats::StatType
};
use serde::{Serialize, Deserialize};

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
}

impl Race {
    pub fn add_subrace(&mut self, subrace: Subrace) {
        match &mut self.subraces {
           PresentedOption::Base(b) => {
               let old = std::mem::replace(b, subrace.clone());
               self.subraces = PresentedOption::Choice(vec![old, subrace]);
           },
           PresentedOption::Choice(v) => v.push(subrace),
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
