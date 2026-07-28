use futures_util::StreamExt;
use gpui::http_client::http::HeaderMap;
use log::error;
use reqwest::{ClientBuilder, Response, multipart};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

trait ResponseHandler {
    async fn handle(self) -> anyhow::Result<serde_json::Value, anyhow::Error>;
}

impl ResponseHandler for reqwest::Response {
    async fn handle(self) -> anyhow::Result<serde_json::Value, anyhow::Error> {
        let status = self.status();
        let bytes = match self.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("读取响应失败: {}{}", err, timeout_suffix(&err));
                return Err(anyhow::anyhow!("读取响应失败: {}", err));
            }
        };
        // let body_str = String::from_utf8_lossy(&bytes);

        if status.is_success() {
            match serde_json::from_slice(&bytes) {
                Ok(data) => Ok(data),
                Err(err) => {
                    error!("响应序列化失败: {}", err);
                    // Err(anyhow::anyhow!("序列化失败: {}, 响应内容: {}", err, body_str))
                    Err(anyhow::anyhow!("序列化失败: {}", err))
                }
            }
        } else {
            error!("请求失败, 状态码: {}", status);
            // Err(anyhow::anyhow!("请求失败, 状态码: {}, 响应: {}", status, body_str))
            Err(anyhow::anyhow!("序列化失败: {}", status))
        }
    }
}

pub struct HttpClient {
    client: Arc<reqwest::Client>,
}

fn timeout_suffix(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        " (请求超时)"
    } else {
        ""
    }
}

// static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

impl HttpClient {
    pub fn new() -> Self {
        static CLIENT: OnceLock<Arc<reqwest::Client>> = OnceLock::new();

        Self {
            client: CLIENT
                .get_or_init(|| {
                    Arc::new(
                        ClientBuilder::new()
                            .timeout(Duration::from_secs(3))
                            .connect_timeout(Duration::from_secs(3))
                            .build()
                            .unwrap(),
                    )
                })
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
        let response = match self.client.get(&url).headers(header).send().await {
            Ok(response) => response,
            Err(err) => {
                error!("下载请求失败 [{}]: {}{}", url, err, timeout_suffix(&err));
                return Err(err.into());
            }
        };

        if !response.status().is_success() {
            error!("下载请求失败 [{}], 状态码: {}", url, response.status());
            return Err(anyhow::anyhow!("response not 200 "));
        }

        let mut file = tokio::fs::File::create(&file_name).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(err) => {
                    error!(
                        "下载数据读取失败 [{}]: {}{}",
                        url,
                        err,
                        timeout_suffix(&err)
                    );
                    return Err(err.into());
                }
            };
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
                error!("GET请求失败 [{}]: {}{}", url, e, timeout_suffix(&e));
                return Err(anyhow::anyhow!("GET请求失败: {}", e));
            }
        };

        if !response.status().is_success() {
            error!("GET请求失败 [{}], 状态码: {}", url, response.status());
        }

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
                error!("GET请求失败 [{}]: {}{}", url, e, timeout_suffix(&e));
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
                error!("POST请求失败 [{}]: {}{}", url, e, timeout_suffix(&e));
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
                error!("POST表单请求失败 [{}]: {}{}", url, e, timeout_suffix(&e));
                return Err(anyhow::anyhow!("POST表单请求失败: {}", e));
            }
        };
        response.handle().await
    }
}
