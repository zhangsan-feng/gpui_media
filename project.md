# 项目状态

## 当前进度

- 播放器和媒体编辑器的播放列表改用 `UniformListScrollHandle` / `uniform_list`，固定 44px 行高，避免原 `ListState` 在追加条目、重设列表和宽度变化后丢失高度提示，导致重绘时滑块尺寸和位置变化；播放器保留切换当前项的最近可见滚动策略，两处滚动条均使用布局视口。列表替换记录条目数与行高调试日志。

- 独立播放器播放列表右上角显示当前播放序号/总数（例如 `23/299`），随播放切换和列表增删更新；未选择播放项时显示 `0/总数`，空列表显示 `0/0`。
- 独立播放器切换条目时，播放列表自动滚动以完整显示当前播放项，覆盖点击播放、上一项/下一项和播放结束自动切换；条目已完整可见时保持滚动位置，复用现有播放切换日志。
- 独立播放器 `medio_player` 已订阅播放完成事件，按当前播放列表顺序自动播放下一项，最后一项结束后停止；通过窗口、播放器实体及当前条目 ID/URL 核对事件，删除文件期间跳过自动切换。自动切换和列表播放完成记录 `[medio-player:auto-next]` 日志。
- 已完成 gpui-kit 0.6 依赖迁移。
- 根应用和 `crates/player_core` 统一使用 `gpui_kit` 导出的 GPUI、Component、Assets 和 Platform 类型。
- HTTP client 使用 workspace 中的 `gpui-pre-reqwest-client` 依赖。
- 保留 GStreamer、播放器、转码器、抓取器和 Windows 打包基础设施。
- 保留现有 fern 日志初始化；日志目录为 `logs/`，文件格式为 `logs/YYYY-MM-DD.log`。
- 播放位置采样间隔从 200ms 缩短为 16ms（目标约每秒 60 次），改善媒体编辑器播放头和通用播放器进度条的跳动；仅位置或时长变化时通知界面刷新，进度任务启动日志记录会话和刷新间隔。
- 未新增 GUI 测试，按项目约束以编译和手工启动冒烟验证 GUI。
- 播放器自绘标题栏已接入当前 `PlayStatic.title`，选集播放后显示选集标题。
- 播放器已有视频帧时进入网络缓冲状态会显示“缓冲中”视频内遮罩。
- 首页媒体编辑器左侧素材库支持多文件导入、拖入媒体、选择文件夹和网络链接；文件夹素材按文件名排序，素材库下方显示媒体信息。
- 首页播放列表滚动条对齐音乐播放器配置：16px 宽、常驻显示，并使用布局视口计算。
- 首页播放列表支持右键删除本地磁盘文件；删除前确认，若 GStreamer 正在使用该文件则先停止管线并等待进入 `Null` 状态后再删除。删除失败时保留列表项、显示错误并尝试恢复播放。
- 视频播放器删除确认使用系统标准确定按钮；删除请求、确认返回值、文件删除阶段均写入日志，便于定位没有触发或执行失败的环节。
- 首页媒体编辑器播放结束后停留当前素材并保留管线与最后画面；结束时停止进度轮询并固定最终位置，避免首次重播从末尾前的极短位置启动。音乐和视频页面沿用各自播放完成逻辑。
- 新增独立“播放器”页面，将 Git 中旧 `VideoPlayer` 的播放能力迁移到 `src/gui/home/medio_player`：支持播放/暂停、进度定位、音量、上一首/下一首、播放列表、拖入本地媒体和网络链接播放；播放列表增删与播放切换记录到现有日志，媒体编辑器功能保持不变。
- 独立播放器右侧栏顶部的网络“播放”按钮右侧增加文件夹图标，可选择目录并按文件名顺序追加支持的媒体、播放首项；播放列表右键菜单支持确认后删除本地文件和打开所在文件夹，删除当前播放文件前先释放 GStreamer 管线，失败时保留列表项并显示错误。
- 目录选择、读取失败和空媒体目录通过项目日志记录。
- 剪辑结果、整段和批量导出统一到下方“导出”菜单；批量选择目录后依次处理素材库，保留导出状态和失败日志。

