use crate::{
    character::{choice::PresentedOption, class::ItemCategory, items::Item},
    prelude::*,
};

type ItemChoice = PresentedOption<Vec<(ItemCategory, usize)>>;

/// Builds a character from parts.
///
/// ```rust
/// # #[cfg(feature = "dnd5eapi")] {
/// # use dnd_lib::prelude::*;
/// # use tokio::runtime::Runtime;
/// # let rt = Runtime::new().unwrap();
/// # rt.block_on(async {
/// # let provider = Dnd5eapigetter::new();
/// # let barbarian = provider.get_class("barbarian").await.unwrap();
/// # let human = provider.get_race("human").await.unwrap();
/// # let acolyte = provider.get_background("acolyte").await.unwrap();
/// let george = CharacterBuilder::new("george")
///     .class(&barbarian)
///     .background(&acolyte)
///     .race(&human)
///     .stats(Stats::default())
///     .build().unwrap();
/// # })
/// # }
/// ```
///
/// ### Items
/// This builder can also be used to choose items while building the character with
/// [choose_items](CharacterBuilder::choose_items) and
/// [set_unchosen_category](CharacterBuilder::set_unchosen_category).
///
/// ```rust
/// # #[cfg(feature = "dnd5eapi")] {
/// # use dnd_lib::prelude::*;
/// # use tokio::runtime::Runtime;
/// # let rt = Runtime::new().unwrap();
/// # rt.block_on(async {
/// # let provider = Dnd5eapigetter::new();
/// # let barbarian = provider.get_class("barbarian").await.unwrap();
/// # let human = provider.get_race("human").await.unwrap();
/// # let acolyte = provider.get_background("acolyte").await.unwrap();
/// let spear = provider.get_item("spear").await.unwrap();
/// let george = CharacterBuilder::new("george")
///    // get the basic character parts out of the way
///    .class(&barbarian)
///    .background(&acolyte)
///    .race(&human)
///    .stats(Stats::default())
///    // The first item choice for barbarians is between "a greataxe" and "any martial melee weapon"
///    // We want the first option, so we choose index 0
///    .choose_items(0, 0).0
///    // The second item choice is between "two handaxes" and "any simple weapon"
///    // We want the second option, so we choose index 1
///    .choose_items(1, 1).0
///    // Now, we need to set what "any simple weapon" is. Let's make it a spear.
///    .set_unchosen_category(1, 0, spear).0
///    // Now that we've chosen our weapons, we can build the character.
///    .build().unwrap();
/// # })
/// # }
/// ```
///
/// There isn't anything for reading which items there are directly. Instead, read them from
/// the class using [Class::beginning_items].
// the i stands for internal
#[derive(Clone)]
pub struct CharacterBuilder<'a, 'b, 'c> {
    name: String,
    iclass: Option<&'a Class>,
    items: Option<(Vec<ItemChoice>, Vec<usize>)>,
    ibackground: Option<&'b Background>,
    irace: Option<&'c Race>,
    istats: Option<Stats>,
}

impl<'a, 'b, 'c> CharacterBuilder<'a, 'b, 'c> {
    pub fn new(name: &str) -> Self {
        CharacterBuilder {
            name: name.to_string(),
            iclass: None,
            items: None,
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

    // utility function for methods that need to set items.
    //
    // If items are already set, returns a mutable reference to them.
    // If not, intializes them from the class.
    // If there is no class, returns None.
    fn set_items(&mut self) -> Option<&mut (Vec<ItemChoice>, Vec<usize>)> {
        match (&mut self.items, &mut self.iclass) {
            (Some(_), Some(_)) => self.items.as_mut(),
            (None, Some(c)) => {
                let v = c.beginning_items().clone();
                let v2 = Self::choice_indices(&v);
                self.items = Some((v, v2));
                self.items.as_mut()
            }
            (_, None) => None,
        }
    }

    fn choice_indices(items: &Vec<ItemChoice>) -> Vec<usize> {
        
        let mut ret_vec = vec![];

        for (index, item) in items.iter().enumerate() {
            match item {
                PresentedOption::Base(_) => (),
                PresentedOption::Choice(_) => ret_vec.push(index),
            }
        }

        ret_vec
    }

    /// Sets the chosen items, in the case that you want to choose your items while building the
    /// character.
    ///
    /// This method acts in the same way as [Character::choose_items].
    ///
    /// Does nothing on an error.
    pub fn choose_items(mut self, index: usize, choice_index: usize) -> Self {
        let (items, indexes) = match self.set_items() {
            Some(s) => s,
            _ => return self,
        };

        let item_choice = match indexes.get(index).and_then(|&v| items.get_mut(v)) {
            Some(choice) => choice,
            _ => return self,
        };

        item_choice.choose_in_place(choice_index);
        self
    }

    /// Chooses an unchosen [ItemCategory] directly in the unchosen item list.
    ///
    /// This method acts in the same way as [Character::set_unchosen_category].
    ///
    /// Does nothing on an error.
    pub fn set_unchosen_category(
        mut self,
        index: usize,
        choice_index: usize,
        item: Item,
    ) -> Self {
        let (items, indexes) = match self.set_items() {
            Some(s) => s,
            _ => return self,
        };
        
        let item_choice = match indexes.get(index).and_then(|&v| items.get_mut(v)) {
            Some(choice) => choice,
            _ => return self,
        };

        let choices = match item_choice {
            PresentedOption::Base(choices) => choices,
            _ => return self,
        };
        let category = match choices.get_mut(choice_index) {
            Some(v) => &mut v.0,
            _ => return self,
        };
        *category = ItemCategory::Item(item);
        self
    }

    /// Builds the character. Panics if one or all of the fields have not been set.
    pub fn build(self) -> Result<Character, &'static str> {
        let class = self.iclass.ok_or("Missing class")?;
        let background = self.ibackground.ok_or("Missing background")?;
        let race = self.irace.ok_or("Missing race")?;
        let stats = self.istats.ok_or("Missing stats")?;

        let mut character = Character::new(self.name, class, background, race, stats);
        if let Some(items) = self.items {
            character.unchosen_items = items.0;
            character.add_chosen_items();
        }
        Ok(character)
    }
}
