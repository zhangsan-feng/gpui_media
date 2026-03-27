use log::info;
use crate::entity::MusicConvertLayer;

pub mod kugou_music;
pub mod dtwav_music;
pub mod xmwav_music;
mod gdstudio;
mod bilibili;
/*

https://dtwav.com/
https://xmwav.net/
https://music-api.gdstudio.xyz/api.php

*/



pub async  fn music_recommend() -> Result<Vec<MusicConvertLayer>, anyhow::Error>{
    let mut call_back = Vec::new();
    match dtwav_music::recommend::call().await {
        Ok(val)=>{
            call_back.extend(val)
        }
        Err(e)=>{
            info!("err:{}",e)
        }
    }
    match xmwav_music::recommend::call().await{
        Ok(val)=>{
            call_back.extend(val)
        }
        Err(e)=>{
            info!("err:{}",e)
        }
    }
    Ok(call_back)
    // xmwav_music::recommend::call().await;
    // kugou_music::recommend::call(page).await
}


pub fn music_local(){}


pub fn music_search(keyword:&str) -> Result<Vec<MusicConvertLayer>, anyhow::Error>{
    let mut call_back = Vec::new();

    Ok(call_back)
    
}