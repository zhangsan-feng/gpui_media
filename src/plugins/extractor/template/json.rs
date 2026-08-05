use super::ExtractedItem;
use super::css;
use crate::com::request::HttpClient;
use crate::plugins::extractor::config::{ChildrenConfig, FieldConfig, PageConfig, PlatformConfig};
use serde_json::{Value, json};
use std::collections::HashMap;

pub(crate) async fn fetch(url: &str, config: &PlatformConfig) -> anyhow::Result<Value> {
    HttpClient::new()
        .get(url, css::headers(&config.headers))
        .await
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

pub(crate) fn parse_page(document: &Value, page: &PageConfig, base: &str) -> Vec<ExtractedItem> {
    let Some(items) = json_path(document, &page.item_selector).and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let source = field_value(item, &page.detail_url)
                .map(|value| super::css::resolve(base, &value))?;
            let name = field_value(item, &page.name).unwrap_or_else(|| "未命名资源".to_string());
            let image = page
                .image
                .as_ref()
                .and_then(|field| field_value(item, field))
                .map(|value| super::css::resolve(base, &value))
                .unwrap_or_default();
            let extra = page
                .extra
                .iter()
                .filter_map(|(key, field)| {
                    field_value(item, field).map(|value| (key.clone(), json!(value)))
                })
                .collect::<HashMap<String, Value>>();
            Some(ExtractedItem::new(source, name, image, extra))
        })
        .collect()
}

pub(crate) fn parse_children(
    document: &Value,
    children: &ChildrenConfig,
    base: &str,
) -> Vec<(String, String, String)> {
    let Some(items) = json_path(document, &children.item_selector).and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let source = field_value(item, &children.play_url)
                .map(|value| super::css::resolve(base, &value))?;
            let name =
                field_value(item, &children.name).unwrap_or_else(|| "未命名分集".to_string());
            let image = children
                .image
                .as_ref()
                .and_then(|field| field_value(item, field))
                .map(|value| super::css::resolve(base, &value))
                .unwrap_or_default();
            Some((source, name, image))
        })
        .collect()
}
