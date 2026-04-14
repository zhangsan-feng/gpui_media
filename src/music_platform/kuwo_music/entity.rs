use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecommendRespRoot {
    pub code: f64,
    #[serde(rename = "curTime")]
    pub cur_time: f64,
    pub data: Data,
    pub msg: String,
    #[serde(rename = "profileId")]
    pub profile_id: String,
    #[serde(rename = "reqId")]
    pub req_id: String,
    #[serde(rename = "tId")]
    pub t_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataItem {
    pub img: String,
    pub uname: String,
    pub lossless_mark: String,
    pub favorcnt: String,
    pub isnew: String,
    pub extend: String,
    pub uid: String,
    pub total: String,
    pub commentcnt: String,
    pub imgscript: String,
    pub digest: String,
    pub name: String,
    pub listencnt: String,
    pub id: String,
    pub attribute: String,
    pub radio_id: String,
    pub desc: String,
    pub info: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    pub total: f64,
    pub data: Vec<DataItem>,
    pub rn: f64,
    pub pn: f64,
}