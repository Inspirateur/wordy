use std::fmt::Display;
use super::rasterisable::Rasterisable;

pub struct Image {
    path: String
}

impl Image {
    pub fn new(path: String) -> Self {
        Self {path}
    }
}

impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl Rasterisable for Image {
    fn to_bitmap(&self, size: f32) -> super::hxbitmap::HXBitmap {
        todo!()
    }

    fn draw(&self, image: &mut image::RgbaImage, pos: (usize, usize), size: f32) {
        todo!()
    }
}