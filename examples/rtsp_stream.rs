use std::env;
use gpui::{
    prelude::*, px, size, App, Bounds, WindowBounds, WindowOptions,
};
use gpui_platform::application;
use gpui_video::{VideoPlayer, VideoSource};

fn main() {
    let args: Vec<String> = env::args().collect();
    let stream_url = if args.len() > 1 {
        args[1].clone()
    } else {
        std::env::var("RTSP_URL").unwrap_or_else(|_| {
            "rtsp://username:password@192.168.1.100:554/stream1".to_string()
        })
    };

    println!("[gpui-video] Starting native GPUI RTSP Player for: {}", stream_url);

    application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.0), px(540.0)), cx);
        let url_clone = stream_url.clone();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("gpui-video • Live RTSP Camera Player".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                focus: true,
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    VideoPlayer::new(
                        VideoSource::Rtsp(url_clone),
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
