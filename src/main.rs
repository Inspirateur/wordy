mod idiom;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use idiom::Idioms;
use itertools::Itertools;
use wordcloud_rs::*;

fn main() {
    let mut idioms = Idioms::new();
    let file = File::open("assets/movie_lines.tsv").unwrap();
    let reader = BufReader::new(file);
    for line_res in reader.lines() {
        if let Ok(line) = line_res {
            let record = line.split("\t").collect_vec();
            if record.len() == 3 {
                idioms.update(
                    String::new(), record[1].to_string(), record[2].to_string()
                );
            }
        }
    }
    let person = "SPIDER-MAN".to_string();
    let idiom = idioms.idiom(person);
    let tokens = idiom.into_iter()
    .map(|(token, v)| (Token::Text(token), v)).collect_vec();
    let img = WordCloud::new().generate(tokens);
    img.save("test.png").unwrap();
}
