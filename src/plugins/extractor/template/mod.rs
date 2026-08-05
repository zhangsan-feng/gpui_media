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
    pub(crate) extra: HashMap<String, Value>,
}

impl ExtractedItem {
    pub(crate) fn new(
        source: String,
        name: String,
        image: String,
        extra: HashMap<String, Value>,
    ) -> Self {
        Self {
            source,
            name,
            image,
            extra,
        }
    }
}

pub(crate) mod css;
pub(crate) mod json;
pub(crate) mod regex;
