use itertools::Itertools;
use leaderboard::{Ranking, View, Sections};
use serenity::model::prelude::Emoji;
const TOP_EMO: usize = 5;
const MAX_EMO_GROUP: usize = 15;

pub struct EmojiUsage(pub Emoji, pub f64);

impl PartialEq for EmojiUsage {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl PartialOrd for EmojiUsage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

pub fn emo_entry_msg(rank: usize, freq: f64, emos: Vec<&Emoji>) -> String {
    // limit to 20 emojis because the message gets too long otherwise
    let ellipsis = if emos.len() > MAX_EMO_GROUP { "… " } else { "" };
    let emo_str = emos.into_iter().take(MAX_EMO_GROUP).join("");
    format!("{}. {}{}: {:.0}%", rank, emo_str, ellipsis, freq*100.0)
}

pub fn emo_ranking_msg(emo_ranking: Vec<EmojiUsage>) -> String {
    if emo_ranking.len() == 0 {
        return "No entries :(".to_string();
    }
    let grouped_ranking = emo_ranking.iter_ranked().collect_vec();
    let len = grouped_ranking.len();
    let mut rank = 0;
    grouped_ranking.iter_sections(vec![0..TOP_EMO, (len-1)..len]).map(|view|
        match view {
            View::Item(emo_usages) => {
                let emos = emo_usages.into_iter().map(
                    |EmojiUsage(emo, _usage)| emo
                ).collect_vec();
                let freq = emo_usages[0].1;
                rank += 1;
                emo_entry_msg(rank, freq, emos)
            }
            View::Skipped(count) => {
                rank += count;
                "...".to_string()
            }
        }
    ).join("\n")
}
