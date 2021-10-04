//! # Basic Usage
//!
//!
//! ## CRUD Operations
//! These imports and structs are assumed for all examples
//! ```
//! use hdk::prelude::*;
//! use hc_crud::{
//!     Entity, Collection, EntryModel, EntityType,
//!     create_entity, get_entity, update_entity, delete_entity,
//!     now, get_origin_address, get_entities,
//! };
//!
//! #[hdk_entry(id = "post", visibility="public")]
//! #[derive(Clone)]
//! pub struct PostEntry {
//!     pub title: String,
//!     pub message: String,
//!     pub published_at: Option<u64>,
//!     pub last_updated: Option<u64>,
//! }
//! ```
//!
//! ### Create
//! Create an entry.
//!
//! Example
//! ```ignore
//! let input = PostEntry {
//!     title: String::from("Greeting"),
//!     message: String::from("Hello world!"),
//!     published_at: Some(1633108520744),
//!     last_updated: None,
//! };
//!
//! let post_entity = create_entity( &input )?;
//! ```
//!
//! ### Read
//! Get an entry.
//!
//! Example
//! ```ignore
//! let post_entity = get_entity( &entity.id )?;
//! ```
//!
//! ### Update
//! Update an entry.
//!
//! Example
//! ```ignore
//! let post_entity = update_entity( &entity.address, |mut previous: PostEntry, _| {
//!     previous.message = String::from("Hello, world!");
//!     previous.last_updated = Some(1633108520744);
//!     Ok(previous)
//! })?;
//! ```
//!
//! ### Delete
//! Delete an entry.
//!
//! Example
//! ```ignore
//! delete_entity::<PostEntry>( &entity.id )?;
//! ```

mod errors;
mod entities;
mod utils;

use hdk::prelude::{
    debug,
    hash_entry, get, create_entry, update_entry, delete_entry, create_link, get_links,
    Element, Entry, Link, LinkTag, EntryHash, HeaderHash, WasmError,
    TryFrom, EntryDefRegistration, CreateInput, GetOptions,
};

pub use entities::{ Collection, Entity, EntityType, EntryModel };
pub use errors::{ UtilsResult, UtilsError };
pub use utils::{ now, find_latest_link, check_entry_type };


/// The down-link tag for Entity life-cycles
pub const TAG_UPDATE: &'static str = "update";
/// The up-link tag for Entity life-cycles
pub const TAG_ORIGIN: &'static str = "origin";

/// Get the entity ID for any given entity address
pub fn get_origin_address(addr: &EntryHash) -> UtilsResult<EntryHash> {
    let origin_links = get_links(addr.to_owned(), Some(LinkTag::new(TAG_ORIGIN)))?.into_inner();

    debug!("Found {} [tag: {}] links for address {:?}", origin_links.len(), TAG_ORIGIN, addr );
    match origin_links.len() {
	0 => Ok( addr.to_owned() ),
	1 => Ok( origin_links.first().unwrap().target.to_owned() ),
	_ => Err( UtilsError::MultipleOriginsError(addr.to_owned()) ),
    }
}

/// Get the element for any given EntryHash
pub fn fetch_element(addr: &EntryHash) -> UtilsResult<(HeaderHash, Element)> {
    let element = get( addr.to_owned(), GetOptions::latest() )?
	.ok_or( UtilsError::EntryNotFoundError(addr.to_owned()) )?;

    Ok( (element.header_address().to_owned(), element) )
}

/// Get the latest element for any given entity ID
pub fn fetch_element_latest(id: &EntryHash) -> UtilsResult<(HeaderHash, Element, Element)> {
    //
    // - Get event details `ElementDetails` (expect to be a Create or Update)
    // - If it has updates, select the one with the latest Header timestamp
    // - Get the update element
    //
    let origin_addr = get_origin_address( id )?;
    if *id != origin_addr {
	return Err(UtilsError::NotOriginEntryError( id.to_owned(), origin_addr ));
    }

    let (header_addr, mut element) = fetch_element( &id )?;

    let origin_element = element.clone();

    let update_links = get_links(id.to_owned(), Some(LinkTag::new(TAG_UPDATE)))?.into_inner();

    debug!("Found {} [tag: {}] links for entry: {}", update_links.len(), TAG_UPDATE, id );
    if update_links.len() > 0 {
	let latest_link = find_latest_link( update_links ).unwrap();

	debug!("Determined {:?} is be the newest state for ID: {:?}", latest_link.target, id );
	if let Some(v) = get( latest_link.target, GetOptions::latest() )? {
	    element = v;
	}
    }

    Ok( (header_addr, element, origin_element) )
}


