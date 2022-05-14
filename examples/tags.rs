use gelbooru_api::{Client, tags};

type EResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> EResult<()> {
    let client = Client::public();

    let tags = tags()
        .limit(5)
        .send(&client)
        .await?;

    for tag in tags.tags {
        println!(
            "Tag {name:25} [{id:06}] count {count:7} of type {tag_type:?}, ambigious: {ambigious}",
            id = tag.id(),
            name = tag.name(),
            count = tag.count(),
            tag_type = tag.tag_type(),
            ambigious = tag.ambigious(),
        );
    }

    Ok(())
}
