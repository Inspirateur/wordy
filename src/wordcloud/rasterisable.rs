use super::hxbitmap::HXBitmap;
use image::RgbaImage;
use palette::rgb::Rgba;
use std::fmt::{Debug, Display};

pub trait Rasterisable: Display {
    fn to_bitmap(&self) -> HXBitmap;

    fn draw(&self, image: &mut RgbaImage, pos: (usize, usize)); 
}