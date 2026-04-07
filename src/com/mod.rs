use anyhow::Error;
use deno_core::{JsRuntime, RuntimeOptions, serde_v8};
use futures_util::StreamExt;
use gpui::http_client::http::HeaderMap;
use gpui::*;
use log::info;
use reqwest::{Response, multipart};
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;

pub fn call_js(js_path: &str, fn_name: &str, params: Vec<String>) -> Result<String, Error> {
    let js_code = fs::read_to_string(js_path)?;
    let mut runtime = JsRuntime::new(RuntimeOptions::default());
    runtime.execute_script("<init>", js_code)?;
    let args = params
        .into_iter()
        .map(|p| serde_json::to_string(&p).unwrap())
        .collect::<Vec<_>>()
        .join(",");

    let code = format!("{fn_name}({args})");
    let result = runtime.execute_script("<call>", code)?;
    let context = runtime.main_context();
    let isolate = runtime.v8_isolate();
    deno_core::v8::scope_with_context!(scope, isolate, context);

    let local = deno_core::v8::Local::new(scope, result);
    let result: String = serde_v8::from_v8(scope, local)?;

    Ok(result)
}

trait ResponseHandler {
    async fn handle(self) -> Result<serde_json::Value, anyhow::Error>;
}

impl ResponseHandler for reqwest::Response {
    async fn handle(self) -> Result<serde_json::Value, anyhow::Error> {
        let status = self.status();
        let bytes = self.bytes().await.unwrap_or_default();
        let body_str = String::from_utf8_lossy(&bytes);

        if status.is_success() {
            match serde_json::from_slice(&bytes) {
                Ok(data) => Ok(data),
                Err(err) => {
                    info!("序列化失败: {}", err);
                    // Err(anyhow::anyhow!("序列化失败: {}, 响应内容: {}", err, body_str))
                    Err(anyhow::anyhow!("序列化失败: {}", err))
                }
            }
        } else {
            info!("请求失败, 状态码: {}", status);
            // Err(anyhow::anyhow!("请求失败, 状态码: {}, 响应: {}", status, body_str))
            Err(anyhow::anyhow!("序列化失败: {}", status))
        }
    }
}

pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn download_music(
        &self,
        file_name: String,
        url: String,
        header: HeaderMap,
    ) -> anyhow::Result<()> {
        if Path::new(&file_name).exists() {
            return Ok(());
        }

        let client = reqwest::Client::new();
        let response = client.get(&url).headers(header).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("d"));
        }

        let mut file = tokio::fs::File::create(&file_name).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        println!("下载完成: {}", file_name);
        Ok(())
    }

    pub async fn get_for_html(
        &self,
        url: &str,
        header: HeaderMap,
    ) -> Result<Response, anyhow::Error> {
        let response = match self.client.get(url).headers(header).send().await {
            Ok(r) => r,
            Err(e) => {
                info!("GET请求失败 [{}]: {}", url, e);
                return Err(anyhow::anyhow!("GET请求失败: {}", e));
            }
        };
        Ok(response)
    }

    pub async fn get(
        &self,
        url: &str,
        header: HeaderMap,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let response = match self.client.get(url).headers(header).send().await {
            Ok(r) => r,
            Err(e) => {
                info!("GET请求失败 [{}]: {}", url, e);
                return Err(anyhow::anyhow!("GET请求失败: {}", e));
            }
        };

        response.handle().await
    }

    pub async fn post(
        &self,
        url: &str,
        header: HeaderMap,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let response = match self
            .client
            .post(url)
            .headers(header)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                info!("POST请求失败 [{}]: {}", url, e);
                return Err(anyhow::anyhow!("POST请求失败: {}", e));
            }
        };

        response.handle().await
    }

    pub async fn post_form(
        &self,
        url: String,
        form: multipart::Form,
    ) -> Result<serde_json::Value, anyhow::Error> {
        let response = match self.client.post(&url).multipart(form).send().await {
            Ok(r) => r,
            Err(e) => {
                info!("POST表单请求失败 [{}]: {}", url, e);
                return Err(anyhow::anyhow!("POST表单请求失败: {}", e));
            }
        };
        response.handle().await
    }
}

pub fn rgb_u8(r: u8, g: u8, b: u8) -> Rgba {
    let color: u32 = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    rgb(color)
}


// pub fn window_center_options(window: &mut Window, w: f32, h: f32) -> WindowOptions {
//     let parent_bounds = window.bounds();
//     let parent_x = parent_bounds.origin.x;
//     let parent_y = parent_bounds.origin.y;

//     let parent_width = parent_bounds.size.width;
//     let parent_height = parent_bounds.size.height;

//     let child_x = parent_x + (parent_width - px(w)) / 2.0;
//     let child_y = parent_y + (parent_height - px(h)) / 2.0;
//     let mut window_options = WindowOptions::default();
//     let window_size = size(px(w), px(h));

//     let bounds = Bounds {
//         origin: Point {
//             x: child_x,
//             y: child_y,
//         },
//         size: window_size,
//     };
//     window_options.window_bounds = Some(WindowBounds::Windowed(bounds));

//     window_options.window_min_size = Some(window_size);
//     window_options.is_resizable = true;
//     window_options.titlebar = Some(TitlebarOptions {
//         title: Some(SharedString::from("")),
//         appears_transparent: false,
//         ..Default::default()
//     });
//     window_options
// }
