# Gelbooru API
[![Crates.io](https://img.shields.io/crates/v/gelbooru-api)](https://crates.io/crates/gelbooru-api)

Rudimentary Gelbooru API.

### Usage
Fetch latest 20 Safe-rated posts with tags `solo` and `hatsune_miku`.
```rust
use gelbooru_api::{Client, Rating, posts};

let client = Client::public();
let posts = posts()
	.limit(20)
	.rating(Rating::Safe)
	.tags(&["solo", "hatsune_miku"])
	.send(&client)
	.await?;

for post in posts {
	println!(
		"Post {id} created at {created_at} by {owner} [{image_url}]",
		id = post.id(),
		created_at = post.created_at(),
		owner = post.owner(),
		image_url = post.image_url(),
	);
}
```

### API coverage
- [x] Authentication
- [x] Posts
- [x] Tags
- [ ] Users
- [ ] Comments
