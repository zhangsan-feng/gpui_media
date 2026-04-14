

/*

https://www.kuwo.cn/api/www/classify/playlist/getRcmPlayList?pn=1&rn=20&order=new&httpsStatus=1&reqId=4f0d6b30-275c-11f1-864a-299a7923a0a7&plat=web_www&from= 每日推荐   rid 请求详情
https://www.kuwo.cn/api/www/playlist/playListInfo?pid=1082685104&pn=1&rn=20&httpsStatus=1&reqId=e7af4bc0-272e-11f1-bb54-1bffef53f12b&plat=web_www

https://www.kuwo.cn/api/www/classify/playlist/getTagPlayList?id=1848  翻唱
https://www.kuwo.cn/api/www/classify/playlist/getTagPlayList?id=621 网络
https://www.kuwo.cn/api/www/classify/playlist/getTagPlayList?id=146 伤感


isListenFee false 是免费
*/

use reqwest::{ Url};
use crate::com::HttpClient;
use crate::music_platform::kuwo_music::sign;





pub async fn detail(pid:&str, page:&str) ->Result<serde_json::Value, anyhow::Error>{

    let base_url = "https://www.kuwo.cn/api/www/playlist/playListInfo";
    let mut url = Url::parse(base_url)?;
    {
        let mut params = url.query_pairs_mut();
        params.append_pair("pid", pid);
        params.append_pair("pn", page);
        params.append_pair("rn", "30");
        params.append_pair("httpsStatus", "1");
        let id =  sign::uuid()?;
        params.append_pair("reqId", id.as_str());
        params.append_pair("plat", "web_www");
        params.append_pair("from", "");
    }


    HttpClient::new().get(url.as_ref(), sign::headers()).await
}

pub async fn call(page:&str) ->Result<serde_json::Value, anyhow::Error>{


    let base_url = "https://www.kuwo.cn/api/www/classify/playlist/getRcmPlayList";
    let mut url = Url::parse(base_url)?;
    {
        let mut params = url.query_pairs_mut();
        params.append_pair("pn", page);
        params.append_pair("rn", "20");
        params.append_pair("order", "hot");   // hot
        params.append_pair("httpsStatus", "1");
        let id =  sign::uuid()?;
        params.append_pair("reqId", id.as_str());
        params.append_pair("plat", "web_www");
        params.append_pair("from", "");
    }

    HttpClient::new().get(url.as_ref(), sign::headers()).await
}

