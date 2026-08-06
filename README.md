
```aiignore
这是一个基于gpui + gst 实现的 音乐播放器和视频播放器 
支持网络直播 视频 tv  
hls m3u8 flv 等等 能手动输入网络链接 或者本地拖拽 播放 构建在actions 里面 

比如
https://live.zbds.top/tv/iptv4.txt
https://github.com/youhunwl/TVAPP

视频来源 都是网络上的cms 站点 src/plugins/extractor 如果你有更好的资源可以让ai 接入

开发阶段需要 下载gst的c代码环境 
https://gstreamer.freedesktop.org/download/

由于构建需要裁减c 的dll 打包比较麻烦 
windows 构建
    cargo run --release -p build_windows -- doctor
    cargo run --release -p build_windows -- package --force
    cargo run --release -p build_windows -- verify
    cargo run --release -p build_windows -- support


cli 构建 actions
    gh workflow run build-windows.yml --ref master
    
web 构建
    Build Windows Package → 进入工作流详情页 → 右上角点击 Run workflow → 选择 master → 点击绿色 Run workflow。


```
![134432.png](example_img/134432.png)
![134755.png](example_img/134755.png)
![134412.png](example_img/134412.png)
![092252.png](example_img/092252.png)
![135551.png](example_img/135551.png)
