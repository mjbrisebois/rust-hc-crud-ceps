
mod errors;
mod entities;
mod types;

use hdk::prelude::*;

pub use entities::{ Collection, Entity, EntityType, EntryModel };
pub use types::{ UpdateEntityInput, GetEntityInput };
pub use errors::{ UtilsResult, UtilsError };


pub const TAG_UPDATE: &'static str = "update";
pub const TAG_ORIGIN: &'static str = "origin";


pub fn now() -> UtilsResult<u64> {
    sys_time()
	.map( |t| (t.as_micros() / 1000) as u64 )
	.map_err(UtilsError::HDKError)
}

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

pub fn fetch_entry_latest(id: EntryHash) -> UtilsResult<(HeaderHash, Element)> {
    //
    // - Get event details `ElementDetails` (expect to be a Create or Update)
    // - If it has updates, select the one with the latest Header timestamp
    // - Get the update element
    //
    let result = get(id.clone(), GetOptions::latest())
	.map_err(UtilsError::HDKError)?;

    let mut element = result.ok_or(UtilsError::EntryNotFoundError(id.clone()))?;

    let update_links = get_links(id.clone(), Some(LinkTag::new(TAG_UPDATE)))
	.map_err(UtilsError::HDKError)?.into_inner();

    debug!("Found {} update links for entry: {}", update_links.len(), id );
    if update_links.len() > 0 {
	let latest_link = find_latest_link( update_links ).unwrap();

	debug!("Determined ({}) is the newest update for entry: {}", latest_link.target, id );
	if let Some(v) = get( latest_link.target, GetOptions::latest() ).map_err(UtilsError::HDKError)? {
	    element = v;
	}
    }

    Ok( (element.header_address().to_owned(), element) )
}

pub fn get_id_for_addr(addr: EntryHash) -> UtilsResult<EntryHash> {
    let parent_links = get_links(addr.clone(), Some(LinkTag::new(TAG_ORIGIN)))
	.map_err(UtilsError::HDKError)?.into_inner();

    match parent_links.len() {
	0 => Ok( addr ),
	1 => Ok( parent_links.first().unwrap().target.clone() ),
	_ => Err( UtilsError::MultipleOriginsError(addr) ),
    }
}

pub fn fetch_entry(addr: EntryHash) -> UtilsResult<(HeaderHash, Element)> {
    let element = get(addr.clone(), GetOptions::latest())
	.map_err( UtilsError::HDKError )?
	.ok_or( UtilsError::EntryNotFoundError(addr.clone()) )?;

    Ok( (element.header_address().to_owned(), element) )
}

pub fn get_entity(id: &EntryHash) -> UtilsResult<Entity<Element>> {
    let (header_hash, element) = fetch_entry_latest( id.clone() )?;

    let address = element
	.header()
	.entry_hash()
	.ok_or(UtilsError::EntryNotFoundError(id.clone()))?;

    Ok(Entity {
	id: id.clone(),
	header: header_hash,
	address: address.to_owned(),
	ctype: EntityType::new( "element", "entry" ),
	content: element,
    })
}


pub fn create_entity<T>(input: &T) -> UtilsResult<Entity<T>>
where
    T: Clone + EntryModel,
    CreateInput: TryFrom<T, Error = WasmError>,
    Entry: TryFrom<T, Error = WasmError>,
{
    let create_input = CreateInput::try_from( input.to_owned() )
	.map_err(UtilsError::HDKError)?;

    let entry_hash = hash_entry( input.to_owned() )
	.map_err(UtilsError::HDKError)?;

    let header_hash = create( create_input )
	.map_err(UtilsError::HDKError)?;

    Ok( Entity {
	id: entry_hash.clone(),
	address: entry_hash,
	header: header_hash,
	ctype: input.get_type(),
	content: input.to_owned(),
    } )
}

pub fn update_entity<T, F>(id: Option<EntryHash>, addr: EntryHash, callback: F) -> UtilsResult<Entity<T>>
where
    T: Clone + EntryModel + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    CreateInput: TryFrom<T, Error = WasmError>,
    Entry: TryFrom<T, Error = WasmError>,
    F: FnOnce(Element) -> UtilsResult<T>,
{
    let id = match id {
	Some(id) => id,
	None => {
	    get_id_for_addr( addr.clone() )?
	},
    };

    let (header, element) = fetch_entry( addr.clone() )?;

    debug!("Found element (header): {:?}", element.header() );
    let current = match T::try_from( element.clone() ) {
	Ok(entry) => entry,
	Err(_) => {
	    return Err(UtilsError::DeserializationError( addr, T::entry_def_id() ));
	},
    };

    // TODO: Compare element entry type to updated entry type rather than this hack way of rehashing
    // the entry.
    //
    // If rehashing does not result in the same entry hash, that means the entry bytes
    // coincidentally deserialized into the expected struct.  Some fields would be missing which
    // results in a different hash.
    let current_hash = hash_entry( current.clone() )?;

    debug!("Verifiying src matches entry type: {:?} == {:?}", addr, current_hash );
    if addr != current_hash {
	return Err(UtilsError::WrongEntryTypeError( addr, current_hash, T::entry_def_id() ));
    }

    let updated_entry = callback( element.clone() )?;


    let create_input = CreateInput::try_from( updated_entry.clone() )
	.map_err(UtilsError::HDKError)?;

    let entry_hash = hash_entry( updated_entry.to_owned() )
	.map_err(UtilsError::HDKError)?;

    let header_hash = update( header, create_input )
	.map_err(UtilsError::HDKError)?;

    debug!("Linking original ({}) to DNA: {}", id, entry_hash );
    create_link(
	id.clone(),
	entry_hash.clone(),
	LinkTag::new(TAG_UPDATE)
    ).map_err(UtilsError::HDKError)?;

    debug!("Linking DNA ({}) to original: {}", entry_hash, id );
    create_link(
	entry_hash.clone(),
	id.clone(),
	LinkTag::new(TAG_ORIGIN)
    ).map_err(UtilsError::HDKError)?;

    Ok(	Entity {
	id: id,
	header: header_hash,
	address: entry_hash,
	ctype: updated_entry.get_type(),
	content: updated_entry,
    } )
}