/// Create a new entity
pub fn create_entity<T>(input: &T) -> UtilsResult<Entity<T>>
where
    T: Clone + EntryModel,
    CreateInput: TryFrom<T, Error = WasmError>,
    Entry: TryFrom<T, Error = WasmError>,
{
    let entry_hash = hash_entry( input.to_owned() )?;
    let header_hash = create_entry( input.to_owned() )?;

    Ok(Entity {
	id: entry_hash.to_owned(),
	address: entry_hash,
	header: header_hash,
	ctype: input.get_type(),
	content: input.to_owned(),
    })
}

/// Get an entity by its ID
pub fn get_entity<T>(id: &EntryHash) -> UtilsResult<Entity<T>>
where
    T: Clone + EntryModel + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    Entry: TryFrom<T, Error = WasmError>,
{
    let (header_hash, element, origin_element) = fetch_element_latest( id )?;

    check_entry_type::<T>( &origin_element, id )?;

    let address = element
	.header()
	.entry_hash()
	.ok_or(UtilsError::EntryNotFoundError(id.to_owned()))?;

    let content = T::try_from( element.clone() )?;

    Ok(Entity {
	id: id.to_owned(),
	header: header_hash,
	address: address.to_owned(),
	ctype: content.get_type(),
	content: content,
    })
}

/// Update an entity
pub fn update_entity<T, F>(addr: &EntryHash, callback: F) -> UtilsResult<Entity<T>>
where
    T: Clone + EntryModel + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    CreateInput: TryFrom<T, Error = WasmError>,
    Entry: TryFrom<T, Error = WasmError>,
    F: FnOnce(T, Element) -> UtilsResult<T>,
{
    let (header, element) = fetch_element( addr )?;

    let current = check_entry_type( &element, addr )?;
    let updated_entry = callback( current, element.clone() )?;

    let entry_hash = hash_entry( updated_entry.to_owned() )?;
    let header_hash = update_entry( header, updated_entry.to_owned() )?;

    let id = get_origin_address( addr )?; // Get origin address for up/down linking
    debug!("Down-link from origin to update: {:?} -[tag: {}]-> {:?}", id, TAG_UPDATE, entry_hash );
    create_link(
	id.to_owned(),
	entry_hash.to_owned(),
	LinkTag::new(TAG_UPDATE)
    )?;
    debug!("Up-link from update to origin: {:?} -[tag: {}]-> {:?}", entry_hash, TAG_ORIGIN, id );
    create_link(
	entry_hash.to_owned(),
	id.to_owned(),
	LinkTag::new(TAG_ORIGIN)
    )?;

    Ok(Entity {
	id: id,
	header: header_hash,
	address: entry_hash,
	ctype: updated_entry.get_type(),
	content: updated_entry,
    })
}

/// Delete an entity
pub fn delete_entity<T>(id: &EntryHash) -> UtilsResult<HeaderHash>
where
    T: Clone + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    Entry: TryFrom<T, Error = WasmError>,
{
    let id = get_origin_address( id )?;
    let (header_hash, element) = fetch_element( &id )?;

    check_entry_type::<T>( &element, &id )?;
    delete_entry( header_hash.to_owned() )?;

    Ok( header_hash )
}


/// Get multiple entities for a given base and link tag filter
pub fn get_entities<B, T>(id: &EntryHash, link_tag: LinkTag) -> UtilsResult<Collection<Entity<T>>>
where
    B: Clone + EntryModel + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    T: Clone + EntryModel + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    Entry: TryFrom<T, Error = WasmError>,
    Entry: TryFrom<B, Error = WasmError>,
{
    match get_entity::<B>( id ) {
	Err(UtilsError::HDKError(WasmError::Serialize(_)))
	    | Err(UtilsError::DeserializationError(_, _)) => Err(UtilsError::LinkBaseWrongTypeError(id.to_owned(), B::entry_def_id())),
	x => x,
    }?;

    let links: Vec<Link> = get_links(
        id.to_owned(),
	Some(link_tag)
    )?.into();

    let list = links.into_iter()
	.filter_map(|link| {
	    get_entity( &link.target ).ok()
	})
	.collect();

    Ok(Collection {
	base: id.to_owned(),
	items: list,
    })
}
