use super::transcoder::{
    OutputBranches, PlayCoreTranscodeTrim, add_elements, connect_decodebin_pad_added,
    link_elements, make_element, make_filesink, make_uri_decodebin, prepare_output,
    run_transcode_pipeline, sink_pad,
};
use anyhow::{Context, anyhow};
use gpui_kit::http_client::Url;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

pub(crate) fn stage_video_segment(
    input: &Path,
    output: &Path,
    trim: PlayCoreTranscodeTrim,
) -> anyhow::Result<()> {
    prepare_output(output)?;
    let uri = Url::from_file_path(input).map_err(|_| anyhow!("无法转换输入路径"))?;
    let source = make_uri_decodebin(uri.as_str())?;
    let pipeline = gst::Pipeline::new();
    let video_queue = make_element("queue")?;
    let video_convert = make_element("videoconvert")?;
    let video_encoder = make_element("avenc_ffv1")?;
    let audio_queue = make_element("queue")?;
    let audio_convert = make_element("audioconvert")?;
    let audio_caps = make_element("capsfilter")?;
    audio_caps.set_property(
        "caps",
        gst::Caps::builder("audio/x-raw")
            .field("format", "F32LE")
            .field("layout", "interleaved")
            .build(),
    );
    let mux = make_element("matroskamux")?;
    let sink = make_filesink(output)?;
    add_elements(
        &pipeline,
        &[
            &video_queue,
            &video_convert,
            &video_encoder,
            &audio_queue,
            &audio_convert,
            &audio_caps,
            &mux,
            &sink,
        ],
    )?;
    link_elements(&[&video_queue, &video_convert, &video_encoder])?;
    link_elements(&[&audio_queue, &audio_convert, &audio_caps])?;
    mux.link(&sink)?;
    connect_decodebin_pad_added(
        &source,
        OutputBranches {
            video_sink: Some(sink_pad(&video_queue)?),
            audio_sink: Some(sink_pad(&audio_queue)?),
            optional_branches: vec![
                vec![video_queue, video_convert, video_encoder],
                vec![audio_queue, audio_convert, audio_caps],
            ],
            mux: Some(mux),
        },
    );
    pipeline.add(&source)?;
    run_transcode_pipeline(&pipeline, &source, Some(trim))
}

pub(crate) fn make_joined_source(paths: &[PathBuf]) -> anyhow::Result<gst::Element> {
    let bin = gst::Bin::new();
    let video_concat = make_element("concat")?;
    let audio_concat = make_element("concat")?;
    let video_output = make_element("identity")?;
    let audio_output = make_element("identity")?;
    video_output.set_property("single-segment", true);
    audio_output.set_property("single-segment", true);
    bin.add_many([&video_concat, &audio_concat, &video_output, &audio_output])?;
    video_concat.link(&video_output)?;
    audio_concat.link(&audio_output)?;
    let completed = Arc::new(AtomicUsize::new(0));
    let exposed = Arc::new(Mutex::new((false, false)));
    for path in paths {
        let uri = Url::from_file_path(path)
            .map_err(|_| anyhow!("无法转换片段路径: {}", path.display()))?;
        let source = make_uri_decodebin(uri.as_str())?;
        let video_queue = make_element("queue")?;
        let audio_queue = make_element("queue")?;
        bin.add_many([&source, &video_queue, &audio_queue])?;
        let video_pad = video_concat
            .request_pad_simple("sink_%u")
            .context("无法创建视频拼接输入")?;
        let audio_pad = audio_concat
            .request_pad_simple("sink_%u")
            .context("无法创建音频拼接输入")?;
        video_queue.static_pad("src").unwrap().link(&video_pad)?;
        audio_queue.static_pad("src").unwrap().link(&audio_pad)?;
        let bin_weak = bin.downgrade();
        let output_video = video_output.clone();
        let output_audio = audio_output.clone();
        let exposed = exposed.clone();
        source.connect_pad_added(move |source, pad| {
            let result = (|| -> anyhow::Result<()> {
                let Some(bin) = bin_weak.upgrade() else {
                    return Ok(());
                };
                let caps = pad.current_caps().unwrap_or_else(|| pad.query_caps(None));
                let name = caps
                    .structure(0)
                    .map(|caps| caps.name().as_str())
                    .unwrap_or("");
                let (queue, concat, ghost_name, video) = if name.starts_with("video/") {
                    (&video_queue, &output_video, "video", true)
                } else if name.starts_with("audio/") {
                    (&audio_queue, &output_audio, "audio", false)
                } else {
                    return Ok(());
                };
                let target = sink_pad(queue)?;
                if target.is_linked() {
                    return Ok(());
                }
                pad.link(&target)?;
                let mut outputs = exposed.lock().map_err(|_| anyhow!("拼接输出状态已损坏"))?;
                let ready = if video {
                    &mut outputs.0
                } else {
                    &mut outputs.1
                };
                if !*ready {
                    let src = concat.static_pad("src").context("拼接输出缺少 src pad")?;
                    let ghost = gst::GhostPad::builder_with_target(&src)?
                        .name(ghost_name)
                        .build();
                    ghost.set_active(true)?;
                    bin.add_pad(&ghost)?;
                    *ready = true;
                }
                drop(outputs);
                Ok(())
            })();
            if let Err(error) = result {
                log::error!("[gst:timeline-export] joining source failed: {error:#}");
                source.post_error_message(gst::error_msg!(
                    gst::CoreError::Negotiation,
                    ["拼接片段失败: {error:#}"]
                ));
            }
        });
        let bin_weak = bin.downgrade();
        let done = completed.clone();
        let total = paths.len();
        source.connect_no_more_pads(move |_| {
            if done.fetch_add(1, Ordering::AcqRel) + 1 == total {
                if let Some(bin) = bin_weak.upgrade() {
                    bin.no_more_pads();
                }
            }
        });
    }
    Ok(bin.upcast())
}
