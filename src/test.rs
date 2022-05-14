use crate::{posts, tags, Client, Rating, TagType};

#[tokio::test]
async fn posts_builder() {
    let client = Client::public();

    let req = posts()
        .limit(5)
        .rating(Rating::Safe)
        .tags_raw("raw")
        .tags(&["hatsune_miku", "solo"]);

    let result = req.send(&client).await;
    dbg!(&result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn posts_tags() {
    let client = Client::public();

    let req = posts()
        .tags_raw("hello")
        .tag("there")
        .tags(&["hatsune_miku", "solo"]);

    let result = req.send(&client).await;
    dbg!(&result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn posts_bad_raw_tags() {
    let client = Client::public();

    let req = posts().tags_raw("hello! :D");

    let result = req.send(&client).await;
    dbg!(&result);
    assert!(result.is_err());
}

#[tokio::test]
async fn tags_correct_mapping() {
    let client = Client::public();

    async fn compare_mapping(client: &Client, tag: &str, expected: TagType) {
        let tags = tags().name(client, tag).await;
        let tag = tags.unwrap().unwrap();
        assert_eq!(tag.tag_type(), expected);
    }

    compare_mapping(&client, "step_arts", TagType::Artist).await;
    compare_mapping(&client, "clownpiece", TagType::Character).await;
    compare_mapping(&client, "touhou", TagType::Copyright).await;
    //compare_mapping(&client, "", TagType::Deprecated).await;
    compare_mapping(&client, "translation_request", TagType::Metadata).await;
    compare_mapping(&client, "solo", TagType::Tag).await;
}
