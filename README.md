# gpui-video

A video playback and real-time RTSP streaming component for [GPUI](https://github.com/zed-industries/zed) (Zed's GPU-accelerated desktop UI framework).

`gpui-video` provides a GPUI `Render` view that decodes and displays video streams and files directly within GPUI applications.

## Features

- Real-Time RTSP Streaming: Stream live feeds from IP cameras and network sources with automatic reconnection.
- Local Video File Playback: Render video files (MP4, MKV, WebM, etc.) within a GPUI view.
- Aspect-Ratio Handling: Native letterboxing and aspect ratio preservation using GPUI image rendering.
- Overlay & Status Indicators: Built-in connection state and frame statistics overlay.

## Prerequisites

`gpui-video` uses `ffmpeg` for asynchronous video decoding. Ensure `ffmpeg` is installed and available on your system `PATH`:

```bash
# Ubuntu / Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg

# Arch Linux
sudo pacman -S ffmpeg
```

## Installation

Add `gpui-video` and `gpui` to your `Cargo.toml`:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui-video = { git = "https://github.com/ceghap/gpui-video" }
```

## Usage

```rust
use gpui::{prelude::*, App, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;
use gpui_video::{VideoPlayer, VideoSource};

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.0), px(540.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    VideoPlayer::new(
                        VideoSource::Rtsp("rtsp://username:password@192.168.1.100:554/stream1".into()),
                        640,
                        360,
                        15,
                        cx,
                    )
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
```

## Running Examples

### Dynamic Texture Test
```bash
cargo run --example dynamic_texture
```

### Live RTSP Stream
```bash
cargo run --example rtsp_stream -- "rtsp://username:password@192.168.1.100:554/stream1"
```

## License

Dual-licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.
