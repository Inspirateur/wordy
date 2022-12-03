use palette::rgb::Rgba;
use image::Rgba as IRgba;
use serenity::utils::Color;

pub struct Colors {

}

fn convert_color(color: Rgba) -> IRgba<u8> {
    IRgba([
        (color.red*255.) as u8,
        (color.green*255.) as u8,
        (color.blue*255.) as u8,
        (color.alpha*255.) as u8,
    ])
}

impl Colors {
    pub fn new(color: Option<Color>) -> Self {
        Self {  }
    }

    pub fn get(&mut self) -> IRgba<u8> {
        convert_color(Rgba::new(0.7, 0.4, 0.1, 1.0))
    }
}