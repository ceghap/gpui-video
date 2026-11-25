use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{
    div, img, prelude::*, px, rgb, Context, Render, RenderImage, Task, Window,
};
use smallvec::smallvec;

/// Source for video playback or camera streaming.
#[derive(Debug, Clone)]
pub enum VideoSource {
    /// Live RTSP camera or network stream (e.g., `rtsp://user:pass@10.0.0.1:554/stream1`).
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
    running: Arc<AtomicBool>,
    _decode_task: Option<Task<()>>,
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
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
            running: Arc::new(AtomicBool::new(true)),
            _decode_task: None,
        };

        player.start_decoding(cx);
        player
    }

    /// Update the video source dynamically.
    pub fn set_source(&mut self, source: VideoSource, cx: &mut Context<Self>) {
        self.running.store(false, Ordering::Relaxed);
        self.source = source;
        self.current_frame = None;
        self.is_connected = false;
        self.status_text = "Switching stream...".to_string();
        self.running = Arc::new(AtomicBool::new(true));
        self.start_decoding(cx);
        cx.notify();
    }

    fn start_decoding(&mut self, cx: &mut Context<Self>) {
        let source = self.source.clone();
        let width = self.width;
        let height = self.height;
        let fps = self.fps;
        let running_clone = self.running.clone();

        let (mut sender, receiver) = mpsc::channel::<(Arc<RenderImage>, u64)>(2);
        let (status_tx, status_rx) = mpsc::unbounded::<String>();

        // Dedicated OS worker thread for frame ingestion (reactor-independent)
        thread::spawn(move || {
            let stream_target = match &source {
                VideoSource::Rtsp(url) => url.clone(),
                VideoSource::File(path) => path.to_string_lossy().to_string(),
            };

            let frame_bytes_len = (width * height * 4) as usize;
            let mut frame_buf = vec![0u8; frame_bytes_len];
            let mut frame_seq = 0u64;

            while running_clone.load(Ordering::Relaxed) {
                let _ = status_tx.unbounded_send(format!("Connecting to stream: {}", stream_target));

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
                    .spawn()
                {
                    Ok(c) => c,
                    Err(e) => {
                        let msg = if e.kind() == std::io::ErrorKind::NotFound {
                            "FFmpeg not found in PATH. Please install FFmpeg (brew / apt / winget).".to_string()
                        } else {
                            format!("FFmpeg launch failed: {}", e)
                        };
                        let _ = status_tx.unbounded_send(msg);
                        thread::sleep(Duration::from_secs(4));
                        continue;
                    }
                };

                let mut stdout = match child.stdout.take() {
                    Some(s) => s,
                    None => {
                        let _ = child.kill();
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                };

                let _ = status_tx.unbounded_send("Live Stream Active".to_string());

                while running_clone.load(Ordering::Relaxed) && stdout.read_exact(&mut frame_buf).is_ok() {
                    if let Some(rgba_image) =
                        image::RgbaImage::from_raw(width, height, frame_buf.clone())
                    {
                        let frame = image::Frame::new(rgba_image);
                        let render_image = Arc::new(RenderImage::new(smallvec![frame]));
                        frame_seq += 1;

                        // Non-blocking try_send ensures real-time playback with zero lag
                        let _ = sender.try_send((render_image, frame_seq));
                    }
                }

                let _ = child.kill();
                if !running_clone.load(Ordering::Relaxed) {
                    break;
                }
                let _ = status_tx.unbounded_send("Stream disconnected. Reconnecting...".to_string());
                thread::sleep(Duration::from_secs(2));
            }
        });

        // GPUI Task: receives frames and dispatches to GPUI render loop
        let task = cx.spawn(async move |this, cx| {
            let mut status_fuse = status_rx.fuse();
            let mut frames_fuse = receiver.fuse();

            loop {
                futures::select! {
                    status = status_fuse.next() => {
                        if let Some(s) = status {
                            let _ = this.update(cx, |this, cx| {
                                this.status_text = s;
                                cx.notify();
                            });
                        } else {
                            break;
                        }
                    }
                    frame_item = frames_fuse.next() => {
                        if let Some((render_image, seq)) = frame_item {
                            let _ = this.update(cx, |this, cx| {
                                this.current_frame = Some(render_image);
                                this.frame_count = seq;
                                this.is_connected = true;
                                cx.notify();
                            });
                        } else {
                            break;
                        }
                    }
                }
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
                                format!("LIVE • {} FPS", self.fps)
                            } else {
                                status
                            }),
                    ),
            )
    }
}
