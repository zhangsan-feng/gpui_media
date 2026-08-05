pub const AUDIO_RECOMMEND_CONFIG: &str = r###"
[
  {
    "id": "bilibili",
    "resource_type": "audio",
    "extract_type": "json",
    "headers": {
      "user-agent": "Mozilla/5.0",
      "referer": "https://www.bilibili.com/audio/home",
      "origin": "https://www.bilibili.com",
      "accept": "application/json, text/plain, */*"
    },
    "search": null,
    "recommend": [
      {
        "url": "https://www.bilibili.com/audio/music-service-c/web/menu/hit?pn=1&ps=12",
        "category": "热门推荐",
        "item_selector": "data.data",
        "name": {
          "selector": "title",
          "attribute": null
        },
        "author": {
          "selector": "uname",
          "attribute": null
        },
        "image": {
          "selector": "cover",
          "attribute": null
        },
        "detail_url": {
          "selector": "menuId",
          "attribute": null
        },
        "children_url": "https://www.bilibili.com/audio/music-service-c/web/song/of-menu?sid={{id}}&pn=1&ps=20",
        "children": {
          "extract_type": "json",
          "item_selector": "data.data",
          "name": {
            "selector": "title",
            "attribute": null
          },
          "author": {
            "selector": "author",
            "attribute": null
          },
          "image": {
            "selector": "cover",
            "attribute": null
          },
          "play_url": {
            "selector": "id",
            "attribute": null
          }
        }
      },
      {
        "url": "https://www.bilibili.com/audio/music-service-c/web/home/hit-rank",
        "category": "歌曲榜单",
        "item_selector": "data",
        "name": {
          "selector": "title",
          "attribute": null
        },
        "author": {
          "selector": "uname",
          "attribute": null
        },
        "image": {
          "selector": "cover",
          "attribute": null
        },
        "detail_url": {
          "selector": "menuId",
          "attribute": null
        },
        "children_url": null,
        "children": {
          "extract_type": "json",
          "item_selector": "audios",
          "name": {
            "selector": "title",
            "attribute": null
          },
          "author": null,
          "image": null,
          "play_url": {
            "selector": "id",
            "attribute": null
          }
        }
      }
    ],
    "play_regex": null,
    "play_url": "https://www.bilibili.com/audio/music-service-c/web/url?sid={{id}}&privilege=2&quality=1",
    "play_selector": "data.cdns.0"
  }
]
"###;
