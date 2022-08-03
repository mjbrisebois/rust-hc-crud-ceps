//! Other Resources
//!
//! - Source code - [github.com/mjbrisebois/rust-hc-crud-ceps](https://github.com/mjbrisebois/rust-hc-crud-ceps/)
//! - Cargo package - [crates.io/crates/hc_crud_ceps](https://crates.io/crates/hc_crud_ceps)
//!

mod errors;
mod entities;
mod utils;

use std::convert::TryFrom;
use hdk::prelude::*;

pub use entities::{ Entity, EmptyEntity, EntityType, EntryModel };
pub use errors::{ UtilsResult, UtilsError };
pub use utils::{
    now, find_latest_link, path_from_collection,
    trace_action_history, to_entry_type,
};



/// Get the entity ID for any given entity EntryHash
pub fn get_origin_address(addr: &ActionHash) -> UtilsResult<EntryHash> {
    let chain = trace_action_history( addr )?;

    // The starting 'addr' will always be in the chain so it is safe to unwrap.
    Ok( chain.last().unwrap().1.to_owned() )
}

/// Get the record for any given EntryHash
pub fn fetch_record(addr: &EntryHash) -> UtilsResult<(ActionHash, Record)> {
    let record = get( addr.to_owned(), GetOptions::latest() )?
	.ok_or( UtilsError::EntryNotFoundError(addr.to_owned(), Some("".to_string())) )?;

    Ok( (record.action_address().to_owned(), record) )
}

/// Finds and returns the Action with the earliest timestamp from a list
pub fn find_earliest_action(updates: Vec<SignedHashed<Action>>) -> Option<SignedHashed<Action>> {
    if updates.len() == 0 {
	None
    }
    else {
	Some( updates.iter()
            .fold( None, |acc, sh| {
		let ts = sh.action().timestamp();
		match acc {
		    None => Some( (ts, sh.to_owned()) ),
		    Some(current) => {
			Some(match current.0 < ts {
			    true => current,
			    false => (ts, sh.to_owned()),
			})
		    }
		}
	    }).unwrap().1 )
    }
}


/// Follow the trail of (earliest) updates and return the full Action path.
pub fn follow_updates(hash: &ActionHash, trace: Option<Vec<ActionHash>>) -> UtilsResult<Vec<ActionHash>> {
    let mut history = trace.unwrap_or( Vec::new() );
    history.push( hash.to_owned() );

    let details = get_details( hash.to_owned(), GetOptions::latest() )?
	.ok_or( UtilsError::ActionNotFoundError(hash.to_owned(), Some("".to_string())) )?;
    let updates = match details {
	Details::Record(details) => details.updates,
	Details::Entry(details) => details.updates,
    };

    Ok( match find_earliest_action( updates ) {
	None => history,
	Some(next_update) => follow_updates( next_update.action_address(), Some(history) )?,
    })
}

/// Get the latest Record for any given entity ID
pub fn fetch_record_latest(id: &EntryHash) -> UtilsResult<(ActionHash, Record)> {
    let (action_hash, first_record) = fetch_record( id )?;

    match first_record.action() {
	Action::Create(_) => (),
	_ => Err(UtilsError::NotOriginEntryError(action_hash.to_owned()))?,
    }

    let updates = follow_updates( &action_hash, None )?;
    let latest_action_hash = updates.last().unwrap();
    let record = get( latest_action_hash.to_owned(), GetOptions::latest() )?
	.ok_or( UtilsError::ActionNotFoundError(action_hash.to_owned(), Some("".to_string())) )?;

    Ok( (action_hash, record) )
}



/// Create a new entity
pub fn create_entity<T,I,E>(entry: &T) -> UtilsResult<Entity<T>>
where
    ScopedEntryDefIndex: for<'a> TryFrom<&'a I, Error = E>,
    ScopedEntryDefIndex: TryFrom<I, Error = WasmError>,
    EntryVisibility: for<'a> From<&'a I>,
    Entry: TryFrom<I, Error = E>,
    Entry: TryFrom<T, Error = E>,
    WasmError: From<E>,
    T: Clone + EntryModel<I>,
{
    let entry_hash = hash_entry( entry.to_owned() )?;
    let action_hash = create_entry( entry.to_input() )?;

    Ok(Entity {
	id: entry_hash.to_owned(),
	address: entry_hash,
	action: action_hash,
	ctype: entry.get_type(),
	content: entry.to_owned(),
    })
}

