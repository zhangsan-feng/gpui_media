use super::transcoder::PlayCoreTranscodeFormat;
use crate::PlayCoreFilterState;
use anyhow::{Context as AnyhowContext, bail};
use gpui::http_client::Url;
use gstreamer as gst;
use gstreamer::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayCoreExportTrim {
    pub start: Duration,
    pub end: Duration,
}

#[derive(Clone, Debug)]
pub struct PlayCoreExportRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub format: PlayCoreTranscodeFormat,
    pub trim: Option<PlayCoreExportTrim>,
    pub filters: Option<PlayCoreFilterState>,
}

pub struct PlayCoreExport;

pub(crate) struct OutputBranches {
    video_sink: Option<gst::Pad>,
    audio_sink: Option<gst::Pad>,
}

impl PlayCoreExport {
    pub async fn export(request: PlayCoreExportRequest) -> anyhow::Result<()> {
        tokio::task::spawn_blocking(move || Self::export_blocking(request))
            .await
            .context("导出任务异常")?
    }

    pub fn export_blocking(request: PlayCoreExportRequest) -> anyhow::Result<()> {
        gst::init().context("初始化 GStreamer 失败")?;
        validate_request(&request)?;
        prepare_output(&request.output)?;

        let input_uri = Url::from_file_path(&request.input)
            .map_err(|_| anyhow::anyhow!("无法转换输入文件路径: {}", request.input.display()))?
            .to_string();
        let pipeline = gst::Pipeline::new();
        let source = make_element("uridecodebin")?;
        source.set_property("uri", &input_uri);

        let branches = build_output_branches(
            &pipeline,
            &request.output,
            request.format,
            request.filters.as_ref(),
        )?;
        connect_decodebin_pad_added(&source, branches);
        pipeline.add(&source)?;
        run_export_pipeline(&pipeline, request.trim)
    }
}

fn validate_request(request: &PlayCoreExportRequest) -> anyhow::Result<()> {
    if !request.input.is_file() {
        bail!("输入文件不存在: {}", request.input.display());
    }
    if let Some(trim) = request.trim {
        if trim.start >= trim.end {
            bail!("导出裁剪区间无效: start 必须小于 end");
        }
    }
    Ok(())
}

pub(crate) fn prepare_output(output: &Path) -> anyhow::Result<()> {
    if output.as_os_str().is_empty() {
        bail!("输出路径为空");
    }
    if output.exists() {
        bail!("输出文件已存在: {}", output.display());
    }
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建输出目录失败: {}", parent.display()))?;
        }
    }
    Ok(())
}

fn make_element(factory: &str) -> anyhow::Result<gst::Element> {
    gst::ElementFactory::make(factory)
        .build()
        .with_context(|| format!("GStreamer 元素不可用: {factory}"))
}

pub(crate) fn connect_source_setup(source: &gst::Element, mut headers: HeaderMap) {
    if !headers.contains_key(USER_AGENT) {
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36",
            ),
        );
    }

    source.connect("source-setup", false, move |values| {
        let Some(source) = values
            .get(1)
            .and_then(|value| value.get::<gst::Element>().ok())
        else {
            return None;
        };

        let has_extra_headers = source.find_property("extra-headers").is_some();
        let has_user_agent = source.find_property("user-agent").is_some();
        if !has_extra_headers && !has_user_agent {
            return None;
        }

        let mut extra_headers = gst::Structure::builder("extra-headers");
        let mut header_count = 0;
        for (name, value) in &headers {
            let Ok(value) = value.to_str() else {
                continue;
            };
            if name == USER_AGENT {
                if has_user_agent {
                    source.set_property("user-agent", value);
                }
                continue;
            }
            if has_extra_headers {
                extra_headers = extra_headers.field(name.as_str(), value.to_owned());
                header_count += 1;
            }
        }

        if has_extra_headers && header_count > 0 {
            source.set_property("extra-headers", extra_headers.build());
        }
        None
    });
}

fn make_filesink(output: &Path) -> anyhow::Result<gst::Element> {
    let sink = make_element("filesink")?;
    sink.set_property("location", output.to_string_lossy().into_owned());
    Ok(sink)
}

fn add_elements(pipeline: &gst::Pipeline, elements: &[&gst::Element]) -> anyhow::Result<()> {
    for element in elements {
        pipeline.add(*element)?;
    }
    Ok(())
}

fn link_elements(elements: &[&gst::Element]) -> anyhow::Result<()> {
    for pair in elements.windows(2) {
        pair[0].link(pair[1])?;
    }
    Ok(())
}

fn sink_pad(element: &gst::Element) -> anyhow::Result<gst::Pad> {
    element.static_pad("sink").context("导出分支缺少 sink pad")
}

