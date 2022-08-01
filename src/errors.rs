use std::convert::Infallible;
use thiserror::Error;
use hdk::prelude::*;

/// The potential Error types for this CRUD library
#[derive(Error, Debug)]
pub enum UtilsError {
    /// A catch all enum for errors raised by HDK methods
    #[error("HDK raised error: {0:?}")]
    HDKError(WasmError),

    #[error("Unexpected State: {0:?}")]
    UnexpectedState(String),

    /// This means the fetched Record could not be deserialized to the given entry type
    #[error("Failed to deserialize to entry type '{0}': {1:?}")]
    DeserializationError(&'static str, Option<EntryType>),

    /// This indicates that the deserialized EntryType and the Record EntryType do not match
    #[error("Deserialized entry to wrong type: expected {}/{} but found {}/{} (zome/entry)", (.0).0, (.1).0, (.2).0, (.3).0 )]
    WrongEntryTypeError(ZomeId, EntryDefIndex, ZomeId, EntryDefIndex),

    /// A Record was expected to have an App Entry but it does not
    #[error("The Record @ {1} has the action type {0}; expected a Create or Update type")]
    RecordHasNoEntry(ActionHash, ActionType),

    /// Received 'None' when attempting to fetch a Record via EntryHash
    #[error("Record not found for Entry address '{0}': {}", .1.to_owned().unwrap_or("".to_string()) )]
    EntryNotFoundError(EntryHash, Option<String>),

    /// Received 'None' when attempting to fetch a Record via ActionHash
    #[error("Record not found for Action address '{0}': {}", .1.to_owned().unwrap_or("".to_string()) )]
    ActionNotFoundError(ActionHash, Option<String>),

    /// This functions as an integrity check to ensure the CRUD model is understood
    #[error("The given Action address ({0}) is not a Create action type")]
    NotOriginEntryError(ActionHash),

    // /// Indicates that the CRUD model was broken because there are multiple links with the tag
    // /// 'origin'
    // #[error("Found multiple origin links for entry: {0:?}")]
    // MultipleOriginsError(EntryHash),
}

impl From<UtilsError> for WasmError  {
    fn from(error: UtilsError) -> Self {
	wasm_error!(WasmErrorInner::Guest(format!("{}", error)))
    }
}

impl From<WasmError> for UtilsError  {
    fn from(error: WasmError) -> Self {
        UtilsError::HDKError(error)
    }
}

impl From<Infallible> for UtilsError {
    fn from(error: Infallible) -> Self {
	UtilsError::HDKError(error.into())
    }
}

/// The Result type for `Result<T, UtilsError>` ([UtilsError])
pub type UtilsResult<T> = Result<T, UtilsError>;
