use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    div, img, prelude::*, px, rgb, Context, Render, RenderImage, Task, Window,
};
use smallvec::smallvec;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// Source for video playback or camera streaming.
#[derive(Debug, Clone)]
pub enum VideoSource {
    /// Live RTSP camera stream (e.g., `rtsp://user:pass@192.168.1.3:554/stream2`).
    Rtsp(String),
    /// Local video file path (e.g., `.mp4`, `.mkv`, `.webm`).
    File(PathBuf),
}

/// A GPU-accelerated video player view for GPUI.
pub struct VideoPlayer {
    source: VideoSource,
    width: u32,
    height: u32,
    fps: u32,
    current_frame: Option<Arc<RenderImage>>,
    frame_count: u64,
    status_text: String,
    is_connected: bool,
    _decode_task: Option<Task<()>>,
}

impl VideoPlayer {
    /// Create a new VideoPlayer with the specified source and target resolution.
    pub fn new(
        source: VideoSource,
        width: u32,
        height: u32,
        fps: u32,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut player = Self {
            source: source.clone(),
            width,
            height,
            fps,
            current_frame: None,
            frame_count: 0,
            status_text: "Initializing video stream...".to_string(),
            is_connected: false,
            _decode_task: None,
        };

        player.start_decoding(cx);
        player
    }

    /// Update the video source dynamically.
    pub fn set_source(&mut self, source: VideoSource, cx: &mut Context<Self>) {
        self.source = source;
        self.current_frame = None;
        self.is_connected = false;
        self.status_text = "Switching stream...".to_string();
        self.start_decoding(cx);
        cx.notify();
    }

    fn start_decoding(&mut self, cx: &mut Context<Self>) {
        let source = self.source.clone();
        let width = self.width;
        let height = self.height;
        let fps = self.fps;

        let task = cx.spawn(async move |this, cx| {
            let stream_target = match &source {
                VideoSource::Rtsp(url) => url.clone(),
                VideoSource::File(path) => path.to_string_lossy().to_string(),
            };

            let frame_bytes_len = (width * height * 4) as usize;
            let mut frame_buf = vec![0u8; frame_bytes_len];

            loop {
                let _ = this.update(cx, |this, cx| {
                    this.status_text = format!("Connecting to stream: {}", stream_target);
                    this.is_connected = false;
                    cx.notify();
                });

                let mut child = match Command::new("ffmpeg")
                    .args([
                        "-y",
                        "-rtsp_transport",
                        "tcp",
                        "-i",
                        &stream_target,
                        "-vf",
                        &format!("scale={}:{}", width, height),
                        "-f",
                        "rawvideo",
                        "-pix_fmt",
                        "rgba",
                        "-r",
                        &format!("{}", fps),
                        "pipe:1",
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .kill_on_drop(true)
                    .spawn()
                {
                    Ok(c) => c,
                    Err(e) => {
                        let err_msg = format!("FFmpeg launch failed: {}", e);
                        let _ = this.update(cx, |this, cx| {
                            this.status_text = err_msg;
                            cx.notify();
                        });
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        continue;
                    }
                };

                let mut stdout = match child.stdout.take() {
                    Some(s) => s,
                    None => {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };

                let mut first_frame = true;

                while stdout.read_exact(&mut frame_buf).await.is_ok() {
                    if let Some(rgba_image) =
                        image::RgbaImage::from_raw(width, height, frame_buf.clone())
                    {
                        let frame = image::Frame::new(rgba_image);
                        let render_image = Arc::new(RenderImage::new(smallvec![frame]));

                        let res = this.update(cx, |this, cx| {
                            this.current_frame = Some(render_image);
                            this.frame_count += 1;
                            this.is_connected = true;
                            if first_frame {
                                this.status_text = "Live Stream Active".to_string();
                                first_frame = false;
                            }
                            cx.notify();
                        });

                        if res.is_err() {
                            return;
                        }
                    }
                }

                let _ = this.update(cx, |this, cx| {
                    this.status_text = "Stream disconnected. Reconnecting in 2s...".to_string();
                    this.is_connected = false;
                    cx.notify();
                });

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        self._decode_task = Some(task);
    }
}

impl Render for VideoPlayer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let frame_count = self.frame_count;
        let is_connected = self.is_connected;
        let status = self.status_text.clone();

        div()
            .relative()
            .size_full()
            .bg(rgb(0x020617)) // Slate 950
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .child(if let Some(render_image) = &self.current_frame {
                div().size_full().child(
                    img(render_image.clone())
                        .size_full()
                        .object_fit(gpui::ObjectFit::Contain),
                )
            } else {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .p_6()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(0x38bdf8))
                            .child("Connecting to Video Stream..."),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x64748b))
                            .child(status.clone()),
                    )
            })
            // Overlay Status Badge
            .child(
                div()
                    .absolute()
                    .top(px(12.0))
                    .left(px(12.0))
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(rgb(0x0f172a))
                    .border_1()
                    .border_color(rgb(0x334155))
                    .rounded_md()
                    .px_2()
                    .py_1()
                    .child(
                        div()
                            .size(px(8.0))
                            .rounded_full()
                            .bg(if is_connected {
                                rgb(0x10b981) // Emerald
                            } else {
                                rgb(0xf59e0b) // Amber
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(0xf8fafc))
                            .child(if is_connected {
                                format!("LIVE • {} FPS • #{}", self.fps, frame_count)
                            } else {
                                status
                            }),
                    ),
            )
    }
}
