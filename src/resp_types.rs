use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct User {
    pub login: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Enterprise {}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Account {
    User(User),
    Enterprise(Enterprise),
}

#[derive(Deserialize, Debug)]
pub struct Installation {
    pub account: Account,
    pub access_tokens_url: String,
}

#[derive(Deserialize, Debug)]
pub struct TokenResp {
    pub token: String,
}
