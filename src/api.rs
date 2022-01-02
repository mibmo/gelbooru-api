use serde::Deserialize;

/*
use chrono::{offset::FixedOffset, DateTime};
#[derive(Clone, Copy, Debug)]
pub enum Ordering {
    Date,
    Count,
    Name,
}

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub id: u64,
    pub tag: String,
    pub count: u64,
    pub r#type: String,
    pub ambiguous: u64,
}

// @TODO: TagType enum
*/

#[derive(Deserialize, Debug)]
pub struct Post {
    pub source: String,
    pub directory: String,
    pub hash: String,
    pub height: u64,
    pub id: u64,
    pub image: String,
    pub change: u64,
    pub owner: String,
    pub parent_id: Option<u64>,
    pub rating: String,
    pub sample: u64,
    pub preview_height: u64,
    pub preview_width: u64,
    pub sample_height: u64,
    pub sample_width: u64,
    pub score: u64,
    pub tags: String,
    pub title: String,
    pub width: u64,
    pub file_url: String,
    pub created_at: String,
    pub post_locked: u64,
}

impl Post {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn title<'a>(&'a self) -> &'a str {
        &self.title
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::offset::FixedOffset> {
        chrono::DateTime::parse_from_str(&self.created_at, "%a %b %d %H:%M:%S %z %Y")
            .expect("failed to parse DateTime")
    }

    pub fn rating<'a>(&'a self) -> crate::Rating {
        use crate::Rating::*;
        match &self.rating[0..1] {
            "s" => Safe,
            "q" => Questionable,
            "e" => Explicit,
            _ => unreachable!("non-standard rating"),
        }
    }

    pub fn owner<'a>(&'a self) -> &'a str {
        &self.owner
    }

    pub fn tags<'a>(&'a self) -> Vec<&'a str> {
        self.tags.split(' ').collect()
    }

    pub fn dimensions(&self) -> (u64, u64) {
        (self.width, self.height)
    }

    pub fn hash<'a>(&'a self) -> &'a str {
        &self.hash
    }

    pub fn image_url<'a>(&'a self) -> &'a str {
        &self.file_url
    }

    pub fn source<'a>(&'a self) -> &'a str {
        &self.source
    }
}
