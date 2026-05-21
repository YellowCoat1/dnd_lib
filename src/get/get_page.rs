#[cfg(test)]
use std::{hash::{DefaultHasher, Hash, Hasher}, path::PathBuf};

use serde_json::Value;

use super::Dnd5eapiError;

pub async fn get_page(path: String) -> Result<reqwest::Response, reqwest::Error> {
    let total_path = format!("https://www.dnd5eapi.co/api/2014/{path}");
    let response = reqwest::get(total_path).await?;
    Ok(response)
}

pub async fn get_raw_json(path: String) -> Result<serde_json::Value, Dnd5eapiError> {
    #[cfg(test)]
    if let Some(s) = get_cached(&path) {
        return Ok(s)
    }
    let json = get_page(path.clone()).await?.json::<Value>().await?;
    #[cfg(test)]
    set_cached(path, json.clone());

    Ok(json)
}

#[cfg(test)]
fn get_cached(path: &String) -> Option<serde_json::Value> {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let hashed_path = hasher.finish().to_string();
    let mut path = PathBuf::from(".test_http_cache");
    std::fs::create_dir_all(&path).ok()?;
    path.push(&hashed_path);
    if path.exists() {
        let v_str = std::fs::read_to_string(path).ok()?;
        let result = serde_json::from_str::<serde_json::Value>(&v_str).ok()?;
        return Some(result)
    }
    None
}

#[cfg(test)]
fn set_cached(path: String, val: serde_json::Value) -> Option<()> {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let hashed_path = hasher.finish().to_string();
    let mut path = PathBuf::from(".test_http_cache");
    std::fs::create_dir_all(&path).ok()?;
    path.push(hashed_path);
    let body = serde_json::to_string(&val).ok()?;
    std::fs::write(&path, &body).ok()?;
    Some(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_request() {
        let wizard_json = get_raw_json("classes/wizard".to_string()).await.unwrap();

        let map = match wizard_json {
            Value::Object(m) => m,
            _ => panic!("Json from api in an unexpected format"),
        };

        assert_eq!(
            map["url"],
            Value::String("/api/2014/classes/wizard".to_string())
        );
    }
}
