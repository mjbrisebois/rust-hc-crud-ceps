use hdk::prelude::{
    create_link, get_links, delete_link,
    EntryHash, HeaderHash, LinkTag, Serialize, Deserialize,
};
use crate::errors::{ UtilsResult };


/// An Entity categorization format that required the name and model values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    /// An identifier for the type of data
    pub name: String,

    /// An identifier for the data's structure
    pub model: String,
}

/// Identifies a struct as an Entity model type
pub trait EntryModel {
    fn get_type(&self) -> EntityType;
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

    /// The create/update header of the current entry
    pub header: HeaderHash,

    /// The address of the current entry
    pub address: EntryHash,

    #[serde(rename = "type")]
    /// An identifier for the content's type and structure
    pub ctype: EntityType,

    /// The entity's current value
    pub content: T,
}

impl<T> Entity<T> {
    /// Replace the Entity content with another content struct that implements the [EntryModel] trait
    // TODO: As is, this method allows the entity type to change without warning even though the
    // function name implies that only the model should change.  How can this be fixed while trying
    // to avoid returning a Result type.
    pub fn change_model<F, M>(&self, transformer: F) -> Entity<M>
    where
	T: Clone,
	F: FnOnce(T) -> M,
	M: EntryModel
    {
	let content = transformer( self.content.clone() );

	Entity {
	    id: self.id.to_owned(),
	    header: self.header.to_owned(),
	    address: self.address.to_owned(),
	    ctype: content.get_type(),
	    content: content,
	}
    }

    /// Replace the Entity content with another content value and specify a custom model value
    pub fn change_model_custom<F, M>(&self, transformer: F) -> Entity<M>
    where
	T: Clone,
	F: FnOnce(T) -> (M, String),
    {
	let (content, model) = transformer( self.content.clone() );

	Entity {
	    id: self.id.to_owned(),
	    header: self.header.to_owned(),
	    address: self.address.to_owned(),
	    ctype: EntityType {
		name: self.ctype.name.to_owned(),
		model: model,
	    },
	    content: content,
	}
    }

    /// Link this entity to the given base with a specific tag.  Shortcut for [`hdk::prelude::create_link`]
    pub fn link_from(&self, base: &EntryHash, tag: Vec<u8>) -> UtilsResult<HeaderHash> {
	Ok( create_link( base.to_owned(), self.id.to_owned(), LinkTag::new( tag ) )? )
    }

    /// Link the given target to this entity with a specific tag.  Shortcut for [`hdk::prelude::create_link`]
    pub fn link_to(&self, target: &EntryHash, tag: Vec<u8>) -> UtilsResult<HeaderHash> {
	Ok( create_link( self.id.to_owned(), target.to_owned(), LinkTag::new( tag ) )? )
    }

    /// Delete a link matching the `current_base -[tag]-> target` and create a link with `new_base
    /// -[tag]-> target`
    // What happens if there are more than 1 matching links?  And is there a way to organize that
    // ensures we don't have multiple links to the same thing?
    pub fn move_link_from(&self, tag: Vec<u8>, current_base: &EntryHash, new_base: &EntryHash) -> UtilsResult<HeaderHash> {
	let tag = LinkTag::new( tag );
	let all_links = get_links(
            current_base.clone(),
	    Some( tag.clone() )
	)?;

	if let Some(current_link) = all_links.into_inner().into_iter().find(|link| {
	    link.target == self.id
	}) {
	    delete_link( current_link.create_link_hash )?;
	};

	Ok( create_link( new_base.to_owned(), self.id.to_owned(), tag )? )
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Empty {}

/// A general use entity definition for deserializing any entity input when the content is not
/// relevant.
pub type EmptyEntity = Entity<Empty>;


/// A list of items associated with a base EntryHash
#[derive(Debug, Serialize)]
pub struct Collection<T> {
    /// The base that was used in the [`hdk::prelude::get_links`] request
    pub base: EntryHash,

    /// A list of the values relative to the link results
    pub items: Vec<T>,
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::Rng;

    const PI_STR : &'static str = "primitive_inversed";

    #[test]
    fn entity_test() {
	let bytes = rand::thread_rng().gen::<[u8; 32]>();
	let ehash = holo_hash::EntryHash::from_raw_32( bytes.to_vec() );
	let hhash = holo_hash::HeaderHash::from_raw_32( bytes.to_vec() );

	let item = Entity {
	    id: ehash.clone(),
	    header: hhash,
	    address: ehash,
	    ctype: EntityType::new( "boolean", "primitive" ),
	    content: true,
	};

	assert_eq!( item.ctype.name, "boolean" );
	assert_eq!( item.ctype.model, "primitive" );


	let model = String::from( PI_STR );
	let new_item = item.change_model_custom( |current| {
	    ( !current, model.to_owned() )
	});

	assert_eq!( item.content, true );
	assert_eq!( new_item.content, false );
	assert_eq!( new_item.ctype.model, model );

	#[derive(Clone)]
	pub struct AnswerEntry {
	    pub answer: bool,
	}

	impl EntryModel for AnswerEntry {
	    fn get_type(&self) -> EntityType {
		EntityType::new( "answer", PI_STR )
	    }
	}

	let new_item = item.change_model( |current| {
	    AnswerEntry { answer: !current }
	});

	assert_eq!( item.content, true );
	assert_eq!( new_item.content.answer, false );
	assert_eq!( new_item.ctype.model, model );
    }
}
