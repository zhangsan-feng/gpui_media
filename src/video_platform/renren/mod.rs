use reqwest::header;

pub mod recommend;

fn headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert("accept-language", "zh-CN,zh;q=0.9".parse().unwrap());
    headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".parse().unwrap());
    headers
}
