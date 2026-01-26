# dnd_lib
A rust library for creating and managing Dungeons & Dragons characters and other related data.

This library is designed to model D&D's game logic, making it easier to build interactive character sheets and related tools. Elements of gameplay are stored in individual datastructures (Races, Classes, Spells, Items, etc.) that can be composed to create a full character. These datastructures can be fetched from the api or (somewhat tediously) created manually.

### Multiclassing
This library supports multiclassing and multiclass spellcasting. The Character struct can hold multiple classes, and many methods require "class index" parameter to specify which class to use for calculations. Spellcasting is handled by aggregating spell slots and spells known/prepared across all classes that provide spellcasting. Do note that warlocks have a seperate spell slot system that is not combined with other classes.

### Weapon Actions / Spell Actions
Something that you can do in combat is expressed with the Action trait. Note that this is different from the "Action" mechanic in D&D. These are made to show everything a character can do in combat, whether it be attacking with a weapon, casting a spell, or using a special ability. Actions can be created from weapons, some spells, and custom sources.

### Player "Feature"s
When a player gains an ability with a name, description, and possibly some mechanical effect, it is represented with the Feature struct. Features are often granted by classes, (every class ability is a feature,) races, feats, and items. Features express their mechanical effects through FeatureEffect enums, which can modify stats, grant actions, increase AC, and many other things. It's important to note that ability score increases are also represented as features.

### PresentedOption
In the crate, there's many places where a choice must be made, such as choosing between a dagger, or a light crossbow and 20 bolts. These choices are represented with the PresentedOption struct, which is an enum that can either be a single chosen option, or a list of options to choose from. This enum is used in many places across the crate.

## Fetching
The library provide the Dnd5eapigetter, which can fetch data from the [5e DnD API](https://www.dnd5eapi.co/) and parse it into the library's datastructures. The interface for this is through the DataProvider trait, which provides async methods for fetching each type of data.
> **Note:** Fetching from this api (particularly fetching classes) requires a large amount of http requests, so it is recommended to use a local cache or database to store fetched data for later use. These requests can be quite slow, and can run into the pitfalls of regular web requests. (Hanging, rate limiting, etc.)

### Datastore
To provide another option for data fetching, the library also provides the Dnd5eapiDatastore struct. Instead of a single async function, you can request data to add to a local hashmap datastore, and request it later at little cost. Requests are done on seperate threads.


## Future Plans
- More efficient getter functions for less http requests
- Integrations with popular VTTs
- 2024 D&D support
- Higher test coverage and more examples


If you'd like to build a tool to expand this library, particularly in one of the above areas, that would be greatly appreciated. Also, if you're building an application using this library, i'd love to hear about it.
