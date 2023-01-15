use image::DynamicImage;
use moka::sync::Cache;
use reqwest::get;
use anyhow::Result;

pub struct DiscordEmojis {
    emojis: Cache<String, DynamicImage>
}

impl DiscordEmojis {
    pub fn new(cap: usize) -> Self {
        Self {
            emojis: Cache::new(cap as u64)
        }
    }

    pub async fn get(&self, id: &str) -> Result<DynamicImage> {
        if let Some(img) = self.emojis.get(id) {
            Ok(img.clone())
        } else {
            let url = format!("https://cdn.discordapp.com/emojis/{}.webp", id);
            let img_bytes = get(url).await?.bytes().await?;
            let image = image::load_from_memory(&img_bytes)?;
            self.emojis.insert(id.to_string(), image.clone());
            Ok(image)
        }
    }
}