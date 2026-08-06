pub const AUDIO_RECOMMEND_CONFIG: &str = r###"
[
  {
    "id":"bilibili","headers":{"user-agent":"Mozilla/5.0","referer":"https://www.bilibili.com/audio/home","origin":"https://www.bilibili.com","accept":"application/json, text/plain, */*"},"base_url":"https://www.bilibili.com","extract_type":"json","category":"热门推荐",
    "item_children":{"base_url":"/audio/music-service-c/web/menu/hit?pn=1&ps=12","item_selector":"data.data","source":{"selector":"menuId"},"name":{"selector":"title"},"author":{"selector":"uname"},"image":{"selector":"cover"},
      "detail":{"base_url":"/audio/music-service-c/web/song/of-menu?sid={{id}}&pn=1&ps=20","extract_type":"json","item_children":{"item_selector":"data.data","source":{"selector":"id"},"name":{"selector":"title"},"author":{"selector":"author"},"image":{"selector":"cover"}},"play":{"base_url":"/audio/music-service-c/web/url?sid={{id}}&privilege=2&quality=1","extract_type":"json","selector":"data.cdns.0"}}}
  },
  {
    "id":"bilibili","headers":{"user-agent":"Mozilla/5.0","referer":"https://www.bilibili.com/audio/home","origin":"https://www.bilibili.com","accept":"application/json, text/plain, */*"},"base_url":"https://www.bilibili.com","extract_type":"json","category":"歌曲榜单",
    "item_children":{"base_url":"/audio/music-service-c/web/home/hit-rank","item_selector":"data","source":{"selector":"menuId"},"name":{"selector":"title"},"author":{"selector":"uname"},"image":{"selector":"cover"},
      "detail":{"item_children":{"item_selector":"audios","source":{"selector":"id"},"name":{"selector":"title"}},"play":{"base_url":"/audio/music-service-c/web/url?sid={{id}}&privilege=2&quality=1","extract_type":"json","selector":"data.cdns.0"}}}
  }
]
"###;
