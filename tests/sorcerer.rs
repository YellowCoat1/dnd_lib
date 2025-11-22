#![cfg(feature = "network-intensive-tests")]

use dnd_lib::prelude::*;

#[tokio::test]
async fn sorcerer() {
    let provider = Dnd5eapigetter::new();
    let sorcerer = provider.get_class("sorcerer").await.unwrap();
    assert_eq!(sorcerer.name(), "Sorcerer");
}
