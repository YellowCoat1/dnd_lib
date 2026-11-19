![Coverage: 92.22%](https://img.shields.io/badge/Coverage-92.22%25-green?style=plastic)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue?style=plastic)](https://github.com/YellowCoat1/dnd_lib/blob/master/LICENSE)

# DnD Lib

`dnd_lib` is a Rust library for creating and managing Dungeons & Dragons 5e
characters. It provides character-building utilities, multiclassing support, and
an API client for all basic classes and races.

## Installation

Add the latest published version to your project with Cargo:

```sh
cargo add dnd_lib
```

## Features

- Building and Managing D&D 5e characters
- Multiclassing
- Homebrew classes, races, items, and other rules
- Open-ended `DataProvider` trait for retrieving D&D API data

## Quick start

The following example builds a simple level 1 character:

```rust
use dnd_lib::prelude::*;

#[tokio::main]
async fn main() {
    let provider = GraphqlProvider::new();

    let rogue = provider.get_class("rogue").await.unwrap();
    let human = provider.get_race("human").await.unwrap();
    let acolyte = provider.get_background("acolyte").await.unwrap();

    let john = CharacterBuilder::new("John")
        .race(&human)
        .background(&acolyte)
        .class(&rogue)
        .stats(Stats::default())
        .build()?;

    println!("John's armor class is {}", john.ac());
    println!("John's Strength score is {}", john.stats().strength);
}
```
