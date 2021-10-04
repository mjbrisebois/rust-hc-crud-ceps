[![](https://img.shields.io/crates/v/hc-crud-ceps?style=flat-square)](https://crates.io/crates/hc_crud_ceps)

# Holochain CRUD Library (CEPS pattern)
A CRUD library for Holochain zomes that implement the CEPS pattern (Chained, Entry, Permalink,
State-based)


[![](https://img.shields.io/github/issues-raw/mjbrisebois/rust-hc-crud-ceps?style=flat-square)](https://github.com/mjbrisebois/rust-hc-crud-ceps/issues)
[![](https://img.shields.io/github/issues-closed-raw/mjbrisebois/rust-hc-crud-ceps?style=flat-square)](https://github.com/mjbrisebois/rust-hc-crud-ceps/issues?q=is%3Aissue+is%3Aclosed)
[![](https://img.shields.io/github/issues-pr-raw/mjbrisebois/rust-hc-crud-ceps?style=flat-square)](https://github.com/mjbrisebois/rust-hc-crud-ceps/pulls)


### Holochain Version Map
For information on which versions of this package work for each Holochain release, see
[docs/Holochain_Version_Map.md](docs/Holochain_Version_Map.md)


## Overview

## Install

Example of adding to `Cargo.toml`
```toml
[dependencies]
hc_crud_ceps = "0.2.0"
```

Example of importing into your Rust file
```rust
use hc_crud::{
    now, get_origin_address, get_entities,
    create_entity, get_entity, update_entity, delete_entity,
    Entity, Collection, EntryModel, EntityType,
};
```


## Basic Usage

### CRUD Operations
These imports and structs are assumed for all examples
```rust
use hdk::prelude::*;
use hc_crud::{
    now, get_origin_address, get_entities,
    create_entity, get_entity, update_entity, delete_entity,
    Entity, Collection, EntryModel, EntityType,
};

#[hdk_entry(id = "post", visibility="public")]
#[derive(Clone)]
pub struct PostEntry {
    pub title: String,
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}
```

#### Create an entry

Example
```rust
let input = PostEntry {
    title: String::from("Greeting"),
    message: String::from("Hello world!"),
    published_at: Some(1633108520744),
    last_updated: None,
};

let post_entity = create_entity( &input )?;
```

#### [Read] Get an entry

Example
```rust
let post_entity = get_entity( &entity.id )?;
```

#### Update an entry

Example
```rust
let post_entity = update_entity( &entity.address, |mut previous: PostEntry, _| {
    previous.message = String::from("Hello, world!");
    previous.last_updated = Some(1633108520744);
    Ok(previous)
})?;
```

#### Delete an entry

Example
```rust
delete_entity::<PostEntry>( &entity.id )?;
```


### API Reference

See [docs.rs/hc_crud_ceps](https://docs.rs/hc_crud_ceps/)

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)
