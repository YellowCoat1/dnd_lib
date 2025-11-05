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
        self.subraces = match self.subraces {
           PresentedOption::Base(ref b) => PresentedOption::Choice(vec![PresentedOption::Base(b.clone()), PresentedOption::Base(subrace)]),
           PresentedOption::Choice(ref v) => {
                let mut e = v.clone();
                e.push(PresentedOption::Base(subrace));
                PresentedOption::Choice(e)
           }
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
