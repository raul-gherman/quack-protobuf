//! A crate to read binary protobuf files

#![cfg_attr(not(feature = "std"), no_std)]

pub mod errors;
pub mod message;
pub mod reader;
pub mod sizeof;
pub mod writer;

pub use crate::{
    errors::{Error, Result},
    message::{MessageInfo, MessageRead, MessageWrite},
    reader::{decode, BytesReader},
    writer::{BytesWriter, Writer, WriterBackend},
};
