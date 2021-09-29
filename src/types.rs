
use hdk::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEntityInput {
    pub id: EntryHash,
}

impl GetEntityInput {
    pub fn new(id: EntryHash) -> Self {
	GetEntityInput {
	    id: id,
	}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEntityInput<T> {
    pub addr: EntryHash,
    pub properties: T,
}
