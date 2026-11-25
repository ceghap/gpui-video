use gpui::*;

pub struct VideoPlayer {
    width: u32,
    height: u32,
}

impl VideoPlayer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
