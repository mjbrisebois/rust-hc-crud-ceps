
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


pub fn get_id_for_addr(addr: &EntryHash) -> UtilsResult<EntryHash> {
    let parent_links = get_links(addr.to_owned(), Some(LinkTag::new(TAG_ORIGIN)))
	.map_err(UtilsError::HDKError)?.into_inner();

    debug!("Found {} parent links for address {:?}", parent_links.len(), addr );
    match parent_links.len() {
	0 => Ok( addr.to_owned() ),
	1 => Ok( parent_links.first().unwrap().target.to_owned() ),
	_ => Err( UtilsError::MultipleOriginsError(addr.to_owned()) ),
    }
}


pub fn fetch_element(addr: &EntryHash) -> UtilsResult<(HeaderHash, Element)> {
    let element = get( addr.to_owned(), GetOptions::latest() )
	.map_err( UtilsError::HDKError )?
	.ok_or( UtilsError::EntryNotFoundError(addr.to_owned()) )?;

    Ok( (element.header_address().to_owned(), element) )
}


pub fn check_entry_type<T>(element: &Element, addr: &EntryHash) -> UtilsResult<T>
where
    T: Clone + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    Entry: TryFrom<T, Error = WasmError>,
{
    debug!("Found element (header): {:?}", element.header().entry_type() );
    let entry = match T::try_from( element.clone() ) {
	Ok(entry) => entry,
	Err(_) => {
	    return Err(UtilsError::DeserializationError( addr.to_owned(), T::entry_def_id() ));
	},
    };

    // TODO: Compare element entry type to updated entry type rather than this hack way of rehashing
    // the entry.
    //
    // If rehashing does not result in the same entry hash, that means the entry bytes
    // coincidentally deserialized into the expected struct.  Some fields would be missing which
    // results in a different hash.
    let entry_hash = hash_entry( entry.clone() )?;

    debug!("Verifiying src matches entry type: {:?} == {:?}", addr, entry_hash );
    if *addr != entry_hash {
	return Err(UtilsError::WrongEntryTypeError( addr.to_owned(), entry_hash, T::entry_def_id() ));
    }

    Ok( entry )
}


pub fn fetch_element_latest(id: &EntryHash) -> UtilsResult<(HeaderHash, Element, Element)> {
    //
    // - Get event details `ElementDetails` (expect to be a Create or Update)
    // - If it has updates, select the one with the latest Header timestamp
    // - Get the update element
    //
    let origin_addr = get_id_for_addr( id )?;
    if *id != origin_addr {
	return Err(UtilsError::NotOriginEntryError( id.to_owned(), origin_addr ));
    }

    let result = get(id.to_owned(), GetOptions::latest())
	.map_err(UtilsError::HDKError)?;

    let mut element = result.ok_or(UtilsError::EntryNotFoundError(id.to_owned()))?;
    let origin_element = element.clone();

    let update_links = get_links(id.to_owned(), Some(LinkTag::new(TAG_UPDATE)))
	.map_err(UtilsError::HDKError)?.into_inner();

    debug!("Found {} update links for entry: {}", update_links.len(), id );
    if update_links.len() > 0 {
	let latest_link = find_latest_link( update_links ).unwrap();

	debug!("Determined ({}) is the newest update for entry: {}", latest_link.target, id );
	if let Some(v) = get( latest_link.target, GetOptions::latest() ).map_err(UtilsError::HDKError)? {
	    element = v;
	}
    }

    Ok( (element.header_address().to_owned(), element, origin_element) )
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

    Ok(Entity {
	id: entry_hash.to_owned(),
	address: entry_hash,
	header: header_hash,
	ctype: input.get_type(),
	content: input.to_owned(),
    })
}


pub fn get_entity<T>(id: &EntryHash) -> UtilsResult<Entity<T>>
where
    T: Clone + EntryModel + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    Entry: TryFrom<T, Error = WasmError>,
{
    let (header_hash, element, origin_element) = fetch_element_latest( id )?;

    debug!("Checking that ID {:?} is entry type {:?}", id, T::entry_def_id() );
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

    let create_input = CreateInput::try_from( updated_entry.clone() )
	.map_err(UtilsError::HDKError)?;

    let entry_hash = hash_entry( updated_entry.to_owned() )
	.map_err(UtilsError::HDKError)?;

    let header_hash = update( header, create_input )
	.map_err(UtilsError::HDKError)?;

    let id = get_id_for_addr( addr )?;

    debug!("Linking original ({}) to: {}", id, entry_hash );
    create_link(
	id.to_owned(),
	entry_hash.to_owned(),
	LinkTag::new(TAG_UPDATE)
    ).map_err(UtilsError::HDKError)?;

    debug!("Linking ({}) to original: {}", entry_hash, id );
    create_link(
	entry_hash.to_owned(),
	id.to_owned(),
	LinkTag::new(TAG_ORIGIN)
    ).map_err(UtilsError::HDKError)?;

    Ok(Entity {
	id: id,
	header: header_hash,
	address: entry_hash,
	ctype: updated_entry.get_type(),
	content: updated_entry,
    })
}


pub fn delete_entity<T>(id: &EntryHash) -> UtilsResult<HeaderHash>
where
    T: Clone + TryFrom<Element, Error = WasmError> + EntryDefRegistration,
    Entry: TryFrom<T, Error = WasmError>,
{
    let id = get_id_for_addr( id )?;
    let (header_hash, element) = fetch_element( &id )?;

    check_entry_type::<T>( &element, &id )?;
    delete_entry( header_hash.to_owned() )?;

    Ok( header_hash )
}



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
