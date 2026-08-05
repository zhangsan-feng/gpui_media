pub const VIDEO_SEARCH_CONFIG: &str = r###"
[
  {
    "id": "ukuzy0",
    "resource_type": "video",
    "extract_type": "css",
    "headers": {
      "user-agent": "Mozilla/5.0"
    },
    "search": {
      "url": "https://ukuzy0.com/index.php/vod/search.html?wd={{keyword}}",
      "category": "ukuzy0",
      "item_selector": ".xing_vb > ul > li",
      "name": {
        "selector": ".xing_vb4 a",
        "attribute": null
      },
      "image": {
        "selector": "img",
        "attribute": "data-original"
      },
      "detail_url": {
        "selector": ".xing_vb4 a",
        "attribute": "href"
      },
      "extra": {
        "update_time": {
          "selector": ".xing_vb7",
          "attribute": null
        }
      },
      "children": {
        "item_selector": ".module-play-list-content > a[href],a[href*='/vod/play'],a[href*='vodplay']",
        "name": {
          "selector": ":scope",
          "attribute": null
        },
        "image": null,
        "play_url": {
          "selector": ":scope",
          "attribute": "href"
        }
      }
    },
    "recommend": [],
    "play_regex": "https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"
  },
  {
    "id": "jszy333",
    "resource_type": "video",
    "extract_type": "css",
    "headers": {
      "user-agent": "Mozilla/5.0"
    },
    "search": {
      "url": "https://jszy333.com/index.php/vod/search.html?wd={{keyword}}",
      "category": "jszy333",
      "item_selector": ".xing_vb > ul > li",
      "name": {
        "selector": ".xing_vb4 a",
        "attribute": null
      },
      "image": {
        "selector": "img",
        "attribute": "data-original"
      },
      "detail_url": {
        "selector": ".xing_vb4 a",
        "attribute": "href"
      },
      "extra": {
        "update_time": {
          "selector": ".xing_vb7",
          "attribute": null
        }
      },
      "children": {
        "item_selector": ".module-play-list-content > a[href],a[href*='/vod/play'],a[href*='vodplay']",
        "name": {
          "selector": ":scope",
          "attribute": null
        },
        "image": null,
        "play_url": {
          "selector": ":scope",
          "attribute": "href"
        }
      }
    },
    "recommend": [],
    "play_regex": "https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"
  },
  {
    "id": "haohuazy",
    "resource_type": "video",
    "extract_type": "css",
    "headers": {
      "user-agent": "Mozilla/5.0"
    },
    "search": {
      "url": "https://haohuazy.com/index.php/vod/search.html?wd={{keyword}}",
      "category": "haohuazy",
      "item_selector": ".xing_vb > ul > li",
      "name": {
        "selector": ".xing_vb4 a",
        "attribute": null
      },
      "image": {
        "selector": "img",
        "attribute": "data-original"
      },
      "detail_url": {
        "selector": ".xing_vb4 a",
        "attribute": "href"
      },
      "extra": {
        "update_time": {
          "selector": ".xing_vb7",
          "attribute": null
        }
      },
      "children": {
        "item_selector": ".module-play-list-content > a[href],a[href*='/vod/play'],a[href*='vodplay']",
        "name": {
          "selector": ":scope",
          "attribute": null
        },
        "image": null,
        "play_url": {
          "selector": ":scope",
          "attribute": "href"
        }
      }
    },
    "recommend": [],
    "play_regex": "https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"
  },
  {
    "id": "ryzyw",
    "resource_type": "video",
    "extract_type": "css",
    "headers": {
      "user-agent": "Mozilla/5.0"
    },
    "search": {
      "url": "https://www.ryzyw.com/index.php/vod/search.html?wd={{keyword}}",
      "category": "ryzyw",
      "item_selector": ".xing_vb > ul > li",
      "name": {
        "selector": ".xing_vb4 a",
        "attribute": null
      },
      "image": {
        "selector": "img",
        "attribute": "data-original"
      },
      "detail_url": {
        "selector": ".xing_vb4 a",
        "attribute": "href"
      },
      "extra": {
        "update_time": {
          "selector": ".xing_vb7",
          "attribute": null
        }
      },
      "children": {
        "item_selector": ".module-play-list-content > a[href],a[href*='/vod/play'],a[href*='vodplay']",
        "name": {
          "selector": ":scope",
          "attribute": null
        },
        "image": null,
        "play_url": {
          "selector": ":scope",
          "attribute": "href"
        }
      }
    },
    "recommend": [],
    "play_regex": "https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"
  },
  {
    "id": "ffzy5",
    "resource_type": "video",
    "extract_type": "css",
    "headers": {
      "user-agent": "Mozilla/5.0"
    },
    "search": {
      "url": "https://ffzy5.tv/index.php/vod/search.html?wd={{keyword}}",
      "category": "ffzy5",
      "item_selector": ".xing_vb > ul > li",
      "name": {
        "selector": ".xing_vb4 a",
        "attribute": null
      },
      "image": {
        "selector": "img",
        "attribute": "data-original"
      },
      "detail_url": {
        "selector": ".xing_vb4 a",
        "attribute": "href"
      },
      "extra": {
        "update_time": {
          "selector": ".xing_vb7",
          "attribute": null
        }
      },
      "children": {
        "item_selector": ".module-play-list-content > a[href],a[href*='/vod/play'],a[href*='vodplay']",
        "name": {
          "selector": ":scope",
          "attribute": null
        },
        "image": null,
        "play_url": {
          "selector": ":scope",
          "attribute": "href"
        }
      }
    },
    "recommend": [],
    "play_regex": "https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"
  },
  {
    "id": "hongniuziyuan",
    "resource_type": "video",
    "extract_type": "css",
    "headers": {
      "user-agent": "Mozilla/5.0"
    },
    "search": {
      "url": "https://hongniuziyuan.net/index.php/vod/search.html?wd={{keyword}}",
      "category": "hongniuziyuan",
      "item_selector": ".xing_vb > ul > li",
      "name": {
        "selector": ".xing_vb4 a",
        "attribute": null
      },
      "image": {
        "selector": "img",
        "attribute": "data-original"
      },
      "detail_url": {
        "selector": ".xing_vb4 a",
        "attribute": "href"
      },
      "extra": {
        "update_time": {
          "selector": ".xing_vb7",
          "attribute": null
        }
      },
      "children": {
        "item_selector": ".module-play-list-content > a[href],a[href*='/vod/play'],a[href*='vodplay']",
        "name": {
          "selector": ":scope",
          "attribute": null
        },
        "image": null,
        "play_url": {
          "selector": ":scope",
          "attribute": "href"
        }
      }
    },
    "recommend": [],
    "play_regex": "https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"
  }
]
"###;