/// Get an entity by its ID
pub fn get_entity<I,ET>(id: &EntryHash) -> UtilsResult<Entity<I>>
where
    I: TryFrom<Record, Error = WasmError> + Clone + EntryModel<ET>,
    Entry: TryFrom<I, Error = WasmError>,
    ScopedEntryDefIndex: TryFrom<ET, Error = WasmError>,
{
    let (_, record) = fetch_record_latest( id )?;
    let to_type_input = record.to_owned();
    let address = record
	.action()
	.entry_hash()
	.ok_or(UtilsError::EntryNotFoundError(id.to_owned(), None))?;

    let content : I = to_entry_type( to_type_input )?;

    Ok(Entity {
	id: id.to_owned(),
	action: record.action_address().to_owned(),
	address: address.to_owned(),
	ctype: content.get_type(),
	content: content,
    })
}

/// Update an entity
pub fn update_entity<T,I,F,E>(addr: &ActionHash, callback: F) -> UtilsResult<Entity<T>>
where
    ScopedEntryDefIndex: TryFrom<I, Error = WasmError>,
    Entry: TryFrom<I, Error = E>,
    Entry: TryFrom<T, Error = E>,
    WasmError: From<E>,
    T: TryFrom<Record, Error = WasmError>,
    T: Clone + EntryModel<I>,
    F: FnOnce(T, Record) -> UtilsResult<T>,
{
    // TODO: provide automatic check that the given address is the latest one or an optional flag
    // to indicate the intension to branch from an older update.
    let id = get_origin_address( &addr )?;
    let record = get( addr.to_owned(), GetOptions::latest() )?
	.ok_or( UtilsError::ActionNotFoundError(addr.to_owned(), Some("Given origin for update is not found".to_string())) )?;

    let current : T = to_entry_type( record.clone() )?;
    let updated_entry = callback( current, record.clone() )?;

    let entry_hash = hash_entry( updated_entry.to_owned() )?;
    let action_hash = update_entry( addr.to_owned(), updated_entry.to_input() )?;

    Ok(Entity {
	id: id,
	action: action_hash,
	address: entry_hash,
	ctype: updated_entry.get_type(),
	content: updated_entry,
    })
}

/// Delete an entity
pub fn delete_entity<T,ET>(id: &EntryHash) -> UtilsResult<ActionHash>
where
    T: TryFrom<Record, Error = WasmError> + Clone + EntryModel<ET>,
    Entry: TryFrom<T, Error = WasmError>,
    ScopedEntryDefIndex: TryFrom<ET, Error = WasmError>,
{
    let (action_hash, record) = fetch_record( id )?;
    let _ : T = to_entry_type( record )?;
    let delete_hash = delete_entry( action_hash )?;

    Ok( delete_hash )
}


/// Get multiple entities for a given base and link tag filter
pub fn get_entities<T,LT,ET>(id: &EntryHash, link_type: LT, tag: Option<Vec<u8>>) -> UtilsResult<Vec<Entity<T>>>
where
    T: TryFrom<Record, Error = WasmError> + Clone + EntryModel<ET>,
    LT: LinkTypeFilterExt,
    Entry: TryFrom<T, Error = WasmError>,
    ScopedEntryDefIndex: TryFrom<ET, Error = WasmError>,
{
    let links: Vec<Link> = get_links(
        id.to_owned(),
	link_type,
	tag.map( |tag| LinkTag::new( tag ) )
    )?.into();

    let list = links.into_iter()
	.filter_map(|link| {
	    get_entity( &link.target.into() ).ok()
	})
	.collect();

    Ok(list)
}
