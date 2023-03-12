use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct Record {
    /// A valid IPv4 address.
    pub content: String,
    /// DNS record name
    pub name: String,
    /// type
    #[serde(rename = "type")]
    pub type_: String,
    /// comment for the record
    pub comment: Option<String>,
    /// Record Identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudflareResultItem {
    code: u32,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRequestResponse {
    pub result: Option<Vec<Record>>,
    pub errors: Vec<CloudflareResultItem>,
    pub messages: Option<Vec<CloudflareResultItem>>,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteRequestResponse {
    pub result: Option<Record>,
    pub errors: Vec<CloudflareResultItem>,
    pub messages: Option<Vec<CloudflareResultItem>>,
    pub success: bool,
}
