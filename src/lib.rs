//! A library to read binary protobuf files
//!
//! This reader is developed similarly to a pull reader

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod errors;
pub mod message;
pub mod reader;
pub mod sizeofs;
pub mod writer;

pub use crate::{
    errors::{Error, Result},
    message::{MessageInfo, MessageRead, MessageWrite},
    reader::{decode, BytesReader, PackedFixed, PackedFixedIntoIter, PackedFixedRefIter},
    writer::{BytesWriter, Writer, WriterBackend},
};

#[cfg(feature = "std")]
pub use crate::reader::Reader;
#[cfg(feature = "std")]
pub use crate::writer::serialize_into_vec;
