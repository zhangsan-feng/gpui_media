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
  },
  {
    "id":"hongniuzy","headers":{"user-agent":"Mozilla/5.0","referer":"https://hongniuzy.net/"},"base_url":"https://hongniuzy.net","extract_type":"css",
    "item_children":{"base_url":"/index.php/vod/search.html?wd={{keyword}}","item_selector":".xing_vb > ul > li","source":{"selector":".xing_vb4 a","attribute":"href"},"name":{"selector":".xing_vb4 a"},"extra":{"update_time":{"selector":".xing_vb7"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":".vodplayinfo li","source":{"selector":"input[name='copy_sel']","attribute":"value"},"name":{"selector":":scope"}}}}
  },
  {
    "id":"ryzytv","headers":{"user-agent":"Mozilla/5.0","referer":"http://ryzy.tv/"},"base_url":"http://ryzy.tv","extract_type":"css",
    "item_children":{"base_url":"/index.php/vod/search.html?wd={{keyword}}","item_selector":".videoContent > li","source":{"selector":".videoName","attribute":"href"},"name":{"selector":".videoName"},"extra":{"update_time":{"selector":".time1"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":".playlist a[href*='.m3u8']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope","attribute":"title"}}}}
  }
]
"###;

pub const VIDEO_RECOMMEND_CONFIG: &str = r###"
[
  {
    "id":"suonizy","headers":{"user-agent":"Mozilla/5.0","accept":"text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8","referer":"https://suonizy.net/"},"base_url":"https://suonizy.net","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".tab-box table tbody tr","source":{"selector":".title a","attribute":"href"},"name":{"selector":".title a"},"extra":{"update_time":{"selector":".table-right"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":"li.sele-copy-item","source":{"selector":"input.sele-ckeck","attribute":"value"},"name":{"selector":"a"}}}}
  },
  {
    "id":"wujinzy","headers":{"user-agent":"Mozilla/5.0","referer":"https://www.wujinzy.net/"},"base_url":"https://www.wujinzy.net","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".xing_vb > ul > li","source":{"selector":".xing_vb4 a","attribute":"href"},"name":{"selector":".xing_vb4 a"},"extra":{"update_time":{"selector":".xing_vb7"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":".vodplayinfo li","source":{"selector":"input[name='copy_sel']","attribute":"value"},"name":{"selector":":scope"}}}}
  },
  {
    "id":"jinyingzy","headers":{"user-agent":"Mozilla/5.0","referer":"https://jinyingzy.com/"},"base_url":"https://jinyingzy.com","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".xing_vb > ul > li","source":{"selector":".xing_vb4 a","attribute":"href"},"name":{"selector":".xing_vb4 a"},"extra":{"update_time":{"selector":".xing_vb7"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":"a[href*='.m3u8']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope","attribute":"title"}}}}
  },
  {
    "id":"apibdzy","headers":{"user-agent":"Mozilla/5.0","referer":"https://api.apibdzy.com/"},"base_url":"https://api.apibdzy.com","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".stui-vodlist > li","source":{"selector":"h3.title a","attribute":"href"},"name":{"selector":"h3.title a"},"extra":{"update_time":{"selector":".time"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":".stui-content__playlist li","source":{"selector":".copy_text span"},"name":{"selector":".copy_text"}}}}
  },
  {
    "id":"yayazy2","headers":{"user-agent":"Mozilla/5.0","referer":"https://yayazy2.com/"},"base_url":"https://yayazy2.com","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".stui-vodlist > li","source":{"selector":"h3.title a","attribute":"href"},"name":{"selector":"h3.title a"},"extra":{"update_time":{"selector":".time"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":".stui-content__playlist li","source":{"selector":".copy_text span"},"name":{"selector":".copy_text"}}}}
  },
  {
    "id":"okzyw","headers":{"user-agent":"Mozilla/5.0","referer":"https://okzyw.cc/"},"base_url":"https://okzyw.cc","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":"#video-list > a.item","source":{"selector":":scope","attribute":"href"},"name":{"selector":".title"},"extra":{"update_time":{"selector":".date"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":".item .link","source":{"selector":":scope"},"name":{"selector":":scope"}}}}
  },
  {
    "id":"kuaichezy","headers":{"user-agent":"Mozilla/5.0","referer":"https://kuaichezy.com/"},"base_url":"https://kuaichezy.com","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".videoContent > li","source":{"selector":".videoName","attribute":"href"},"name":{"selector":".videoName"},"extra":{"update_time":{"selector":".time1"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":"a[href*='.m3u8']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope","attribute":"title"}}}}
  },
  {
    "id":"mtzy5","headers":{"user-agent":"Mozilla/5.0","referer":"https://mtzy5.com/"},"base_url":"https://mtzy5.com","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":".movie tbody tr","source":{"selector":"td a","attribute":"href"},"name":{"selector":"td a"},"extra":{"update_time":{"selector":"td:nth-child(5)"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":"a.link[href*='.m3u8']","source":{"selector":":scope","attribute":"href"},"name":{"selector":":scope","attribute":"title"}}}}
  },
  {
    "id":"tyyszyapi","headers":{"user-agent":"Mozilla/5.0","accept":"text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8","accept-language":"zh-CN,zh;q=0.9,en;q=0.8","referer":"https://tyyszyapi.com/"},"base_url":"https://tyyszyapi.com","extract_type":"css","category":"推荐",
    "item_children":{"base_url":"/","item_selector":"a.movie-card[href*='/index.php/vod/detail/']","source":{"selector":":scope","attribute":"href"},"name":{"selector":".movie-name span:first-child"},"image":{"selector":"img[data-src]","attribute":"data-src"},"extra":{"update_time":{"selector":".update-time"}},
      "detail":{"base_url":"{{source}}","item_children":{"fallback_play_links":false,"item_selector":"tr[data-url][data-name]","source":{"selector":"a.ep-link"},"name":{"selector":":scope","attribute":"data-name"}}}}
  }
]
"###;
