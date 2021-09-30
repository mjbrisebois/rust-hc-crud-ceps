use hdk::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilsError {
    #[error("HDK raised error: {0:?}")]
    HDKError(WasmError),

    #[error("Entry not found for address: {0:?}")]
    EntryNotFoundError(EntryHash),

    #[error("Entry address ({0:?}) is an 'update'; Use origin address ({1:?}) as Entry ID")]
    NotOriginEntryError(EntryHash, EntryHash),

    #[error("Found multiple origin links for entry: {0:?}")]
    MultipleOriginsError(EntryHash),

    #[error("Failed to deserialize entry to type ({1:?}): {0:?}")]
    DeserializationError(EntryHash, EntryDefId),

    #[error("Deserialized entry to wrong type ({2:?}); hash mismatch: addr={0:?}, rehash={1:?}")]
    WrongEntryTypeError(EntryHash, EntryHash, EntryDefId),

    #[error("Link base ({0:?}) is not the expected type: {1:?}")]
    LinkBaseWrongTypeError(EntryHash, EntryDefId),
}

impl From<UtilsError> for WasmError  {
    fn from(error: UtilsError) -> Self {
        WasmError::Guest(format!("{}", error))
    }
}

impl From<WasmError> for UtilsError  {
    fn from(error: WasmError) -> Self {
        UtilsError::HDKError(error)
    }
}

pub type UtilsResult<T> = Result<T, UtilsError>;
