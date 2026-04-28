use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRoot {

    pub name: String,
    pub headers: Option<HashMap<String, String>>,


}

