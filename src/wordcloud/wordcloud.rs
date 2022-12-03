use fontdue::Font;
use serenity::utils::Color;
use super::{hxbitmap::HXBitmap, colors::Colors, text::Text, rasterisable::Rasterisable, image::Image};
use image::RgbaImage;
// Biggest possible word will take 1/FRACTION of the image
const FRACTION: usize = 4;
pub enum Token {
    TEXT(String),
    IMG(String)
}

struct WorldCloud {
    bitmap: HXBitmap,
    pub image: RgbaImage,
}

impl WorldCloud {
    pub fn new(dim: (usize, usize)) -> Self {
        let bitmap = HXBitmap::new(dim.0, dim.1);
        let image = RgbaImage::new(dim.0 as u32, dim.1 as u32);
        Self { bitmap, image }
    }

    pub fn add(&mut self, token: Box<dyn Rasterisable>) -> bool {
        let bitmap = token.to_bitmap();
        match self.bitmap.place(bitmap) {
            Ok(pos) => {
                token.draw(&mut self.image, pos);
                println!("Placed `{}` at {:?}", token, pos);
                true
            },
            Err(err) => {
                println!("{:?}", err);
                false
            }
        }
    }
}

pub fn size_factor(dim: (usize, usize), tokens: &Vec<(Token, f32)>) -> f32 {
    let sum = tokens.iter().fold(0., |i, (_, s)| i+s);
    let c = dim.0/FRACTION;
    2.*c as f32/sum
}

pub fn wordcloud(font: Font, dim: (usize, usize), mut tokens: Vec<(Token, f32)>, primary: Option<Color>) -> RgbaImage {
    let c = size_factor(dim, &tokens); 
    tokens.sort_by(|(_, s1), (_, s2)| s2.partial_cmp(s1).unwrap());
    let mut colors = Colors::new(primary);
    let mut wc = WorldCloud::new(dim);
    for (token, size) in tokens {
        let rasterisable: Box<dyn Rasterisable> = match token {
            Token::TEXT(text) => Box::new(Text::new(text, font.clone(), size*c, colors.get())),
            Token::IMG(path) => Box::new(Image::new(path, size*c))
        };
        if !wc.add(rasterisable) {
            break;
        }
    }
    // println!("Wordcloud bitmap ({}x{})\n{}", wc.bitmap.width, wc.bitmap.height, wc.bitmap);
    wc.image
}