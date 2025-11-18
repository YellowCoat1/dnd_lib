use crate::prelude::*;

/// Builds a character from parts.
///
/// ```ignore
/// let george = CharacterBuilder::new("george")
///     .class(barbarian)
///     .background(acolyte)
///     .race(human)
///     .stats(Stats::default())
///     .build().unwrap();
///
/// ```
// the i stands for internal
#[derive(Clone)]
pub struct CharacterBuilder<'a, 'b, 'c> {
    name: String,
    iclass: Option<&'a Class>,
    ibackground: Option<&'b Background>,
    irace: Option<&'c Race>,
    istats: Option<Stats>,
}

impl<'a, 'b, 'c> CharacterBuilder<'a, 'b, 'c> {
    pub fn new(name: &str) -> Self {
        CharacterBuilder {
            name: name.to_string(),
            iclass: None,
            ibackground: None,
            irace: None,
            istats: None,
        }
    }

    pub fn class(mut self, class: &'a Class) -> Self {
        self.iclass = Some(class);
        self
    }

    pub fn background(mut self, background: &'b Background) -> Self {
        self.ibackground = Some(background);
        self
    }

    pub fn race(mut self, race: &'c Race) -> Self {
        self.irace = Some(race);
        self
    }

    pub fn stats(mut self, stats: Stats) -> Self {
        self.istats = Some(stats);
        self
    }

    /// Builds the character. Panics if one or all of the fields have not
    pub fn build(self) -> Result<Character, &'static str> {
        let class = self.iclass.ok_or("Missing class")?;
        let background = self.ibackground.ok_or("Missing background")?;
        let race = self.irace.ok_or("Missing race")?;
        let stats = self.istats.ok_or("Missing stats")?;

        Ok(Character::new(self.name, class, background, race, stats))
    }
}
