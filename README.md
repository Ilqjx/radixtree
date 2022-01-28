# radixtree

A radix tree implementation for router, and provides CRUD operations.

Radixtree is part of [treemux](https://github.com/casualjim/rs-treemux/), on top of which updates and removes are added.

## Usage

Add this to your `Cargo.toml`:

```
radixtree = "0.1.0"
```

### Insert

```rust
use radixtree::{Node, Method};

fn main() {
    let mut tree = Node::new();
    tree.insert(Method::GET, "/", "GET");
    tree.insert(Method::POST, "/", "POST");
    tree.insert(Method::PUT, "/", "PUT");
    tree.insert(Method::DELETE, "/", "DELETE");

    let result = tree.search(Method::GET, "/");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"GET");

    let result = tree.search(Method::POST, "/");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"POST");

    let result = tree.search(Method::PUT, "/");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"PUT");

    let result = tree.search(Method::DELETE, "/");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"DELETE");
}
```

### Update

```rust
use radixtree::{Node, Method};

fn main() {
    let mut tree = Node::new();
    tree.insert(Method::GET, "/", "GET");
    tree.insert(Method::POST, "/", "POST");
    tree.insert(Method::PUT, "/", "PUT");
    tree.insert(Method::DELETE, "/", "DELETE");

    tree.update(Method::GET, "/", "UPDATE GET");
    tree.update(Method::POST, "/", "UPDATE POST");
    tree.update(Method::PUT, "/", "UPDATE PUT");
    tree.update(Method::DELETE, "/", "UPDATE DELETE");

    let result = tree.search(Method::GET, "/");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"UPDATE GET");

    let result = tree.search(Method::POST, "/");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"UPDATE POST");

    let result = tree.search(Method::PUT, "/");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"UPDATE PUT");

    let result = tree.search(Method::DELETE, "/");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"UPDATE DELETE");
}
```

### Remove

```rust
use radixtree::{Node, Method};

fn main() {
    let mut tree = Node::new();
    tree.insert(Method::GET, "/", "GET");
    tree.insert(Method::POST, "/", "POST");
    tree.insert(Method::PUT, "/", "PUT");
    tree.insert(Method::DELETE, "/", "DELETE");

    tree.remove("/");

    let result = tree.search(Method::GET, "/");
    assert!(result.is_none());

    let result = tree.search(Method::POST, "/");
    assert!(result.is_none());

    let result = tree.search(Method::PUT, "/");
    assert!(result.is_none());

    let result = tree.search(Method::DELETE, "/");
    assert!(result.is_none());
}
```

### Parameter Paths

```rust
use radixtree::{Node, Method};

fn main() {
    let mut tree = Node::new();
    tree.insert(Method::GET, "/user/$id", "GET");

    let result = tree.search(Method::GET, "/user/1");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"GET");

    tree.update(Method::GET, "/user/$id", "UPDATE GET");

    let result = tree.search(Method::GET, "/user/1");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"UPDATE GET");

    tree.remove("/user/$id");

    let result = tree.search(Method::GET, "/user/1");
    assert!(result.is_none());
}
```

### Wildcard Star

```rust
use radixtree::{Node, Method};

fn main() {
    let mut tree = Node::new();
    tree.insert(Method::GET, "/image/*", "GET");

    let result = tree.search(Method::GET, "/image/hello.jpeg");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"GET");

    let result = tree.search(Method::GET, "/image/jpg/hello.jpg");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"GET");

    let result = tree.search(Method::GET, "/image/png/hello.png");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"GET");

    tree.update(Method::GET, "/image/*", "UPDATE GET");

    let result = tree.search(Method::GET, "/image/hello.jpeg");
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), &"UPDATE GET");

    tree.remove("/image/*");

    let result = tree.search(Method::GET, "/image/hello.jpeg");
    assert!(result.is_none());
}
```

### Match Rules

Some examples of valid URL paths are:

- /abc/def
- /favicon.ico
- /user/$id
- /id/$id/name/$name
- /$year/$month
- /$year/$month/$day
- /images/*
- /images/$category/*

Note that all of the above URL paths may exist in the radix tree at the same time.

Paths starting with `$` indicate parameter paths. Parameter paths only match a single path segment. 
That is, the path `/user/$id` will match on `/user/1` or `/user/2`, but not `/user/1/2`.

Wildcard `*` can match any path. For example, the path `/image/*` will match on `/image/png/hello.png`,
`/image/jpg/hello.jpg` or `image/hello.jpeg`.

### Match Priority

1. Static paths take the highest priority.
2. Parameter paths take second priority.
3. Finally, the wildcard `*` matches paths where static paths and parameter paths do not match.

## Author

Zhenwei Guo (Ilqjx)

## License

This project is licensed under the [MIT license](https://github.com/Ilqjx/radixtree/blob/master/LICENSE).
