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
    source: String,
    directory: String,
    hash: String,
    height: u64,
    id: u64,
    image: String,
    change: u64,
    owner: String,
    parent_id: Option<u64>,
    rating: String,
    sample: u64,
    preview_height: u64,
    preview_width: u64,
    sample_height: u64,
    sample_width: u64,
    score: u64,
    tags: String,
    title: String,
    width: u64,
    file_url: String,
    created_at: String,
    post_locked: u64,
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
}
