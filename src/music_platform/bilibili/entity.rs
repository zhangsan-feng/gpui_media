


use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendTypeDataArchivesStat {
    #[serde(rename = "view")]
    pub view: i64,
    #[serde(rename = "like")]
    pub like: i64,
    #[serde(rename = "danmaku")]
    pub danmaku: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendTypeDataArchivesAuthor {
    #[serde(rename = "mid")]
    pub mid: i64,
    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendTypeDataArchives {
    #[serde(rename = "aid")]
    pub aid: i64,
    #[serde(rename = "bvid")]
    pub bvid: String,
    #[serde(rename = "cid")]
    pub cid: i64,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "cover")]
    pub cover: String,
    #[serde(rename = "duration")]
    pub duration: i64,
    #[serde(rename = "pubdate")]
    pub pubdate: i64,
    #[serde(rename = "stat")]
    pub stat: RecommendTypeDataArchivesStat,
    #[serde(rename = "author")]
    pub author: RecommendTypeDataArchivesAuthor,
    #[serde(rename = "trackid")]
    pub trackid: String,
    #[serde(rename = "goto")]
    pub goto: String,
    #[serde(rename = "rec_reason")]
    pub rec_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendTypeData {
    #[serde(rename = "archives")]
    pub archives: Vec<RecommendTypeDataArchives>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommend {
    #[serde(rename = "code")]
    pub code: i64,
    #[serde(rename = "message")]
    pub message: String,
    #[serde(rename = "ttl")]
    pub ttl: i64,
    #[serde(rename = "data")]
    pub data: RecommendTypeData,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailAudio{
    pub base_url:String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailDash{
    pub audio:Vec<DetailAudio>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  DetailData{
    pub dash:DetailDash
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detail {
    #[serde(rename = "code")]
    pub code: i32,
    #[serde(rename = "message")]
    pub message: String,
    #[serde(rename = "ttl")]
    pub ttl: i32,
    #[serde(rename = "data")]
    pub data: DetailData,
}


