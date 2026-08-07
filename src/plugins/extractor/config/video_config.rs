pub const VIDEO_SEARCH_CONFIG: &str = r###"
[
  {
    "id":"lzizy","headers":{"accept":"*/*","accept-language":"zh-CN,zh;q=0.9","cache-control":"no-cache","origin":"https://lzizy.net","pragma":"no-cache","priority":"u=1, i","referer":"https://lzizy.net/","sec-ch-ua":"\"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"150\", \"Google Chrome\";v=\"150\"","sec-ch-ua-mobile":"?0","sec-ch-ua-platform":"\"Windows\"","sec-fetch-dest":"empty","sec-fetch-mode":"cors","sec-fetch-site":"cross-site","user-agent":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/150.0.0.0 Safari/537.36"},"base_url":"https://lzizy.net","extract_type":"json",
    "item_children":{"base_url":"https://macapi1.com/maccms/json/liangzi/?ac=videolist&wd={{keyword}}&pg=1","item_selector":"list","source":{"selector":"vod_id"},"name":{"selector":"vod_name"},"image":{"selector":"vod_pic"},"extra":{"update_time":{"selector":"vod_time"}},
      "detail":{"base_url":"https://macapi1.com/maccms/json/liangzi/?ac=detail&ids={{source}}","extract_type":"json","item_children":{"item_selector":"list.0.vod_play_url","item_split":{"item_separator":"#","field_separator":"$"},"source":{"selector":"1"},"name":{"selector":"0"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"ukuzy0","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://ukuzy0.com","extract_type":"css",
    "item_children":{"base_url":"/index.php/vod/search.html?wd={{keyword}}","item_selector":".xing_vb > ul > li","source":{"selector":".xing_vb4 a","attribute":"href"},"name":{"selector":".xing_vb4 a"},"image":{"selector":"img","attribute":"data-original"},"extra":{"update_time":{"selector":".xing_vb7"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":true,"item_selector":".module-play-list-content > a[href],a[href*='/vod/play'],a[href*='vodplay']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"haohuazy","headers":{"user-agent":"Mozilla/5.0","accept":"application/json, text/plain, */*","referer":"https://haohuazy.com/"},"base_url":"https://haohuazy.com","extract_type":"json",
    "item_children":{"base_url":"/api.php/provide/vod/?ac=list&wd={{keyword}}","item_selector":"list","source":{"selector":"vod_id"},"name":{"selector":"vod_name"},"image":{"selector":"vod_pic"},"extra":{"update_time":{"selector":"vod_time"}},
      "detail":{"base_url":"/api.php/provide/vod/?ac=detail&ids={{source}}","extract_type":"json","item_children":{"item_selector":"list.0.vod_play_url","item_split":{"item_separator":"#","field_separator":"$"},"source":{"selector":"1"},"name":{"selector":"0"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"ryzyw","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://www.ryzyw.com","extract_type":"json",
    "item_children":{"base_url":"/api.php/provide/vod/?ac=list&wd={{keyword}}","item_selector":"list","source":{"selector":"vod_id"},"name":{"selector":"vod_name"},"image":{"selector":"vod_pic"},"extra":{"update_time":{"selector":"vod_time"}},
      "detail":{"base_url":"/api.php/provide/vod/?ac=detail&ids={{source}}","extract_type":"json","item_children":{"item_selector":"list.0.vod_play_url","item_split":{"item_separator":"#","field_separator":"$"},"source":{"selector":"1"},"name":{"selector":"0"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"ffzy5","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://ffzy5.tv","extract_type":"json",
    "item_children":{"base_url":"/api.php/provide/vod/?ac=list&wd={{keyword}}","item_selector":"list","source":{"selector":"vod_id"},"name":{"selector":"vod_name"},"image":{"selector":"vod_pic"},"extra":{"update_time":{"selector":"vod_time"}},
      "detail":{"base_url":"/api.php/provide/vod/?ac=detail&ids={{source}}","extract_type":"json","item_children":{"item_selector":"list.0.vod_play_url","item_split":{"item_separator":"#","field_separator":"$"},"source":{"selector":"1"},"name":{"selector":"0"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"hongniuziyuan","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://hongniuziyuan.net","extract_type":"css",
    "item_children":{"base_url":"/index.php/vod/search.html?wd={{keyword}}","item_selector":".xing_vb > ul > li","source":{"selector":".xing_vb4 a","attribute":"href"},"name":{"selector":".xing_vb4 a"},"image":{"selector":"img","attribute":"data-original"},"extra":{"update_time":{"selector":".xing_vb7"}},
      "detail":{"base_url":"{{source}}","item_children":{"item_selector":".module-play-list-content > a[href],a[href*='/vod/play'],a[href*='vodplay']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  }
]
"###;

pub const VIDEO_RECOMMEND_CONFIG: &str = r###"
[
  {
    "id":"tyyszyapi","headers":{"user-agent":"Mozilla/5.0","accept":"application/json, text/plain, */*","referer":"https://tyyszyapi.com/"},"base_url":"https://tyyszyapi.com","extract_type":"json","category":"推荐",
    "item_children":{"base_url":"/api.php/provide/vod/?ac=list","item_selector":"list","source":{"selector":"vod_id"},"name":{"selector":"vod_name"},"image":{"selector":"vod_pic"},"extra":{"update_time":{"selector":"vod_time"}},
      "detail":{"base_url":"https://tyyszyapi.com/api.php/provide/vod/?ac=detail&ids={{source}}","extract_type":"json","item_children":{"item_selector":"list.0.vod_play_url","item_split":{"item_separator":"#","field_separator":"$"},"source":{"selector":"1"},"name":{"selector":"0"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"keke2","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://www.keke2.app","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".module-item","source":{"selector":"a[href*='/vod/'], a[href*='/detail/'], a[href*='/play/']","attribute":"href"},"name":{"selector":".module-item-title, .video-name a"},"image":{"selector":".module-item-pic img","attribute":"data-src"},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":true,"item_selector":".module-play-list-content > a[href], a[href*='/vod/play'], a[href*='vodplay'], a[href*='/play/']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"gaze","headers":{"user-agent":"Mozilla/5.0","referer":"https://gaze.red/"},"base_url":"https://gaze.red","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":"a[href^='/play/']","source":{"selector":":scope","attribute":"href"},"name":{"selector":"img[alt]","attribute":"alt"},"image":{"selector":"img","attribute":"src"},
      "detail":{"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"novipnoad","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://www.novipnoad.uk","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":"article, .post, .item","source":{"selector":".entry-title a, h2 a, h3 a","attribute":"href"},"name":{"selector":".entry-title, h2, h3"},"image":{"selector":"img","attribute":"src"},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":true,"item_selector":".entry-content a[href], .post-content a[href], article a[href]","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"juok3","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://juok3.top","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".module-item","source":{"selector":".module-item-pic a, a[href*='/vod/'], a[href*='/detail/'], a[href*='/play/']","attribute":"href"},"name":{"selector":".module-item-title, .video-name a"},"image":{"selector":".module-item-pic img","attribute":"data-src"},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":true,"item_selector":".module-play-list-content > a[href], a[href*='/vod/play'], a[href*='vodplay'], a[href*='/play/']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"zlys9","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://zlys9.top","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".module-item","source":{"selector":".module-item-pic a, a[href*='/vod/'], a[href*='/detail/'], a[href*='/play/']","attribute":"href"},"name":{"selector":".module-item-title, .video-name a"},"image":{"selector":".module-item-pic img","attribute":"data-src"},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":true,"item_selector":".module-play-list-content > a[href], a[href*='/vod/play'], a[href*='vodplay'], a[href*='/play/']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"libvio","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://www.libvio.io","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".module-item","source":{"selector":".module-item-pic a, a[href*='/vod/'], a[href*='/detail/'], a[href*='/play/']","attribute":"href"},"name":{"selector":".module-item-title, .video-name a"},"image":{"selector":".module-item-pic img","attribute":"data-src"},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":true,"item_selector":".module-play-list-content > a[href], a[href*='/vod/play'], a[href*='vodplay'], a[href*='/play/']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  },
  {
    "id":"cz4k","headers":{"user-agent":"Mozilla/5.0"},"base_url":"https://www.cz4k.com","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".module-item","source":{"selector":".module-item-pic a, a[href*='/vod/'], a[href*='/detail/'], a[href*='/play/']","attribute":"href"},"name":{"selector":".module-item-title, .video-name a"},"image":{"selector":".module-item-pic img","attribute":"data-src"},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":true,"item_selector":".module-play-list-content > a[href], a[href*='/vod/play'], a[href*='vodplay'], a[href*='/play/']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope"}},"play":{"base_url":"{{source}}","extract_type":"regex","regex":"https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"}}}
  }
]
"###;
