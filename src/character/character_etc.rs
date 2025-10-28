use serde::{Serialize, Deserialize};

use super::stats::{Alignment, Size};

/// A struct that contains all the etc strings you may want for describing the character.
///
/// All of the fields are split by paragraphs.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CharacterStory {
    pub organizations: Vec<String>,
    pub allies: Vec<String>,
    pub enemies: Vec<String>,
    pub backstory: Vec<String>,
    pub other: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CharacterDescriptors {
    gender: String,
    eyes: String,
    height: String,
    faith: String,
    hair: String,
    skin: String,
    age: usize,
    weight: String,
    size: Size,
    alignment: Option<Alignment>,
}
