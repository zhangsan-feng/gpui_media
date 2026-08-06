mod play;
mod recommend;
mod search;

use futures_util::future::BoxFuture;
use std::sync::Arc;

type FetchDocument = Arc<
    dyn Fn(
            String,
            PlatformConfig,
            super::config::ExtractType,
        ) -> BoxFuture<'static, anyhow::Result<super::template::ExtractedDocument>>
        + Send
        + Sync,
>;

use crate::plugins::extractor::config;
use crate::plugins::extractor::config::{PlatformConfig, ResourceType};
pub use recommend::recommend;
pub use search::search;

pub fn default_plugins() -> Vec<PlatformConfig> {
    config::load_default(ResourceType::Video)
}

fn default_fetcher() -> FetchDocument {
    Arc::new(|url, config, extract_type| {
        Box::pin(async move { super::config::fetch_document(&url, &config, extract_type).await })
    })
}

/*
# cms 站点
https://suonizy.net/                    验证码
http://caiji.dyttzyapi.com/             验证码
https://www.wujinzy.net/                验证码
http://jinyingzy.com/                   验证码
https://cj.ffzyapi.com                  验证码
https://api.apibdzy.com                 验证码
https://yayazy2.com/                    验证码
https://niuniuzy.cc                     验证码
https://okzyw.cc/                       验证码
http://kuaichezy.com/                   验证码
https://mtzy5.com/                      验证码

https://lzizy.net/
https://hongniuzy.net
https://ukuzy0.com/
https://jszy333.com/
https://haohuazy.com/
https://www.ryzyw.com/
https://ffzy5.tv/
https://hongniuziyuan.net/
http://ryzy.tv/


# cms 转发站
https://youzisp.tv
https://www.keke2.app/
https://www.renren.pro/
https://www.bttwo.org/
https://tyyszyapi.com/


cms 采集站
https://www.zzzypro.com/
https://www.yszzq.com/ziyuan/



*/
