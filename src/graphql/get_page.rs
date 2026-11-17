use graphql_client::{GraphQLQuery, Response};

pub(crate) async fn request_graphql_page<Q: GraphQLQuery>(
    vars: Q::Variables,
) -> Option<Q::ResponseData> {
    //get data
    let request_body = Q::build_query(vars);

    let client = reqwest::Client::new();
    let res = client
        .post("https://www.dnd5eapi.co/graphql")
        .json(&request_body)
        .send()
        .await
        .ok()?;
    let response_body: Response<Q::ResponseData> = res.json().await.ok()?;
    response_body.data
}
