// @TODO: actually write some tests :p

use crate::{posts, Client, Rating};

#[test]
fn posts_builder() {
    let client = Client::public();

    let req = posts()
        .limit(5)
        .rating(Rating::Safe)
        .tags_raw("raw")
        .tags(&["hatsune_miku", "solo"]);

    dbg!(&req);

    req.send(&client);
}

#[test]
fn posts_builder_tags() {
    let _client = Client::public();

    let req = posts()
        .tags_raw("hello! :D")
        .tag("hello")
        .tags(&["hatsune_miku", "solo"]);

    //req.send(&client);
}
