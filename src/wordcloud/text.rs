use fontdue::{Font, layout::{Layout, CoordinateSystem, TextStyle}};
use image::RgbaImage;
use itertools::enumerate;
use palette::rgb::Rgba;
use super::{hxbitmap::{HXBitmap, CHARS}, rasterisable::Rasterisable, indexed_chars::IndexedChars};
use std::{iter::zip, fmt::Display};

pub struct Text {
    fonts: [Font; 1],
    text: String,
    color: Rgba,
}

impl Text {
    pub fn new(text: String, font: Font, color: Rgba) -> Self {
        Self { fonts: [font], text, color }
    }

    fn layout(&self, size: f32) -> Layout {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&self.fonts, &TextStyle::new(self.text.as_str(), size, 0));
        layout
    }
}

fn compute_dim(layout: &Layout) -> (usize, usize) {
    let (mut x1, mut y1, mut x2, mut y2): (i32, i32, i32, i32) = (0, 0, 0, 0);
    for pos in layout.glyphs() {
        x1 = x1.min(pos.x as i32);
        y1 = y1.min(pos.y as i32);
        x2 = x2.max(pos.x as i32+pos.width as i32);
        y2 = y2.max(pos.y as i32+pos.height as i32);
    }
    return (1+(x2-x1) as usize, (y2-y1) as usize)
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Rasterisable for Text {
    fn to_bitmap(&self, size: f32) -> HXBitmap {
        let layout = self.layout(size);
        let indexed = IndexedChars::new(&self.text);
        let glyphs: Vec<_> = indexed.chars.iter().map(|c| self.fonts[0].rasterize(*c, size)).collect();
        let dim = compute_dim(&layout);
        let mut bitmap = HXBitmap::new(dim.0, dim.1);
        for (pos, (metrics, char_bitmap)) in zip(layout.glyphs(), glyphs) {
            bitmap.add_bitmap(metrics.width, &char_bitmap, pos.x as usize, pos.y as usize);
        }
        bitmap
    }

    fn draw(&self, image: &mut RgbaImage, pos: (usize, usize), size: f32) {
        let layout = self.layout(size);
        todo!()
    }
}