- 原 `src/gui/home/video_player` 更名为英文目录 `src/gui/home/media_editor`，类型和首页入口更名为 `MediaEditor` / “媒体编辑器”。
- 单素材剪辑工作区采用左侧素材库、中间预览、底部剪辑时间线；支持毫秒时间码、播放头定位、逐帧定位、重置及选区播放，已移除独立入点/出点按钮和读数。
- 时间线支持按住左键直接拖选独立高亮区域，包含反向拖选和超出轨道边界时的钳制；松开定位到选区起点。单击定位，播放头可单独拖动。选区左右两侧新增滑动手柄，分别调整开始和结束时间，限制在时间线内且不交叉；调整开始及最终选区写入现有日志。
- 移除时间线底部的常驻操作提示，保留错误信息；顶部“删除选区”按钮移到高亮区域的右键菜单，右键不再取消选区。
- Delete 或右键菜单中的“删除选区”删除所选时间范围，保留两侧并自动闭合空隙；时间线按剪辑后的时间显示连续片段，支持跨已有接缝再次删除。不会删除磁盘素材，重置或重新选择素材可恢复完整来源。
- 删空时间线后保持空状态并禁用预览和剪辑结果导出；重置恢复完整素材。
- 空格、播放按钮和点击预览画面统一使用编辑器播放队列：选区存在时播放选区，否则播放剪辑结果，跳过已删除的来源区间；快捷键排除输入框，保证网络链接正常输入和编辑。
- 网络素材保留播放和整段导出，剪辑结果导出要求本地素材。
- 预览按保留区间依次执行有界 seek，EOS 后继续下一段；暂停定位通过 `new_preroll` 回调刷新画面。
- 裁剪转码向解码源定位，按实际媒体流创建 mux 轨道，移除无音轨视频的空分支，使用 AAC ADTS 文件格式；FLAC 裁剪先输出临时 PCM，再编码 FLAC 并清理临时文件。
- 剪辑结果导出使用全部保留区间，独立于当前高亮选区；音频先按区间输出 PCM，视频先输出 FFV1/PCM 无损中间片段，拼接后仅进行一次最终格式编码。concat 后使用 single-segment identity 统一时间戳，导出结束清理临时文件。
- Windows runtime 必需功能列表补充 `avenc_ffv1` 和 `concat`，复用已有 libav/coreelements 插件。
- 素材导入、区间变化、删除后总时长及剩余片段数、各段预览和导出阶段写入现有 `logs/YYYY-MM-DD.log`。

## 验证结果

- 2026-10-19 播放器和编辑器列表滚动条稳定性：`cargo check -p gpui_medio --locked`、`cargo fmt --all -- --check`、`git diff --check` 通过，编译包含既有警告；编译日志见 `target/playlist-scroll-check.log`。按项目约束未新增 GUI 测试，尚未完成实际界面重绘、侧栏开合与拖动滚动条验证。

