use palette::rgb::Rgba;
use serenity::utils::Color;

pub struct Colors {

}

impl Colors {
    pub fn new(color: Option<Color>) -> Self {
        Self {  }
    }

    pub fn get(&mut self) -> Rgba {
        Rgba::new(0.7, 0.4, 0.1, 0.0)
    }
}