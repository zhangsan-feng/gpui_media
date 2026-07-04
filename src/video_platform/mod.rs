/*

https://suonizy.net/                    cf + 验证码
http://caiji.dyttzyapi.com/             验证码
https://jszyapi.com/                    cf
https://1080zyk6.com                    cf
http://www.wujinzy.net/                 cf + 验证码
http://jinyingzy.com/                   cf
https://cj.ffzyapi.com                  验证码
https://api.guangsuapi.com/             cf
https://ukuzy0.com/
https://jszy333.com/
https://haohuazy.com/
https://www.ryzyw.com/
https://ffzy5.tv/
https://hongniuziyuan.net/



https://ukuzy0.com/index.php/vod/search.html?wd=%E5%87%A1%E4%BA%BA%E4%BF%AE%E4%BB%99
https://jszy333.com/index.php/vod/search.html?wd=%E5%87%A1%E4%BA%BA%E4%BF%AE%E4%BB%99
https://haohuazy.com/index.php/vod/search.html?wd=%E5%87%A1%E4%BA%BA%E4%BF%AE%E4%BB%99
https://www.ryzyw.com/index.php/vod/search.html?wd=%E5%87%A1%E4%BA%BA%E4%BF%AE%E4%BB%99
https://ffzy5.tv/index.php/vod/search.html?wd=%E5%87%A1%E4%BA%BA%E4%BF%AE%E4%BB%99
https://hongniuziyuan.net/index.php/vod/search.html?wd=%E5%87%A1%E4%BA%BA%E4%BF%AE%E4%BB%99





https://api.apibdzy.com                 验证码
https://lzizy.net/
https://yayazy2.com/
https://niuniuzy.cc                     验证码
https://okzyw.cc/
http://kuaichezy.com/                   验证码
https://www.taopianzy.com/index.html

http://wolongzyw.com/                   没法搜索
https://hongniuzy.net                   没法搜索



https://youzisp.tv
https://www.keke2.app/
https://www.renren.pro/
https://www.bttwo.org/
https://tyyszyapi.com/





cms 采集站
https://www.zzzypro.com/
https://www.yszzq.com/ziyuan/
*/
use crate::drive::NetworkStatic;
use futures_util::future::{BoxFuture, join_all};
use std::collections::HashMap;

mod ffzy5;
mod haohuazy;
mod hongniuziyuan;
mod jszy333;
mod renren;
mod ryzyw;
mod ukuzy0;
mod youzisp;

pub async fn search(keyword: String) -> HashMap<String, Vec<NetworkStatic>> {
    let platforms: Vec<(&'static str, BoxFuture<'static, Vec<NetworkStatic>>)> = vec![
        ("ukuzy0", Box::pin(ukuzy0::search::search(keyword.clone()))),
        (
            "jszy333",
            Box::pin(jszy333::search::search(keyword.clone())),
        ),
        (
            "haohuazy",
            Box::pin(haohuazy::search::search(keyword.clone())),
        ),
        ("ryzyw", Box::pin(ryzyw::search::search(keyword.clone()))),
        ("ffzy5", Box::pin(ffzy5::search::search(keyword.clone()))),
        (
            "hongniuziyuan",
            Box::pin(hongniuziyuan::search::search(keyword)),
        ),
    ];

    let names: Vec<_> = platforms.iter().map(|(name, _)| name.to_string()).collect();
    let searches: Vec<_> = platforms.into_iter().map(|(_, search)| search).collect();
    let results = join_all(searches).await;

    names.into_iter().zip(results.into_iter()).collect()
}

pub async fn recommend() -> Vec<NetworkStatic> {
    let platforms: Vec<BoxFuture<'static, Vec<NetworkStatic>>> = vec![
        Box::pin(youzisp::recommend::recommend()),
        Box::pin(renren::recommend::recommend()),
    ];

    join_all(platforms).await.into_iter().flatten().collect()
}
