
use log::info;
use crate::entity::MusicConvertLayer;


pub mod kuwo_music;
pub mod dtwav_music;
pub mod xmwav_music;

mod bilibili;
mod _90svip;

/*


搜索平台
https://flac.life/
https://www.pjmp3.com/
https://music.90svip.cn/
https://www.yeyulingfeng.com/tools/music/
https://www.songe.cc/
https://music.wujiyan.cc/
https://1music.cc/zh-CN
https://music.gdstudio.xyz/

推荐平台
https://dtwav.com/
https://xmwav.net/
https://music.90svip.cn/

https://m.9ku.com/
https://m.uzz.me/
https://www.gequbao.com/




*/



pub async  fn music_recommend() -> anyhow::Result<Vec<MusicConvertLayer>>{
    let mut call_back = Vec::new();

    
    // match bilibili::recommend::call().await {
    //     Ok(val) => call_back.extend(val),
    //     Err(err) => info!("{}", err),
    // }



    match _90svip::recommend::call().await {
        Ok(val) => call_back.extend(val),
        Err(err) => info!("{}", err),
    }

    // match dtwav_music::recommend::call().await {
    //     Ok(val) => call_back.extend(val),
    //     Err(err) => info!("{}", err),
    // }
    // 
    // 
    // match xmwav_music::recommend::call().await{
    //     Ok(val) => call_back.extend(val),
    //     Err(err) => info!("{}", err),
    // }
    

    Ok(call_back)

}

pub fn music_search(keyword:&str) -> anyhow::Result<Vec<MusicConvertLayer>>{
    let mut call_back = Vec::new();

    Ok(call_back)

}
