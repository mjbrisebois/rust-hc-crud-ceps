use std::convert::TryFrom;
use hdk::prelude::*;
use crate::errors::{
    UtilsResult, UtilsError,
};


/// An Entity categorization format that required the name and model values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    /// An identifier for the type of data
    pub name: String,

    /// An identifier for the data's structure
    pub model: String,
}

/// Identifies a struct as an Entity model type
pub trait EntryModel<T>
where
    ScopedEntryDefIndex: for<'a> TryFrom<&'a T, Error = WasmError>,
{
    fn name() -> &'static str;
    fn get_type(&self) -> EntityType;
    fn to_input(&self) -> T;
}

impl EntityType {
    pub fn new(name: &'static str, model: &'static str) -> Self {
	EntityType {
	    name: name.into(),
	    model: model.into(),
	}
    }
}


/// The context and content of a specific entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity<T> {
    /// The address of the original created entry
    pub id: EntryHash,

    /// The create/update action of the current entry
    pub action: ActionHash,

    /// The address of the current entry
    pub address: EntryHash,

    #[serde(rename = "type")]
    /// An identifier for the content's type and structure
    pub ctype: EntityType,

    /// The entity's current value
    pub content: T,
}

impl<T> Entity<T> {

    /// Link this entity to the given base with a specific tag.  Shortcut for [`hdk::prelude::create_link`]
    pub fn link_from<L,E>(&self, base: &EntryHash, link_type: L, tag_input: Option<Vec<u8>>) -> UtilsResult<ActionHash>
    where
	ScopedLinkType: TryFrom<L, Error = E>,
        WasmError: From<E>,
    {
        Ok( match tag_input {
	    None => create_link( base.to_owned(), self.id.to_owned(), link_type, () )?,
	    Some(input) => create_link( base.to_owned(), self.id.to_owned(), link_type, input )?,
	})
    }

    /// Link the given target to this entity with a specific tag.  Shortcut for [`hdk::prelude::create_link`]
    pub fn link_to<L,E>(&self, target: &EntryHash, link_type: L, tag_input: Option<Vec<u8>>) -> UtilsResult<ActionHash>
    where
	ScopedLinkType: TryFrom<L, Error = E>,
        WasmError: From<E>,
    {
        Ok( match tag_input {
	    None => create_link( self.id.to_owned(), target.to_owned(), link_type, () )?,
	    Some(input) => create_link( self.id.to_owned(), target.to_owned(), link_type, input )?,
	})
    }

    /// Delete an existing link from the 'current_base' and create a new link from the 'new_base'
    pub fn move_link_from<LT,E>(&self, link_type: LT, tag_input: Option<Vec<u8>>, current_base: &EntryHash, new_base: &EntryHash) -> UtilsResult<ActionHash>
    where
	LT: LinkTypeFilterExt + Clone + std::fmt::Debug,
        ScopedLinkType: TryFrom<LT, Error = E>,
        WasmError: From<E>,
    {
	let tag_filter = tag_input.to_owned().map( |tag| LinkTag::new( tag ) );
	let all_links = get_links(
	    current_base.clone(),
	    link_type.to_owned(),
	    tag_filter.to_owned(),
	)?;

	if let Some(current_link) = all_links.into_iter().find(|link| {
	    link.target == self.id.to_owned().into()
	}) {
            delete_link( current_link.create_link_hash )?;
	}
	else {
	    Err(UtilsError::UnexpectedState(format!("Aborting 'move_from_link' because existing link was not found")))?;
	};

	let new_links = get_links(
	    new_base.clone(),
	    link_type.to_owned(),
	    tag_filter.to_owned(),
	)?;

	if let Some(existing_link) = new_links.into_iter().find(|link| {
	    link.target == self.id.to_owned().into()
	}) {
            Ok( existing_link.create_link_hash )
	}
	else {
            self.link_from( new_base, link_type, tag_input )
	}
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Empty {}

/// A general use entity definition for deserializing any entity input when the content is not
/// relevant.
pub type EmptyEntity = Entity<Empty>;



#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn entity_test() {
	let bytes = rand::thread_rng().gen::<[u8; 32]>();
	let ehash = holo_hash::EntryHash::from_raw_32( bytes.to_vec() );
	let hhash = holo_hash::ActionHash::from_raw_32( bytes.to_vec() );

	let item = Entity {
	    id: ehash.clone(),
	    action: hhash,
	    address: ehash,
	    ctype: EntityType::new( "boolean", "primitive" ),
	    content: true,
	};

	assert_eq!( item.ctype.name, "boolean" );
	assert_eq!( item.ctype.model, "primitive" );
    }
}
