mod wordcloud;
use wordcloud::{wordcloud, Token};

fn test_tokens() -> Vec<(Token, f32)> {
    vec![
        // (Token::IMG("../assets/images/pogo.png".to_string()), 0.01), 
        // (Token::IMG("../assets/images/sadge.png".to_string()), 0.005), 
        (Token::TEXT("KeK'".to_string()), 0.02), 
        (Token::TEXT("Sauce".to_string()), 0.01), 
        (Token::TEXT("Polisson".to_string()), 0.005)
    ]
}

fn main() {
    let font = include_bytes!("../assets/whitneymedium.otf") as &[u8];
    // Parse it into the font type.
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

    let wc = wordcloud(font, (160, 80), test_tokens(), None);
    wc.save("test.png").unwrap();
}
