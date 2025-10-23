use serde_json::Value;

pub async fn get_page(path: String) -> Result<reqwest::Response, reqwest::Error>{
    let total_path = format!("https://www.dnd5eapi.co/api/2014/{path}");
    let response = reqwest::get(total_path)
        .await?;
    Ok(response)
}

pub async fn get_raw_json(path: String) -> Result<serde_json::Value, reqwest::Error> {
    let json = get_page(path)
        .await?
        .json::<Value>()
        .await?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_request() {
        let wizard_json =  get_raw_json("classes/wizard".to_string())
            .await
            .unwrap();

        let map = match wizard_json {
            Value::Object(m) => m,
            _ => panic!("Json from api in an unexpected format"),
        };

        assert_eq!(map["url"], Value::String("/api/2014/classes/wizard".to_string()));
    }
}
