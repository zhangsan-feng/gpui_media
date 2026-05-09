
use std::sync::Arc;
use regex::Regex;
use scraper::{Html, Selector};
use uuid::Uuid;
use crate::com::HttpClient;
use crate::entity::{ StreamMedioConvertLayer};
use crate::video_platform::youzisp::headers;


fn detail_func(url:String) -> Vec<String> {
    let mut call_back =    Vec::from([]);
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let response = HttpClient::new().get_for_html(&*url, headers()).await.expect("");
            let response_text = response.text().await.expect("html parse error");
            let document = Html::parse_document(&response_text);
            let selector = &Selector::parse(".module-play-list-content > a").expect("");
            for element in document.select(&selector).by_ref(){
                let link = element.value().attr("href").expect("href not found").to_string();
                call_back.push(link);
            }
        })
    });
    call_back
}

fn play_func(url:String) -> String {
    if url.contains(".m3u8"){
        return url
    }
    log::info!("{}", format!("https://youzisp.tv{}", url));

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let response = HttpClient::new()
                .get_for_html(format!("https://youzisp.tv{}", url).as_str(), headers()).await
                .expect("r");
            let response_text = response.text().await.expect("html parse error");
            let response_text_unescaped = response_text.replace("\\/", "/");
            let re = Regex::new(r#"https?://[^\s"'<>]+\.m3u8"#).unwrap();
            let url = re
                .captures(&response_text_unescaped)
                .and_then(|c| c.get(0))
                .map(|m| m.as_str().to_string())
                .expect("url not found");

            println!("{}", url);

            return  url;
        })
    })
}

async fn request_web() -> Vec<StreamMedioConvertLayer>{
    let mut call_back = Vec::from([]);

    let response = HttpClient::new()
        .get_for_html("https://youzisp.tv", headers()).await
        .expect("request https://youzisp.tv error");
    let response_text = response.text().await.expect("html parse error");
    let document = Html::parse_document(&response_text);

    for element in document.select(&Selector::parse(".main > .content > .module").expect("")).by_ref(){
        if element.html().contains("午夜剧场"){
            continue;
        }
        for elem in element.select(&Selector::parse(".module-main").expect("")).by_ref(){
            for ele in elem.select(&Selector::parse(".module-items > a").expect("")).by_ref() {

                let name = ele.value().attr("title").expect("Invalid title").trim().to_string();
                let mut link = ele.value().attr("href").expect("Invalid href").to_string();
                let img_select = Selector::parse(".module-item-pic > img").expect("Invalid img");
                let img_el = ele.select(&img_select).next().expect("未找到图片");
                let mut img = img_el.value().attr("data-original").expect("Image src").to_string();
                link = format!("https://youzisp.tv{}",link);
                if !img.contains("https://"){
                    img = format!("https://youzisp.tv/{}", img);
                }
                log::info!("{}, {}, {}", name, link, img);

                call_back.push(StreamMedioConvertLayer{
                    id: Uuid::new_v4().to_string(),
                    name:name,
                    img:img,
                    detail_link: link.to_string(),
                    detail_set: vec![],
                    headers: headers(),
                    detail_func: Arc::new(detail_func),
                    play_func: Arc::new(play_func),
                })
            }
        }
    }
    call_back
}


pub async fn recommend() -> Vec<StreamMedioConvertLayer>{
    request_web().await
}