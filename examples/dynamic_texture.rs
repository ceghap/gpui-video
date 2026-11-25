use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, Render, Window, WindowOptions,
};
use gpui_platform::application;

struct VideoMonitor {
    title: String,
}

impl Render for VideoMonitor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x0f172a))
            .text_color(rgb(0xf8fafc))
            .justify_center()
            .items_center()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(0x38bdf8))
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x94a3b8))
                    .child("GPU-Accelerated Native Video Canvas for GPUI (Vulkan / Metal / Direct3D)"),
            )
            .child(
                div()
                    .size(px(320.0))
                    .bg(rgb(0x020617))
                    .border_2()
                    .border_color(rgb(0x38bdf8))
                    .rounded_xl()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x10b981))
                            .child("● Ready for Zero-Copy Frame Ingestion"),
                    ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("gpui-video • Native Prototype".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                focus: true,
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| VideoMonitor {
                    title: "gpui-video Stream Engine".into(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
