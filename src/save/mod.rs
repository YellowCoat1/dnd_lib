//! A simple helper to save other data to a file.
//!
//! ```
//! use dnd_lib::character::items::Item;
//! use dnd_lib::get::get_item;
//! use dnd_lib::save::{save_serialized, get_serialized};
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() {
//!     // get an item from the api. For simplicity, we're just going with the shortsword.
//!     let item = get_item("shortsword").await.unwrap();
//!     assert_eq!(&item.name, "Shortsword");
//!
//!     // We'll be saving it to item.json
//!     let item_path = Path::new("./item.json");
//!
//!     // First, we save it,
//!     save_serialized(item_path, &item)
//!         .expect("failed to save item to disk");
//!     // Then we get it from that path.
//!     let gotten_item: Item = get_serialized(item_path)
//!         .expect("failed to get item from disk");
//!     // Finally, we just double check that it's the same
//!     assert_eq!(item, gotten_item);
//! }


use std::path::Path;
use std::fs::{self, File};
use std::io::BufReader;
use std::error::Error;
use serde::Serialize;
use serde::de::DeserializeOwned;


pub fn save_serialized<T: Serialize>(path: &Path, t: &T) -> Result<(), Box<dyn Error>> {
    let class_string = serde_json::to_string(t)?;
    fs::write(path, class_string)?;
    Ok(())
}

pub fn get_serialized<T: DeserializeOwned>(path: &Path)  -> Result<T, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}


