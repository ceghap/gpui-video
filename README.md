# gpui-video 🎥

[![Crates.io](https://img.shields.io/crates/v/gpui-video.svg)](https://crates.io/crates/gpui-video)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A high-performance, GPU-accelerated video playback and real-time RTSP streaming widget for **GPUI** (Zed's GPU-accelerated desktop UI framework).

## Features

- ⚡ **Zero-Copy GPU Uploads**: Efficient frame ingestion into GPUI's image pipeline.
- 📡 **Real-Time RTSP Streaming**: Native support for low-latency CCTV and IP camera feeds (Tapo, Reolink, ONVIF, Hikvision).
- 🎞️ **Local Video Playback**: Smooth MP4, MKV, and WebM video playback with correct aspect-ratio letterboxing.
- 🦀 **Pure Rust Integration**: Designed specifically for the GPUI component model.

## Quick Start

Add `gpui-video` to your `Cargo.toml`:

```toml
[dependencies]
gpui = "0.2.2"
gpui-video = "0.1"
```

## Running Examples

### 1. Dynamic Texture Test (60 FPS Test Pattern)
```bash
cargo run --example dynamic_texture
```

### 2. Live RTSP IP Camera Stream
```bash
cargo run --example rtsp_stream -- "rtsp://username:password@192.168.1.3:554/stream2"
```

## License

Dual-licensed under either:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
