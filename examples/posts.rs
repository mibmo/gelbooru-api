use gelbooru_api::{Client, Rating, posts};

type EResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> EResult<()> {
    let client = Client::public();

    let posts = posts()
        .limit(5)
        .rating(Rating::Safe)
        .tags(&["solo", "hatsune_miku"])
        .send(&client)
        .await?;

    for post in posts.posts {
        println!(
            "Post {id} created at {created_at} by {owner} [{image_url}]",
            id = post.id(),
            created_at = post.created_at(),
            owner = post.owner(),
            image_url = post.image_url(),
        );
    }

    Ok(())
}