- 2026-10-19 播放列表当前序号：`cargo check -p gpui_medio --locked`、`cargo fmt --all -- --check`、`git diff --check` 通过，编译包含既有警告；日志见 `target/medio-player-position-check.log`。未新增 GUI 测试，尚未进行界面手工验证。
- 2026-10-19 播放列表跟随当前播放项：`cargo check -p gpui_medio --locked`、`cargo fmt --all -- --check`、`git diff --check` 通过；编译输出包含既有警告，日志见 `target/medio-player-follow-check.log`。未新增 GUI 测试，尚未进行手工滚动验证。
- 2026-10-19 独立播放器自动播放下一项：`cargo check -p gpui_medio --locked`、`cargo fmt --all -- --check`、`git diff --check` 通过；编译输出包含既有警告，日志见 `target/medio-player-auto-next-check.log`。按项目约束未新增 GUI 测试，尚未进行实际连续播放手工验证。
- 2026-10-13 进度条刷新优化：`cargo check -p gpui_medio --locked`、`cargo fmt --all -- --check`、`git diff --check` 通过，未新增 GUI 测试；实际运行时流畅度尚未手工验证。编译和链接完成后，因运行中的 `target/debug/gpui-medio.exe` 无法覆盖，Cargo 最后复制失败；新链接产物另存为 `target/debug/gpui-medio-progress.exe`，SHA256 与链接产物一致。检查和构建日志见 `target/media-editor-progress-check.log`、`target/media-editor-progress-build.log`。
- 2026-10-13 选区手柄与右键删除菜单：`cargo check -p gpui_medio --locked`、`cargo fmt --all -- --check`、`git diff --check` 通过，未新增 GUI 测试；GUI 拖动与右键菜单未进行手工交互验证。
- 本次 `cargo build --bin gpui-medio --locked` 已完成编译和链接；因运行中的 `target/debug/gpui-medio.exe` 被占用，Cargo 最后复制产物失败。已将新链接的 `target/debug/deps/gpui_medio.exe` 另存为 `target/debug/gpui-medio-selection.exe`，SHA256 校验与链接产物一致。构建包含既有 31 项警告，日志见 `target/media-editor-selection-build.log`。
- 最新 `cargo fmt --all -- --check`：通过。
- 独立播放器页面：`cargo check -p gpui_medio --locked`、`cargo fmt --all -- --check`、`git diff --check` 通过；未新增 GUI 测试。应用编译输出包含既有模块的警告。
- 独立播放器目录导入与播放列表右键菜单：`cargo check -p gpui_medio --locked`、`cargo fmt --all -- --check`、`git diff --check` 通过；未新增 GUI 测试，尚未进行手工界面操作验证。
- `cargo check --workspace --locked`：通过。
- 视频播放器目录选择与侧栏 2:1 布局调整：`cargo fmt --all -- --check`、`cargo check --workspace --locked` 均通过；未新增 GUI 测试。
- 播放列表滚动条按音乐播放器配置调整：`cargo fmt --all -- --check`、`cargo check --workspace --locked` 均通过；未新增 GUI 测试。
- 播放完成自动下一项：`cargo fmt --all -- --check`、`cargo check --workspace --locked` 均通过；未新增 GUI 测试。
- 播放列表右键删除与 GStreamer 文件释放：`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`cargo check -p player_core --locked`、`git diff --check` 均通过；未新增 GUI 测试。
- 播放列表删除确认按钮与流程日志：`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 均通过；未新增 GUI 测试，尚未进行 GUI 手工删除验证。
- 视频播放器单独/批量导出：`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 均通过；未新增 GUI 测试。
- `cargo check -p player_core --locked`：通过。
- `cargo check -p build_windows --locked`：通过。
- `cargo tree -i gpui-pre`：统一为 `gpui-pre 0.3.4`。
- `cargo tree -i gpui-pre-platform`：统一为 `gpui-pre-platform 0.3.4`。
- 旧 Zed GPUI 和旧 `longbridge/gpui-component` Git 源检索：无结果。
- 早期 `cargo run --locked` 链接缺少 `gstvideo-1.0.lib`；已定位本机 GStreamer SDK，后续设置 `GSTREAMER_1_0_ROOT_MSVC_X86_64` 后完整构建通过。

- 媒体编辑器最终源码：`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 通过；未新增 GUI 测试。
- 媒体编辑器播放完成后首次重播位置：`cargo check -p player_core --locked`、`rustfmt --check --edition 2024 crates/player_core/src/core/player_event.rs`、`git diff --check` 通过；当时全仓格式检查被播放器迁移文件的格式差异阻断，随后在独立播放器页面迁移中格式化并通过；未新增或运行 GUI 测试。
- 拖选删除并拼接：`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 通过；未新增 GUI 测试，最终拖选、右键、空格及跨接缝预览尚未进行 GUI 手工确认。
- `cargo test -p player_core -p gpui_medio --locked`：通过，根应用 3 项现有测试和播放器 crate 5 项非 GUI 剪辑模型测试通过。新增测试覆盖删除中间区间、跨接缝重复删除、选区映射、删空和无效区间；已验证先失败后通过。
- 临时 CLI 验证实际生成并检查 10 份裁剪文件：WAV、FLAC、MP3、AAC、无音轨视频 MP4/MKV/MOV，以及同时含音视频的 MP4/MKV/MOV。`ffprobe` 均识别正常；1–3 秒选区对应约 2 秒成品，MP3 1.976408 秒、AAC 1.984 秒，其余约 2 秒。验证代码已清理，结果位于 `target/media-editor-export-verified/verification.json`。
- 拼接导出实际检查 11 份文件：删除 4 秒来源的 1–3 秒，导出 0–1 秒和 3–4 秒保留内容，覆盖上述 10 种音视频组合及额外 WAV 音调样本。`ffprobe` 均识别正常；WAV/FLAC 为 2 秒，MP3 1.997774 秒、AAC 1.984 秒，视频约 2.000667 秒且均为 60 帧，音视频流类型正确。结果位于 `target/media-editor-joined-smoke/verification.json`。
- WAV 音调验证：原素材依次为 440/660/660/880 Hz，删除中间两秒后检测到前段 440 Hz、后段 880 Hz，总时长 2 秒，确认删除范围及拼接顺序正确。结果位于 `target/media-editor-joined-smoke/tone-verification.json`；临时 Rust 验证入口已清理。
- 应用启动曾成功初始化 GStreamer 和 Direct3D，并记录素材导入与时间线操作日志；本轮 GUI 交互验证状态见上。
- 最新 `cargo build --bin gpui-medio --locked`：通过，已生成包含区间删除和拼接导出的 `target/debug/gpui-medio.exe`。日志：`target/media-editor-build.log`。
- `cargo test --workspace --locked`：复现既有 `build_windows` 7 项通过、5 项失败；本次该目录仅补充 runtime 所需功能，未改测试或基础设施源码。失败分别为 `package::tests::manifest_rejects_platform_without_a_dependency_inspector`、`package::tests::staging_copies_dynamic_modules_and_their_transitive_dependencies`、`package::tests::staging_separates_plugins_from_runtime_dependencies`、`runtime::tests::validation_reports_a_missing_dynamic_runtime_module`、`runtime::tests::validation_reports_the_exact_missing_runtime_file`；输出涉及测试夹具 TOML 缺少 `platform` 和错误信息断言。日志：`target/media-editor-workspace-tests.log`。

## 目录与文件职责

```text
项目根目录/
    Cargo.toml  workspace、应用依赖和 gpui-kit 统一入口
    Cargo.lock  Cargo 依赖锁定结果，沿用既有忽略配置
    build.rs  Windows GStreamer 链接路径配置
    project.md  项目状态、验证结果与目录职责
