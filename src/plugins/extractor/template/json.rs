use super::ExtractedItem;
use super::css;
use crate::com::request::HttpClient;
use crate::plugins::extractor::config::{
    FieldConfig, ItemChildrenConfig, ItemSplitConfig, PlatformConfig,
};
use serde_json::{Value, json};
use std::collections::HashMap;

pub(crate) async fn fetch(url: &str, config: &PlatformConfig) -> anyhow::Result<Value> {
    fetch_with_headers(url, config, css::headers(&config.headers)).await
}

pub(crate) async fn fetch_with_headers(
    url: &str,
    _config: &PlatformConfig,
    headers: reqwest::header::HeaderMap,
) -> anyhow::Result<Value> {
    HttpClient::new().get(url, headers).await
}

pub(crate) fn field_value(value: &Value, field: &FieldConfig) -> Option<String> {
    let value = json_path(value, &field.selector)?;
    let value = field
        .attribute
        .as_deref()
        .and_then(|attribute| value.get(attribute))
        .unwrap_or(value);
    json_string_value(value)
}

pub(crate) fn json_string(value: &Value, selector: &str) -> Option<String> {
    json_path(value, selector).and_then(json_string_value)
}

fn json_string_value(value: &Value) -> Option<String> {
    match value {
        Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

pub(crate) fn json_path<'a>(value: &'a Value, selector: &str) -> Option<&'a Value> {
    let selector = selector.trim();
    if selector.is_empty() || selector == ":scope" || selector == "$" {
        return Some(value);
    }

    let selector = selector.strip_prefix("$.").unwrap_or(selector);
    selector
        .split('.')
        .filter(|part| !part.is_empty())
        .try_fold(value, |current, part| match current {
            Value::Object(object) => object.get(part),
            Value::Array(array) => part
                .parse::<usize>()
                .ok()
                .and_then(|index| array.get(index)),
            _ => None,
        })
}

pub(crate) fn fill_template(template: &str, value: &str) -> String {
    template
        .replace("{{id}}", value)
        .replace("{{source}}", value)
}

pub(crate) fn parse_items(
    document: &Value,
    config: &ItemChildrenConfig,
    base: &str,
) -> Vec<ExtractedItem> {
    let Some(value) = json_path(document, &config.item_selector) else {
        return Vec::new();
    };
    let items = if let Some(items) = value.as_array() {
        items.clone()
    } else if let (Some(value), Some(item_split)) = (value.as_str(), config.item_split.as_ref()) {
        split_items(value, item_split)
    } else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let source =
                field_value(item, &config.source).map(|value| resolve_value(base, &value))?;
            let name = field_value(item, &config.name).unwrap_or_else(|| "未命名资源".to_string());
            let image = config
                .image
                .as_ref()
                .and_then(|field| field_value(item, field))
                .map(|value| resolve_value(base, &value))
                .unwrap_or_default();
            let author = config
                .author
                .as_ref()
                .and_then(|field| field_value(item, field))
                .unwrap_or_default();
            let extra = config
                .extra
                .iter()
                .filter_map(|(key, field)| {
                    field_value(item, field).map(|value| (key.clone(), json!(value)))
                })
                .collect::<HashMap<String, Value>>();
            Some(ExtractedItem::new(
                source,
                name,
                image,
                author,
                extra,
                Some(item.clone()),
            ))
        })
        .collect()
}

fn split_items(value: &str, config: &ItemSplitConfig) -> Vec<Value> {
    value
        .split(&config.item_separator)
        .filter_map(|item| {
            let item = item.trim();
            if item.is_empty() {
                return None;
            }
            let fields = config
                .field_separator
                .as_deref()
                .map(|separator| {
                    item.split(separator)
                        .map(|field| Value::String(field.trim().to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| vec![Value::String(item.to_string())]);
            Some(Value::Array(fields))
        })
        .collect()
}

fn resolve_value(base: &str, value: &str) -> String {
    if value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
    {
        super::css::resolve(base, value)
    } else {
        value.to_string()
    }
}
