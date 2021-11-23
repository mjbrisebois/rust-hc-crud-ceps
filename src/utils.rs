
use std::convert::TryFrom;
use hdk::prelude::{
    debug, sys_time, hash_entry,
    Element, Entry, Link, EntryHash, WasmError, Path,
    EntryDefRegistration,
};
use crate::errors::{ UtilsResult, UtilsError };

/// Get the current unix timestamp
pub fn now() -> UtilsResult<u64> {
    sys_time()
	.map( |t| (t.as_micros() / 1000) as u64 )
	.map_err(UtilsError::HDKError)
}

/// Find the latest link from a list of links
pub fn find_latest_link(links: Vec<Link>) -> Option<Link> {
    links
       .into_iter()
       .fold(None, |latest: Option<Link>, link: Link| match latest {
	   Some(latest) => {
	       if link.timestamp > latest.timestamp {
		   Some(link)
	       } else {
		   Some(latest)
	       }
	   },
	   None => Some(link),
       })
}

/// Verify an element's entry is the expected entry type
///
/// - `T` - the expected entry type
/// - `element` - an Element expected to have an App entry
/// - `addr` - the expected hash of the entry contents
///
/// An entry type check could fail with:
///
/// - [`UtilsError::DeserializationError`] - indicating that it is the wrong entry type
/// - [`UtilsError::WrongEntryTypeError`] - indicating that the successful deserialization was a coincidence
///
/// ```ignore
/// check_entry_type::<PostEntry>( &element, &expected_hash )?
/// ```
pub fn check_entry_type<T>(element: &Element, addr: &EntryHash) -> UtilsResult<T>
where
    T: Clone + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    Entry: TryFrom<T, Error = WasmError>,
{
    // TODO: Compare element entry type to updated entry type rather than this hack way of rehashing
    // the entry.
    //
    // If rehashing does not result in the same entry hash, that means the entry bytes
    // coincidentally deserialized into the expected struct.  Some fields would be missing which
    // results in a different hash.
    //
    // UPDATE:
    //
    //     The 'entry_defs()` method used to convert an entry def index to an entry def ID is
    //     defined by the 'entry_defs!' macro (see holochain/crates/hdk/src/entry.rs).  Which means
    //     it can't be accessed by this function.  It seems the only option would be to have an
    //     entry type registration function to pass this library the
    //     hdk::prelude::EntryDefsCallbackResult so it can be used by this method.  I don't like the
    //     idea of requiring that registration, so it could be an optional optimization where the
    //     current code is the default check.

    debug!("Checking that element entry type {:?} matches expected type {:?}", element.header().entry_type(), T::entry_def() );
    let entry = match T::try_from( element.clone() ) {
	Ok(entry) => entry,
	Err(_) => {
	    return Err(UtilsError::DeserializationError( addr.to_owned(), T::entry_def_id() ));
	},
    };

    let entry_hash = hash_entry( entry.clone() )?;

    debug!("Verifiying deserialized entry hashes matches expected hash: {:?} == {:?}", entry_hash, addr );
    if *addr != entry_hash {
	return Err(UtilsError::WrongEntryTypeError( addr.to_owned(), entry_hash, T::entry_def_id() ));
    }

    Ok( entry )
}


pub fn path_from_collection<T>(segments: T) -> UtilsResult<Path>
where
    T: IntoIterator,
    T::Item: std::fmt::Display,
{
    let components : Vec<hdk::hash_path::path::Component> = segments.into_iter()
	.map( |value| {
	    hdk::hash_path::path::Component::from( format!("{}", value ) )
	})
	.collect();

    Ok( Path::from( components ) )
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn path_from_collection_test() {
	let path = path_from_collection( vec!["some", "string", "path"] ).unwrap();

	assert_eq!( path, Path::from("some.string.path") );

	let bytes = rand::thread_rng().gen::<[u8; 32]>();
	let hash = holo_hash::EntryHash::from_raw_32( bytes.to_vec() );
	let items : Vec<Box<dyn std::fmt::Display>> = vec![ Box::new("some"), Box::new("string"), Box::new(hash.to_owned()) ];

	let path = path_from_collection( items ).unwrap();
	let path_manual = format!("some.string.{}", hash );

	println!("{:?} == {:?}", path, path_manual );
	assert_eq!( path, Path::from( path_manual ) );
    }
}