src/
    main.rs  日志初始化、GStreamer 初始化、HTTP client、资源合并及主窗口启动
    com/
        request.rs  网络请求封装和文件下载
    drive/
        mod.rs  网络媒体类型与下载接口
    component/
        window.rs  通用窗口配置
        window_title_bar.rs  自绘标题栏
        draggable_list.rs  拖拽列表组件
        color.rs  颜色工具
    icon/
        scissors.svg  媒体编辑器使用的 Lucide 剪刀图标，无新增图标依赖
    gui/
        home/
            mod.rs  首页布局、页面初始化与 Render 入口
            sidebar_menu.rs  音乐、视频、媒体编辑器和播放器导航
            medio_player/
                mod.rs  播放器类型、等高列表滚动句柄、初始化、播放完成事件订阅与 Render 入口
                core.rs  目录导入、URL 播放、前后切换、播放完成自动下一项、列表滚动跟随当前播放项、右键文件删除/打开目录及侧栏状态
                external.rs  对外播放器实体查询及播放列表设置/追加接口
                ui.rs  视频帧、播放控制、目录按钮、进度/音量、44px 等高播放列表、当前序号/总数及右键菜单和媒体信息渲染
            media_editor/
                mod.rs  类型、状态、快捷键、初始化、订阅与 Render 入口
                external.rs  对外素材列表接口及播放器实体查询
                core/
                    mod.rs  素材播放、文件夹扫描、侧栏状态和磁盘文件删除
                    editing.rs  素材导入、编辑时间定位、拖选与左右边界调整、删除选区、重置及跨来源区间预览队列
                    export.rs  剪辑拼接结果、整段和批量导出，格式、路径及进度状态
                ui/
                    mod.rs  工作区布局、预览和播放控制渲染
                    timeline.rs  剪辑后时间码、相邻保留片段、选区高亮与左右滑动手柄、右键删除菜单、播放头和重置/播放工具栏渲染
                    export.rs  底部统一导出菜单渲染
                    side_panel.rs  素材库、44px 等高虚拟列表与稳定滚动条、右键磁盘文件删除菜单及媒体信息渲染
        music_page/
            mod.rs  音乐页面状态与播放交互
        video_page/
            mod.rs  视频页面状态与播放交互
            core.rs  视频页面核心数据流程
    plugins/
        extractor/
            video/play.rs  视频播放资源请求和 URL 解析
            audio/  音频平台播放、推荐和搜索
            config/  平台配置、请求和资源解析配置
            template/  抓取模板与响应解析
