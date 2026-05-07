use reqwest::header;

pub mod recommend;



fn headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
    headers.insert("accept-language", "zh-CN,zh;q=0.9".parse().unwrap());
    headers.insert("cache-control", "no-cache".parse().unwrap());
    headers.insert("pragma", "no-cache".parse().unwrap());
    headers.insert("priority", "u=0, i".parse().unwrap());
    headers.insert("sec-ch-ua", "\"Google Chrome\";v=\"147\", \"Not.A/Brand\";v=\"8\", \"Chromium\";v=\"147\"".parse().unwrap());
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
    headers.insert("sec-fetch-dest", "document".parse().unwrap());
    headers.insert("sec-fetch-mode", "navigate".parse().unwrap());
    headers.insert("sec-fetch-site", "none".parse().unwrap());
    headers.insert("sec-fetch-user", "?1".parse().unwrap());
    headers.insert("upgrade-insecure-requests", "1".parse().unwrap());
    headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".parse().unwrap());
    // headers.insert(header::COOKIE, "__51vcke__3Hc8GT8iyrvlO9ML=492921c0-a185-5e3a-a1a1-68ad25d2489a; __51vuft__3Hc8GT8iyrvlO9ML=1777445465639; mx_style=white; __51uvsct__3Hc8GT8iyrvlO9ML=5; showBtn=true; PHPSESSID=066se7fm7a7pcn9kkkdvkmp9ik; __vtins__3Hc8GT8iyrvlO9ML=%7B%22sid%22%3A%20%22c0748c93-f553-5698-bfa2-cfb186bc8920%22%2C%20%22vd%22%3A%202%2C%20%22stt%22%3A%2025179%2C%20%22dr%22%3A%2025179%2C%20%22expires%22%3A%201778048089703%2C%20%22ct%22%3A%201778046289703%7D".parse().unwrap());
    
    headers
}