fn create_video_filter(
    filters: Option<&PlayCoreFilterState>,
) -> anyhow::Result<Option<gst::Element>> {
    let Some(filters) = filters else {
        return Ok(None);
    };
    let filter = make_element("videobalance")?;
    filter.set_property("brightness", &(filters.brightness as f64));
    filter.set_property("contrast", &(filters.contrast as f64));
    filter.set_property("saturation", &(filters.saturation as f64));
    filter.set_property("hue", &(filters.hue as f64));
    Ok(Some(filter))
}

pub(crate) fn build_output_branches(
    pipeline: &gst::Pipeline,
    output: &Path,
    format: PlayCoreTranscodeFormat,
    filters: Option<&PlayCoreFilterState>,
) -> anyhow::Result<OutputBranches> {
    match format {
        PlayCoreTranscodeFormat::Mp4 | PlayCoreTranscodeFormat::MOV => {
            let video_queue = make_element("queue")?;
            let video_filter = create_video_filter(filters)?;
            let video_convert = make_element("videoconvert")?;
            let video_encoder = make_element("x264enc")?;
            let video_parser = make_element("h264parse")?;
            let audio_queue = make_element("queue")?;
            let audio_convert = make_element("audioconvert")?;
            let audio_resample = make_element("audioresample")?;
            let audio_encoder = make_element("voaacenc")?;
            let audio_parser = make_element("aacparse")?;
            let mux_factory = if matches!(format, PlayCoreTranscodeFormat::Mp4) {
                "mp4mux"
            } else {
                "qtmux"
            };
            let mux = make_element(mux_factory)?;
            if matches!(format, PlayCoreTranscodeFormat::Mp4) {
                mux.set_property("faststart", true);
            }
            let sink = make_filesink(output)?;

            let mut elements = vec![&video_queue];
            if let Some(filter) = video_filter.as_ref() {
                elements.push(filter);
            }
            elements.extend([
                &video_convert,
                &video_encoder,
                &video_parser,
                &audio_queue,
                &audio_convert,
                &audio_resample,
                &audio_encoder,
                &audio_parser,
                &mux,
                &sink,
            ]);
            add_elements(pipeline, &elements)?;

            let mut video_chain = vec![&video_queue];
            if let Some(filter) = video_filter.as_ref() {
                video_chain.push(filter);
            }
            video_chain.extend([&video_convert, &video_encoder, &video_parser, &mux, &sink]);
            link_elements(&video_chain)?;
            link_elements(&[
                &audio_queue,
                &audio_convert,
                &audio_resample,
                &audio_encoder,
                &audio_parser,
                &mux,
            ])?;

            Ok(OutputBranches {
                video_sink: Some(sink_pad(&video_queue)?),
                audio_sink: Some(sink_pad(&audio_queue)?),
            })
        }
        PlayCoreTranscodeFormat::Mkv => {
            let video_queue = make_element("queue")?;
            let video_filter = create_video_filter(filters)?;
            let video_convert = make_element("videoconvert")?;
            let video_encoder = make_element("x264enc")?;
            let video_parser = make_element("h264parse")?;
            let audio_queue = make_element("queue")?;
            let audio_convert = make_element("audioconvert")?;
            let audio_resample = make_element("audioresample")?;
            let audio_encoder = make_element("opusenc")?;
            let mux = make_element("matroskamux")?;
            let sink = make_filesink(output)?;

            let mut elements = vec![&video_queue];
            if let Some(filter) = video_filter.as_ref() {
                elements.push(filter);
            }
            elements.extend([
                &video_convert,
                &video_encoder,
                &video_parser,
                &audio_queue,
                &audio_convert,
                &audio_resample,
                &audio_encoder,
                &mux,
                &sink,
            ]);
            add_elements(pipeline, &elements)?;

            let mut video_chain = vec![&video_queue];
            if let Some(filter) = video_filter.as_ref() {
                video_chain.push(filter);
            }
            video_chain.extend([&video_convert, &video_encoder, &video_parser, &mux, &sink]);
            link_elements(&video_chain)?;
            link_elements(&[
                &audio_queue,
                &audio_convert,
                &audio_resample,
                &audio_encoder,
                &mux,
            ])?;

            Ok(OutputBranches {
                video_sink: Some(sink_pad(&video_queue)?),
                audio_sink: Some(sink_pad(&audio_queue)?),
            })
        }
        PlayCoreTranscodeFormat::Mp3 | PlayCoreTranscodeFormat::FLAC => {
            let audio_queue = make_element("queue")?;
            let audio_convert = make_element("audioconvert")?;
            let audio_resample = make_element("audioresample")?;
            let encoder_factory = if matches!(format, PlayCoreTranscodeFormat::Mp3) {
                "lamemp3enc"
            } else {
                "flacenc"
            };
            let audio_encoder = make_element(encoder_factory)?;
            let sink = make_filesink(output)?;
            add_elements(
                pipeline,
                &[
                    &audio_queue,
                    &audio_convert,
                    &audio_resample,
                    &audio_encoder,
                    &sink,
                ],
            )?;
            link_elements(&[
                &audio_queue,
                &audio_convert,
                &audio_resample,
                &audio_encoder,
                &sink,
            ])?;
            Ok(OutputBranches {
                video_sink: None,
                audio_sink: Some(sink_pad(&audio_queue)?),
            })
        }
        _ => Ok(OutputBranches {
            video_sink: None,
            audio_sink: None,
        }),
    }
}

