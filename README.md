[![](https://img.shields.io/crates/v/hc_crud_ceps?style=flat-square)](https://crates.io/crates/hc_crud_ceps)

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
hc_crud_ceps = "0.3.0"
```

Example of common imports
```rust
use hc_crud::{
    now,
    create_entity, get_entity, get_entities, update_entity, delete_entity,
    Entity, EntryModel, EntityType,
};
```


## Basic Usage

### CRUD Operations
These imports and structs are assumed for all examples
```rust
use hdk::prelude::*;
use hc_crud::{
    now,
    create_entity, get_entity, get_entities, update_entity, delete_entity,
    Entity, EntryModel, EntityType,
};

#[hdk_entry_helper]
#[derive(Clone)]
pub struct PostEntry {
    pub title: String,
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

#[hdk_entry_defs]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    #[entry_def]
    Post(PostEntry),
}

impl EntryModel<EntryTypes> for PostEntry {
    fn name() -> &'static str { "Post" }
    fn get_type(&self) -> EntityType {
        EntityType::new( "post", "entry" )
    }
    fn to_input(&self) -> EntryTypes {
        EntryTypes::Post(self.clone())
    }
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
    previous.last_updated = Some( now()? );
    Ok(previous)
})?;
```

#### Delete an entry

Example
```rust
delete_entity::<PostEntry,EntryTypes>( &entity.id )?;
```


### Example of CRUD for relationships
Create a 1-to-many relationship for post entries to have comment entries.

The following examples use this additional struct
```rust
#[hdk_entry_helper]
#[derive(Clone)]
pub struct CommentEntry {
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

impl EntryModel<EntryTypes> for CommentEntry {
    fn name() -> &'static str { "Comment" }
    fn get_type(&self) -> EntityType {
        EntityType::new( "comment", "entry" )
    }
    fn to_input(&self) -> EntryTypes {
        EntryTypes::Comment(self.clone())
    }
}
```

Add `CommentEntry` to `EntryTypes` enum
```diff
 #[hdk_entry_defs]
 #[unit_enum(UnitEntryTypes)]
 pub enum EntryTypes {
     #[entry_def]
     Post(PostEntry),
+    #[entry_def]
+    Comment(CommentEntry),
 }
```

Create a `CommentEntry` and link it to the `PostEntry`
```rust
#[hdk_link_types]
pub enum LinkTypes {
    Post,
    Comment,
}

let input = CommentEntry {
    message: String::from("Where is the sun?"),
    published_at: Some( now()? ),
    last_updated: None,
};

let comment_entity = create_entity( &input )?;

comment_entity.link_from( &post_entity.id, LinkTypes::Comment, None )?;
```

Get a `Collection` for a specific base and tag
```rust
let collection : Vec<Entity<CommentEntry>> = get_entities( &post_entity.id, LinkTypes::Comment, None )?;
```


### API Reference

See [docs.rs/hc_crud_ceps](https://docs.rs/hc_crud_ceps/)

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)
