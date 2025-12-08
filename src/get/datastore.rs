use super::raw_getters::*;
use crate::character::{items::Item, spells::Spell};
use crate::getter::CharacterDataError;
use crate::prelude::*;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;

enum LoadState<T> {
    Loading,
    Ready(Arc<T>),
}

type ClassRequester = InternalRequester<
    Class,
    Box<
        dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<Class, CharacterDataError>> + Send>>
            + Send
            + Sync,
    >,
>;

type BackgroundRequester = InternalRequester<
    Background,
    Box<
        dyn Fn(
                String,
            )
                -> Pin<Box<dyn Future<Output = Result<Background, CharacterDataError>> + Send>>
            + Send
            + Sync,
    >,
>;

type RaceRequester = InternalRequester<
    Race,
    Box<
        dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<Race, CharacterDataError>> + Send>>
            + Send
            + Sync,
    >,
>;

type ItemRequester = InternalRequester<
    Item,
    Box<
        dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<Item, CharacterDataError>> + Send>>
            + Send
            + Sync,
    >,
>;

type SpellRequester = InternalRequester<
    Spell,
    Box<
        dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<Spell, CharacterDataError>> + Send>>
            + Send
            + Sync,
    >,
>;

/// A different model for retrieving Dnd5e data from the api. Usually, you should use [Dnd5eapiGetter](super::Dnd5eapigetter) instead.
///
/// This datastore allows you to request data to be loaded in the background, and then check later
/// if it's ready. Data is requested on a sepearate thread using a Tokio runtime.
///
/// First, call a `request_` function to start loading the data you want. This will return
/// immediately. Then, whenever you need, call the corresponding `get_` function to check if the
/// data is ready. Note that the `get_` functions just retrieve from a [HashMap], so calling them multiple times is
/// cheap.
///
/// ```rust
/// use dnd_lib::get::Dnd5eapiDatastore;
/// let datastore = Dnd5eapiDatastore::new();
/// datastore.request_class("wizard".to_string());
/// // Do other stuff...
/// if let Some(wizard_class) = datastore.get_class("wizard".to_string()) {
///     // Use the wizard_class data
///     println!("Wizard class data is ready!");
/// } else {
///     println!("Wizard class data is still loading.");
/// }
/// ```
pub struct Dnd5eapiDatastore {
    runtime: Runtime,
    classes: ClassRequester,
    backgrounds: BackgroundRequester,
    races: RaceRequester,
    items: ItemRequester,
    spells: SpellRequester,
}

impl Dnd5eapiDatastore {
    pub fn new() -> Self {
        Self {
            classes: InternalRequester::new_async(|s: String| async move {
                Dnd5eapigetter::new().get_class(&s).await
            }),
            backgrounds: InternalRequester::new_async(|s: String| async move {
                Dnd5eapigetter::new().get_background(&s).await
            }),
            races: InternalRequester::new_async(|s: String| async move { get_race_raw(&s).await }),
            items: InternalRequester::new_async(|s: String| async move { get_item_raw(&s).await }),
            spells: InternalRequester::new_async(
                |s: String| async move { get_spell_raw(&s).await },
            ),
            runtime: Runtime::new().unwrap(),
        }
    }
    pub fn request_class(&self, class_name: String) {
        self.classes.request(class_name, &self.runtime);
    }
    pub fn get_class(&self, class_name: String) -> Option<Arc<Class>> {
        self.classes.try_get(class_name)
    }
    pub fn request_background(&self, background_name: String) {
        self.backgrounds.request(background_name, &self.runtime);
    }
    pub fn get_background(&self, background_name: String) -> Option<Arc<Background>> {
        self.backgrounds.try_get(background_name)
    }
    pub fn request_race(&self, race_name: String) {
        self.races.request(race_name, &self.runtime);
    }
    pub fn get_race(&self, race_name: String) -> Option<Arc<Race>> {
        self.races.try_get(race_name)
    }
    pub fn request_item(&self, item_name: String) {
        self.items.request(item_name, &self.runtime);
    }
    pub fn get_item(&self, item_name: String) -> Option<Arc<Item>> {
        self.items.try_get(item_name)
    }
    pub fn request_spell(&self, spell_name: String) {
        self.spells.request(spell_name, &self.runtime);
    }
    pub fn get_spell(&self, spell_name: String) -> Option<Arc<Spell>> {
        self.spells.try_get(spell_name)
    }
}

impl Default for Dnd5eapiDatastore {
    fn default() -> Self {
        Self::new()
    }
}

struct InternalRequester<T, U>
where
    U: Fn(String) -> Pin<Box<dyn Future<Output = Result<T, CharacterDataError>> + Send>>
        + Send
        + Sync
        + 'static,
    T: Send + Sync + 'static,
{
    cache: Arc<Mutex<HashMap<String, LoadState<T>>>>,
    get_func: Arc<U>,
}

impl<T, U> InternalRequester<T, U>
where
    U: Fn(String) -> Pin<Box<dyn Future<Output = Result<T, CharacterDataError>> + Send>>
        + Send
        + Sync
        + 'static,
    T: Send + Sync + 'static,
{
    fn new(get_func: U) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            get_func: Arc::new(get_func),
        }
    }

    /// Start loading if not already loading / loaded.
    pub fn request(&self, class_name: String, rt: &Runtime) {
        let key = class_name.to_lowercase();
        let mut map = self.cache.lock().unwrap();

        // If already loading or loaded, do nothing
        if map.get(&key).is_some() {
            return;
        }

        map.insert(key.clone(), LoadState::Loading);
        drop(map);

        let cache = Arc::clone(&self.cache);
        let get = Arc::clone(&self.get_func);

        rt.spawn(async move {
            match get(key.clone()).await {
                Ok(value) => {
                    cache
                        .lock()
                        .unwrap()
                        .insert(key, LoadState::Ready(Arc::new(value)));
                }
                Err(err) => {
                    eprintln!("error fetching {key}: {err}");
                }
            }
        });
    }

    /// Check if the value is ready
    pub fn try_get(&self, name: String) -> Option<Arc<T>> {
        let map = self.cache.lock().unwrap();
        match map.get(&name.to_lowercase())? {
            LoadState::Ready(data) => Some(Arc::clone(data)),
            LoadState::Loading => None,
        }
    }
}

impl<T>
    InternalRequester<
        T,
        Box<
            dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<T, CharacterDataError>> + Send>>
                + Send
                + Sync,
        >,
    >
where
    T: Send + Sync + 'static,
{
    fn new_async<F, Fut>(f: F) -> Self
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, CharacterDataError>> + Send + 'static,
    {
        Self::new(Box::new(move |name| Box::pin(f(name))))
    }
}
