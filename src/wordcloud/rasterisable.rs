use super::hxbitmap::HXBitmap;
use image::RgbaImage;
use std::fmt::{Debug, Display};

pub trait Rasterisable: Display {
    fn to_bitmap(&self, size: f32) -> HXBitmap;

    fn draw(&self, image: &mut RgbaImage, pos: (usize, usize), size: f32); 
}