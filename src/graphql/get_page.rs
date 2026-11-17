#[cfg(feature = "network-intensive-tests")]
use std::hash::{DefaultHasher, Hash, Hasher};

use graphql_client::{GraphQLQuery, QueryBody};
use reqwest::Response as ReqwestResponse;
use serde::{de::DeserializeOwned, Serialize};

pub(crate) async fn request_graphql_page<Q>(vars: Q::Variables) -> Option<Q::ResponseData>
where
    Q: GraphQLQuery,
    Q::Variables: Serialize,
    Q::ResponseData: DeserializeOwned,
{
    #[cfg(feature = "network-intensive-tests")]
    let mut hasher = DefaultHasher::new();
    #[cfg(feature = "network-intensive-tests")]
    serde_json::to_string(&vars).unwrap().hash(&mut hasher);
    #[cfg(feature = "network-intensive-tests")]
    let hashed_val = hasher.finish().to_string();

    #[cfg(feature = "network-intensive-tests")]
    if let Some(data) = get_cached(&hashed_val) {
        return serde_json::from_value(data.clone()).ok()?;
    };
    //get data
    let request_body: QueryBody<Q::Variables> = Q::build_query(vars);

    let client = reqwest::Client::new();
    let res: ReqwestResponse = client
        .post("https://www.dnd5eapi.co/graphql")
        .json(&request_body)
        .send()
        .await
        .ok()?;
    let response_body: serde_json::Value = res.json().await.ok()?;

    let data = response_body.get("data")?.clone();
    #[cfg(feature = "network-intensive-tests")]
    set_cached(hashed_val, data.clone()).unwrap();

    let parsed: <Q as GraphQLQuery>::ResponseData = serde_json::from_value(data.clone()).ok()?;

    Some(parsed)
}

#[cfg(feature = "network-intensive-tests")]
fn get_cached(path: &str) -> Option<serde_json::Value> {
    use std::{
        hash::{DefaultHasher, Hash, Hasher},
        path::PathBuf,
    };

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let hashed_path = hasher.finish().to_string();
    let mut path = PathBuf::from(".test_graphql_cache");
    std::fs::create_dir_all(&path).ok()?;
    path.push(&hashed_path);
    if path.exists() {
        let v_str = std::fs::read_to_string(path).ok()?;
        let result = serde_json::from_str::<serde_json::Value>(&v_str).ok()?;
        return Some(result);
    }
    None
}

#[cfg(feature = "network-intensive-tests")]
fn set_cached(path: String, val: serde_json::Value) -> Option<()> {
    use std::{
        hash::{DefaultHasher, Hash, Hasher},
        path::PathBuf,
    };
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let hashed_path = hasher.finish().to_string();
    let mut path = PathBuf::from(".test_graphql_cache");
    std::fs::create_dir_all(&path).ok()?;
    path.push(hashed_path);
    let body = serde_json::to_string(&val).ok()?;
    std::fs::write(&path, &body).ok()?;
    Some(())
}
