use std::fmt::Debug;
use std::sync::Arc;
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

impl Debug for NetworkStatic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkStatic")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("img", &self.img)
            .field("author", &self.author)
            .field("headers", &self.headers)
            .field("source", &self.source)
            .finish()
    }
}

impl NetworkStatic{
    pub fn download(&self){
        self.func.download(self);
    }
    pub fn play(&self, url:&str) -> String{
        self.func.play(self)
    }
}

pub trait NetworkStaticInterface{
    fn download(&self, params:&NetworkStatic);
    fn play(&self, params:&NetworkStatic) -> String;
    fn detail(&self, params:&NetworkStatic) -> Vec<NetworkStatic>;
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
    fn detail(&self, params:&NetworkStatic) -> Vec<NetworkStatic>{
        Vec::new()
    }
}
