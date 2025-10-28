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
    pub gender: String,
    pub eyes: String,
    pub height: String,
    pub faith: String,
    pub hair: String,
    pub skin: String,
    pub age: usize,
    pub weight: String,
    pub size: Size,
    pub alignment: Option<Alignment>,
}
