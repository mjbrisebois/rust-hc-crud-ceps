use hdk::prelude::*;
use crate::entities::{ EntryModel };
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


/// Verify a Record's entry is the expected entry type
///
/// - `T` - the expected entry type
/// - `record` - a Record expected to have an App entry
///
/// An entry type check could fail with:
///
/// - [`UtilsError::DeserializationError`] - indicating that it is the wrong entry type
/// - [`UtilsError::WrongEntryTypeError`] - indicating that the successful deserialization was a coincidence
///
/// ```ignore
/// let entry : T = to_entry_type( &record, &expected_hash )?
/// ```
pub fn to_entry_type<T,ET>(record: Record) -> UtilsResult<T>
where
    T: EntryModel<ET>,
    T: TryFrom<Record, Error = WasmError> + Clone,
    ScopedEntryDefIndex: for<'a> TryFrom<&'a ET, Error = WasmError>,
{
    let content = T::try_from( record.clone() )
	.map_err(|_| UtilsError::DeserializationError( T::name(), record.action().entry_type().map(|et| et.to_owned())) )?;
    let scoped_def = ScopedEntryDefIndex::try_from( &content.to_input() )?;

    if let Some(EntryType::App(AppEntryDef {zome_index, entry_index, ..})) = record.action().entry_type() {
	if *zome_index == scoped_def.zome_index && *entry_index == scoped_def.zome_type {
	    Ok(content)
	}
	else {
	    Err(UtilsError::WrongEntryTypeError(scoped_def.zome_index, scoped_def.zome_type, zome_index.to_owned(), entry_index.to_owned()))?
	}
    }
    else {
	Err(UtilsError::RecordHasNoEntry(record.action_address().to_owned(), record.action().action_type()))?
    }
}


/// Create a Path from any iterable list of items that implement the Display trait
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


fn trace_action_history_with_chain(action_hash: &ActionHash, history: Option<Vec<(ActionHash,EntryHash)>>) -> UtilsResult<Vec<(ActionHash,EntryHash)>> {
    let sh_action = must_get_action( action_hash.to_owned().into() )?;
    let mut history = history.unwrap_or( Vec::new() );

    match sh_action.action() {
	Action::Create(create) => {
	    history.push( (action_hash.to_owned(), create.entry_hash.to_owned()) );

	    Ok( history )
	},
	Action::Update(update) => {
	    history.push( (action_hash.to_owned(), update.entry_hash.to_owned()) );

	    trace_action_history_with_chain( &update.original_action_address, Some(history) )
	},
	action => Err(wasm_error!(WasmErrorInner::Guest(format!("Unexpected action type @ trace depth {}: {:?}", history.len(), action ))))?,
    }
}

/// Follow the Action's origin until we find the Create Action.
pub fn trace_action_history(action_hash: &ActionHash) -> UtilsResult<Vec<(ActionHash,EntryHash)>> {
    trace_action_history_with_chain(action_hash, None)
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
