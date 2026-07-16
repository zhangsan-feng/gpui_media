use std::path::Path;
use std::sync::{Arc, OnceLock};
use futures_util::StreamExt;
use gpui::http_client::http::HeaderMap;
use log::info;
use reqwest::{multipart, Response};
use tokio::io::AsyncWriteExt;

trait ResponseHandler {
    async fn handle(self) -> anyhow::Result<serde_json::Value, anyhow::Error>;
}

impl ResponseHandler for reqwest::Response {
    async fn handle(self) -> anyhow::Result<serde_json::Value, anyhow::Error> {
        let status = self.status();
        let bytes = self.bytes().await.unwrap_or_default();
        // let body_str = String::from_utf8_lossy(&bytes);

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
    client: Arc<reqwest::Client>,
}

// static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

impl HttpClient {
    pub fn new() -> Self {
        static CLIENT: OnceLock<Arc<reqwest::Client>> = OnceLock::new();

        Self {
            client: CLIENT
                .get_or_init(|| Arc::new(reqwest::Client::new()))
                .clone(),
        }
    }

    pub async fn download_file(
        &self,
        file_name: String,
        url: String,
        header: HeaderMap,
    ) -> anyhow::Result<()> {
        if Path::new(&file_name).exists() {
            return Ok(());
        }

        println!("当前下载: {}", file_name);
        let client = reqwest::Client::new();
        let response = client.get(&url).headers(header).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("response not 200 "));
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
    ) -> anyhow::Result<Response, anyhow::Error> {
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
    ) -> anyhow::Result<serde_json::Value, anyhow::Error> {
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
    ) -> anyhow::Result<serde_json::Value, anyhow::Error> {
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
    ) -> anyhow::Result<serde_json::Value, anyhow::Error> {
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