use lazy_static::lazy_static;
use palette::{rgb::Rgba, FromColor, Lcha, Hue};
use image::{Rgba as IRgba};
use serenity::utils::Color;
use rand::{self, rngs::ThreadRng, distributions::Uniform, prelude::Distribution};

const ANGLES: [i32; 5] = [
    -15, 0, 15, 180-15, 180+15
];

lazy_static! {
    static ref INDEXES: Uniform<usize> = Uniform::from(0..ANGLES.len());
    static ref H_NOISE: Uniform<f32> = Uniform::from(-2.0..2.);
    static ref C_NOISE: Uniform<f32> = Uniform::from(-10.0..10.);
}

pub struct Colors {
    anchor: Lcha,
    rng: ThreadRng
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
    pub fn new(color_opt: Option<Color>) -> Self {
        let color: Rgba = match color_opt {
            Some(color) => Rgba::new(
                color.r() as f32/255., 
                color.g() as f32/255., 
                color.b() as f32/255., 
                1.
            ),
            None => Rgba::new(1., 1., 1., 1.)
        };
        let mut anchor = Lcha::from_color(color);
        anchor.chroma = anchor.chroma.max(20.);
        Self {
            rng: rand::thread_rng(),
            anchor
        }
    }

    pub fn get(&mut self) -> IRgba<u8> {
        let angle = ANGLES[INDEXES.sample(&mut self.rng)] as f32;
        let hue_shift = angle+H_NOISE.sample(&mut self.rng);
        let mut new_color = self.anchor.shift_hue(hue_shift);
        new_color.chroma += C_NOISE.sample(&mut self.rng);
        convert_color(Rgba::from_color(new_color))
    }
}