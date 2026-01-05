use std::time::{Duration, Instant};

#[test]
fn datastore_get_item() {
    let datastore = super::Dnd5eapiDatastore::new();

    datastore.request_item("shortsword");

    let start = Instant::now();
    let interval = Duration::from_millis(100);

    while start.elapsed() < Duration::from_secs(15) {
        if datastore.get_item("shortsword").is_some() {
            break;
        }
        std::thread::sleep(interval);
    }

    let item = match datastore.get_item("shortsword") {
        Some(item) => item,
        None => panic!("Datastore getter timed out"),
    };

    assert_eq!(item.name, "Shortsword");
}

#[test]
fn datastore_get_spell() {
    let datastore = super::Dnd5eapiDatastore::new();
    datastore.request_spell("fireball");

    let start = Instant::now();
    let interval = Duration::from_millis(100);

    while start.elapsed() < Duration::from_secs(15) {
        if datastore.get_spell("fireball").is_some() {
            break;
        }
        std::thread::sleep(interval);
    }

    let spell = match datastore.get_spell("fireball") {
        Some(spell) => spell,
        None => panic!("Datastore getter timed out"),
    };

    assert_eq!(spell.name, "Fireball");
    assert_eq!(spell.level, 3);
}