pub const VIDEO_RECOMMEND_CONFIG: &str = r###"
[
  {
    "id": "youzisp",
    "resource_type": "video",
    "extract_type": "css",
    "headers": {
      "user-agent": "Mozilla/5.0"
    },
    "search": null,
    "recommend": [
      {
        "url": "https://youzisp.tv/vodshow/dianying-----------.html",
        "category": "电影",
        "item_selector": "a.module-item[href]",
        "name": {
          "selector": ":scope",
          "attribute": "title"
        },
        "image": {
          "selector": ".module-item-pic img",
          "attribute": "data-original"
        },
        "detail_url": {
          "selector": ":scope",
          "attribute": "href"
        },
        "children": {
          "item_selector": ".module-play-list-content > a[href]",
          "name": {
            "selector": ":scope",
            "attribute": null
          },
          "image": null,
          "play_url": {
            "selector": ":scope",
            "attribute": "href"
          }
        }
      }
    ],
    "play_regex": "https?://[^\\s\"'<>]+\\.m3u8[^\\s\"'<>]*"
  },
  {
    "id": "renren",
    "resource_type": "video",
    "extract_type": "css",
    "headers": {
      "user-agent": "Mozilla/5.0"
    },
    "search": null,
    "recommend": [
      {
        "url": "https://www.renren.pro",
        "category": "推荐",
        "item_selector": ".module-item",
        "name": {
          "selector": ".module-item-title, .video-name a",
          "attribute": null
        },
        "image": {
          "selector": ".module-item-pic img",
          "attribute": "data-src"
        },
        "detail_url": {
          "selector": "a[href*='/play/']",
          "attribute": "href"
        },
        "children": {
          "item_selector": ".module-blocklist a[href]",
          "name": {
            "selector": ":scope",
            "attribute": null
          },
          "image": null,
          "play_url": {
            "selector": ":scope",
            "attribute": "href"
          }
        }
      }
    ],
    "play_regex": "url:\\s*[\"']([^\"']+\\.m3u8[^\"']*)[\"']"
  }
]
"###;
