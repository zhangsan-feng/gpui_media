
use log::info;
use crate::entity::MusicConvertLayer;


pub mod kuwo_music;
pub mod dtwav_music;
pub mod xmwav_music;
mod gdstudio;
mod bilibili;
mod _90svip;
mod uzz;
/*

https://dtwav.com/
https://xmwav.net/
https://music-api.gdstudio.xyz/api.php



https://music.90svip.cn/
https://m.uzz.me/

*/



pub async  fn music_recommend() -> anyhow::Result<Vec<MusicConvertLayer>>{
    let mut call_back = Vec::new();

    
    // match bilibili::recommend::call().await { 
    //     Ok(val) => call_back.extend(val),
    //     Err(err) => info!("{}", err),
    // }
    
    match uzz::recommend::call().await {
        Ok(val) => call_back.extend(val),
        Err(err) => info!("{}", err),
    }
    
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
