use serde_json::Value;
use std::collections::HashMap;

pub(crate) enum ExtractedDocument {
    Html(String),
    Json(Value),
}

pub(crate) struct ExtractedItem {
    pub(crate) source: String,
    pub(crate) name: String,
    pub(crate) image: String,
    pub(crate) author: String,
    pub(crate) extra: HashMap<String, Value>,
    pub(crate) raw: Option<Value>,
}

impl ExtractedItem {
    pub(crate) fn new(
        source: String,
        name: String,
        image: String,
        author: String,
        extra: HashMap<String, Value>,
        raw: Option<Value>,
    ) -> Self {
        Self {
            source,
            name,
            image,
            author,
            extra,
            raw,
        }
    }
}

pub(crate) mod css;
pub(crate) mod json;
pub(crate) mod regex;
