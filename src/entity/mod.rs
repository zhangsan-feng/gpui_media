use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use url::Url;

#[derive(Clone)]
pub struct NetworkStatic{
    pub id:String,
    pub name:String,
    pub img: String,
    pub author:String,
    pub headers:reqwest::header::HeaderMap,
    pub source:String,
    pub func: Arc<dyn NetworkStaticInterface + Send + Sync>

}

impl Default for NetworkStatic {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            img: String::new(),
            author: String::new(),
            headers: reqwest::header::HeaderMap::new(),
            source: String::new(),
            func: Arc::new(LocalStatic),
        }
    }
}

pub trait NetworkStaticInterface{
    fn download(&self, params:&NetworkStatic);
    fn play(&self, params:&NetworkStatic) -> String;
}

impl NetworkStatic{
    pub fn download(&self){
        self.func.download(self);
    }
    pub fn play(&self, url:&str) -> String{
        self.func.play(self)
    }
}

pub struct LocalStatic;
impl NetworkStaticInterface for LocalStatic{
    fn download(&self, params:&NetworkStatic){

    }
    fn play(&self, params:&NetworkStatic) -> String {
        let source = params.source.trim();
        if source.is_empty() {
            panic!("player source not found");
            return String::new();
        }
        if source.contains("://") {
            return source.to_string();
        }

        Url::from_file_path(Path::new(source))
            .map(|uri| uri.to_string())
            .unwrap_or_else(|_| source.to_string())
    }
}
