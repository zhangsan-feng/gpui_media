

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConvertType {
    String(String),
    Number(u64),
}
impl std::fmt::Display for ConvertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConvertType::String(s) => write!(f, "{}", s),
            ConvertType::Number(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecpmmondMusicEntity {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "songSheetName")]
    pub song_sheet_name: String,
    #[serde(rename = "author")]
    pub author: String,
    #[serde(rename = "songId")]
    pub song_id: Vec<ConvertType>,
    #[serde(rename = "songIds")]
    pub song_ids: Vec<ConvertType>,
    #[serde(rename = "songNames")]
    pub song_names: Vec<String>,
    #[serde(rename = "songTypes")]
    pub song_types: Vec<String>,
    #[serde(rename = "albumNames")]
    pub album_names: Vec<String>,
    #[serde(rename = "artistNames")]
    pub artist_names: Vec<String>,
    #[serde(rename = "albumCovers")]
    pub album_covers: Vec<String>,
    #[serde(rename = "locations")]
    pub locations: Vec<String>,
    #[serde(rename = "sign")]
    pub sign: Vec<String>,
}