pub(crate) fn connect_decodebin_pad_added(source: &gst::Element, branches: OutputBranches) {
    let OutputBranches {
        video_sink,
        audio_sink,
    } = branches;
    source.connect_pad_added(move |_source, pad| {
        let caps = pad.current_caps().unwrap_or_else(|| pad.query_caps(None));
        let Some(structure) = caps.structure(0) else {
            return;
        };
        let media_type = structure.name().as_str();
        let target = if media_type.starts_with("video/") {
            video_sink.clone()
        } else if media_type.starts_with("audio/") {
            audio_sink.clone()
        } else {
            None
        };
        let Some(target) = target else {
            return;
        };
        if target.is_linked() {
            return;
        }
        if let Err(error) = pad.link(&target) {
            log::warn!("[gst:export] link {media_type} pad failed: {error}");
        }
    });
}

pub(crate) fn run_pipeline_until_input_end(
    pipeline: &gst::Pipeline,
    appsrc: &gstreamer_app::AppSrc,
    receiver: &mut tokio::sync::mpsc::Receiver<Vec<u8>>,
) -> anyhow::Result<()> {
    if let Err(error) = pipeline.set_state(gst::State::Playing) {
        let _ = pipeline.set_state(gst::State::Null);
        return Err(error.into());
    }

    let push_result = loop {
        let Some(chunk) = receiver.blocking_recv() else {
            break appsrc
                .end_of_stream()
                .map_err(|error| anyhow::anyhow!(error));
        };
        if chunk.is_empty() {
            continue;
        }
        if let Err(error) = appsrc.push_buffer(gst::Buffer::from_mut_slice(chunk)) {
            break Err(anyhow::anyhow!("向实时转码管线写入数据失败: {error}"));
        }
    };

    let result = push_result.and_then(|_| wait_for_eos(pipeline));
    let _ = pipeline.set_state(gst::State::Null);
    result
}

pub(crate) fn run_export_pipeline(
    pipeline: &gst::Pipeline,
    trim: Option<PlayCoreExportTrim>,
) -> anyhow::Result<()> {
    if let Some(trim) = trim {
        if let Err(error) = pipeline.set_state(gst::State::Paused) {
            let _ = pipeline.set_state(gst::State::Null);
            return Err(error.into());
        }
        let (state_result, state, _) = pipeline.state(Some(gst::ClockTime::from_seconds(10)));
        state_result.context("导出管线进入暂停状态失败")?;
        if state < gst::State::Paused {
            bail!("导出管线未完成预加载");
        }
        let start =
            gst::ClockTime::from_nseconds(trim.start.as_nanos().min(u64::MAX as u128) as u64);
        let end = gst::ClockTime::from_nseconds(trim.end.as_nanos().min(u64::MAX as u128) as u64);
        pipeline.seek(
            1.0,
            gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE | gst::SeekFlags::SEGMENT,
            gst::SeekType::Set,
            start,
            gst::SeekType::Set,
            end,
        )?;
    }

    if let Err(error) = pipeline.set_state(gst::State::Playing) {
        let _ = pipeline.set_state(gst::State::Null);
        return Err(error.into());
    }
    let result = wait_for_eos(pipeline);
    let _ = pipeline.set_state(gst::State::Null);
    result
}

fn wait_for_eos(pipeline: &gst::Pipeline) -> anyhow::Result<()> {
    let bus = pipeline.bus().context("导出管线没有消息总线")?;
    let mut segment_finished = false;
    loop {
        let Some(message) = bus.timed_pop(gst::ClockTime::from_mseconds(100)) else {
            continue;
        };
        match message.view() {
            gst::MessageView::Eos(..) => return Ok(()),
            gst::MessageView::SegmentDone(_) if !segment_finished => {
                segment_finished = true;
                pipeline.send_event(gst::event::Eos::new());
            }
            gst::MessageView::Error(error) => {
                bail!("导出失败: {} ({:?})", error.error(), error.debug());
            }
            _ => {}
        }
    }
}
