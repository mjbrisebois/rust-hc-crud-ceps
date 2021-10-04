use hdk::prelude::{
    WasmError, EntryHash, EntryDefId
};
use thiserror::Error;

/// The potential Error types for this CRUD library
#[derive(Error, Debug)]
pub enum UtilsError {
    /// A catch all enum for errors raised by HDK methods
    #[error("HDK raised error: {0:?}")]
    HDKError(WasmError),

    /// Occurs when an HDK `get` fails for any reason
    #[error("Entry not found for address: {0:?}")]
    EntryNotFoundError(EntryHash),

    /// This functions as an integrity check to ensure the CRUD model is understood
    #[error("Entry address ({0:?}) is an 'update'; Use origin address ({1:?}) as Entry ID")]
    NotOriginEntryError(EntryHash, EntryHash),

    /// Indicates that the CRUD model was broken because there are multiple links with the tag
    /// 'origin'
    #[error("Found multiple origin links for entry: {0:?}")]
    MultipleOriginsError(EntryHash),

    /// This means the entry for the given address does not match expected type
    #[error("Failed to deserialize entry to type ({1:?}): {0:?}")]
    DeserializationError(EntryHash, EntryDefId),

    /// This indicates that an entry was not deserialized to the corrent entry type because it does
    /// not match the expected hash
    #[error("Deserialized entry to wrong type ({2:?}); hash mismatch: addr={0:?}, rehash={1:?}")]
    WrongEntryTypeError(EntryHash, EntryHash, EntryDefId),

    /// This functions as an integrity check to ensure the link base (EntryHash) is the expected
    /// type
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

/// The Result type for `Result<T, UtilsError>` ([UtilsError])
pub type UtilsResult<T> = Result<T, UtilsError>;