crates/
    player_core/
        Cargo.toml  播放核心依赖入口
        src/
            lib.rs  播放核心公开类型与接口导出
            external.rs  对外画面/进度/媒体状态、无内部播放手势预览及播放结束保留选项
            state.rs  播放器全局状态、事件和订阅类型
            core/
                mod.rs  播放器类型、状态、初始化和 Render 入口
                player_control.rs  播放、暂停、定位和区间播放
                player_pipeline.rs  GStreamer 管线、普通帧与暂停 preroll 帧缓存
                player_event.rs  总线事件、缓冲、EOS 与编辑区间结束处理
                player_task.rs  16ms 进度采样、仅变化时刷新界面、视频帧刷新和加载任务
                player_runtime.rs  管线、任务与图像运行时类型
            internal/
                control.rs  内部拖拽、播放控制及删除前释放管线
            editing/
                mod.rs  保留来源区间模型与模块声明
                core.rs  删除逻辑时间范围、闭合空隙及来源/编辑时间映射
                external.rs  对外时间线创建、删除、区间和时长查询接口
                tests.rs  非 GUI 区间删除和接缝映射验证
            export/
                mod.rs  导出子模块和多片段拼接请求类型
                export.rs  裁剪请求校验、裁剪转码、FLAC 临时 PCM 导出与清理
                timeline.rs  多个保留区间的无损暂存、拼接导出与临时文件清理
            transcoder/
                transcoder.rs  解码、输出分支、按实际流创建轨道及离线/实时转码
                timeline.rs  FFV1/PCM 视频区间暂存、concat 输入顺序及统一时间戳输出
            ui/
                frame.rs  通用预览画面及可选内部播放手势
                controls.rs  播放核心的通用控制条、进度和音量渲染
    build_windows/
        gstreamer-runtime.toml  Windows 插件清单与转码/拼接功能要求
        src/runtime.rs  Windows GStreamer runtime 校验与环境配置
        src/package.rs  Windows runtime 收集与打包
logs/
    YYYY-MM-DD.log  应用日志，优先用于定位播放与导出问题
target/
    debug/gpui-medio-progress.exe  包含选区手柄与右键菜单、16ms 进度条刷新优化的已链接程序
    media-editor-progress-check.log  进度条刷新优化的编译检查日志
    media-editor-progress-build.log  进度条刷新优化的构建与产物复制失败日志
    debug/gpui-medio-selection.exe  本次选区手柄与右键菜单修改后的已链接程序
    media-editor-selection-check.log  本次编辑器交互修改的编译检查日志
    media-editor-selection-build.log  本次完整构建与产物复制失败日志
    media-editor-export-verified/verification.json  本次实际裁剪产物时长与流类型验证记录
    media-editor-joined-smoke/verification.json  拼接产物时长、媒体流和视频帧数验证记录
    media-editor-joined-smoke/tone-verification.json  删除中间内容与保留两侧顺序的音调验证
    media-editor-tests.log  应用现有测试和 player_core 剪辑模型测试输出
    media-editor-workspace-tests.log  工作区完整测试与既有打包测试失败记录
    media-editor-build.log  最新完整构建输出
```
