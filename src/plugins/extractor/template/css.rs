use super::super::config::{FieldConfig, PageConfig};
use super::ExtractedItem;
use crate::com::request::HttpClient;
use anyhow::{Context, Result};
use reqwest::{
    Url,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use scraper::{ElementRef, Html, Selector};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

pub(crate) fn headers(values: &HashMap<String, String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in values {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(name), HeaderValue::try_from(value)) {
            headers.insert(name, value);
        }
    }
    headers
}

pub(crate) async fn fetch(url: &str, request_headers: &HeaderMap) -> Result<String> {
    let response = HttpClient::new()
        .get_for_html(url, request_headers.clone())
        .await
        .context("extractor request failed")?;
    let status = response.status();
    let body = response
        .text()
        .await
        .context("extractor response read failed")?;
    if !status.is_success() {
        let preview = body.chars().take(160).collect::<String>();
        anyhow::bail!("extractor response failed: {} [{}]", status, preview);
    }
    Ok(body)
}

pub(crate) fn resolve(base: &str, path: &str) -> String {
    Url::parse(path)
        .map(|url| url.to_string())
        .or_else(|_| Url::parse(base).and_then(|base| base.join(path).map(|url| url.to_string())))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| path.to_string())
}

pub(crate) fn search_url(template: &str, keyword: &str) -> String {
    template.replace("{{keyword}}", &percent_encode(keyword))
}

fn percent_encode(value: &str) -> String {
    value.bytes().fold(String::new(), |mut encoded, byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{:02X}", byte));
        }
        encoded
    })
}

pub(crate) fn parse_page(html: &str, page: &PageConfig, base: &str) -> Vec<ExtractedItem> {
    let document = Html::parse_document(html);
    let Ok(item_selector) = Selector::parse(&page.item_selector) else {
        return Vec::new();
    };
    document
        .select(&item_selector)
        .filter_map(|item| {
            let source = field_value(item, &page.detail_url).map(|value| resolve(base, &value))?;
            let name = field_value(item, &page.name)
                .or_else(|| field_value(item, &FieldConfig::text(":scope")))
                .unwrap_or_else(|| "未命名资源".to_string());
            let image = page
                .image
                .as_ref()
                .and_then(|field| field_value(item, field))
                .map(|value| resolve(base, &value))
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
    html: &str,
    item: &super::super::config::ChildrenConfig,
    base: &str,
) -> Vec<(String, String, String)> {
    let document = Html::parse_document(html);
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    if let Ok(selector) = Selector::parse(&item.item_selector) {
        for element in document.select(&selector) {
            if let Some(value) = parse_child(element, item, base)
                && seen.insert(value.0.clone())
            {
                result.push(value);
            }
        }
    }

    if let Ok(selector) = Selector::parse("a[href]") {
        for element in document.select(&selector) {
            let Some(href) = element.value().attr("href") else {
                continue;
            };
            if !is_play_url(href) {
                continue;
            }
            if let Some(value) = parse_child(element, item, base)
                && seen.insert(value.0.clone())
            {
                result.push(value);
            }
        }
    }

    result
}

fn parse_child(
    element: ElementRef<'_>,
    item: &super::super::config::ChildrenConfig,
    base: &str,
) -> Option<(String, String, String)> {
    let source = field_value(element, &item.play_url).map(|value| resolve(base, &value))?;
    let name = field_value(element, &item.name).unwrap_or_else(|| "未命名分集".to_string());
    let image = item
        .image
        .as_ref()
        .and_then(|field| field_value(element, field))
        .map(|value| resolve(base, &value))
        .unwrap_or_default();
    Some((source, name, image))
}

fn is_play_url(value: &str) -> bool {
    value.contains("/vod/play") || value.contains("vodplay") || value.contains(".m3u8")
}

fn field_value(element: ElementRef<'_>, field: &FieldConfig) -> Option<String> {
    let selected = if field.selector == ":scope" {
        Some(element)
    } else {
        Selector::parse(&field.selector)
            .ok()
            .and_then(|selector| element.select(&selector).next())
    }?;
    let value = match field.attribute.as_deref() {
        Some(attribute) => selected.value().attr(attribute)?.to_string(),
        None => selected.text().collect::<String>(),
    };
    let value = html_unescape(value.trim());
    (!value.is_empty()).then_some(value)
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}
