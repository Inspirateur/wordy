mod wordcloud;
use serenity::utils::Color;
use wordcloud::{wordcloud, Token};

fn test_tokens() -> Vec<(Token, f32)> {
    vec![
        (Token::IMG("assets/images/kek.webp".to_string()), 0.01), 
        (Token::IMG("assets/images/kingsip.webp".to_string()), 0.005), 
        (Token::TEXT("KeK'".to_string()), 0.02), 
        (Token::TEXT("Sauce".to_string()), 0.01), 
        (Token::TEXT("Polisson".to_string()), 0.005),
        (Token::TEXT("C'est".to_string()), 0.005),
        (Token::TEXT("la".to_string()), 0.005),
        (Token::TEXT("fête".to_string()), 0.005)
    ]
}

fn main() {
    let font = include_bytes!("../assets/whitneymedium.otf") as &[u8];
    // Parse it into the font type.
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

    let wc = wordcloud(font, (800, 400), test_tokens(), Some(Color::from_rgb(220, 240, 60)));
    wc.save("test.png").unwrap();
}
