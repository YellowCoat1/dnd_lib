# DnD Lib 
A rust library for creating and managing Dungeons & Dragons characters.

The main goal of this library is to model D&D characters, and have an easy interface for using those characters.


## Installation
Just add the latest version through `cargo add`:
```bash
cargo add dnd_lib
```
## Features
* Multiclasing
* Homebrew classes, races, items, and more
* Open-ended api trait
* Serializable datastructures for easy caching

## Usage
Creating a simple level 1 character. Do note this takes ~20 seconds, since the api gets are sequential.
```rust
use dnd_lib::prelude::*;
#[tokio::main]
async fn main() {
	let provider = Dnd5eapigetter::new();
	
	let rogue = provider.get_class("rogue").await.unwrap();
	let human = provider.get_race("human").await.unwrap();
	let acolyte = provider.get_background("acolyte").await.unwrap();

	let john = CharacterBuilder::new("John")
		.race(&human)
		.background(&acolyte)
		.class(&rogue)
		.stats(Stats::default())
		.build().unwrap();
		
	println!("John's ac is {}", john.ac());
	println!("John has {} strength", john.stats().strength);
}

```
