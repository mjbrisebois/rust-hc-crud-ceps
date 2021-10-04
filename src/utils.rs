
use hdk::prelude::{
    debug, sys_time, hash_entry,
    Element, Entry, Link, EntryHash, WasmError,
    TryFrom, EntryDefRegistration,
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
