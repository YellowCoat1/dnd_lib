use crate::character::{
    features::{PresentedOption, Feature}, 
    stats::StatType
};
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub name: String,
    pub speed: usize,
    pub ability_bonuses: Vec<(StatType, isize)>,
    pub size: String,
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
    pub ability_bonuses: Vec<(StatType, isize)>,
    pub traits: Vec<PresentedOption<Feature>>,
}
