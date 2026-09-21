#![forbid(unsafe_code)]
#![allow(unknown_lints)] // Certain lints only apply to later versions of Rust
#![allow(clippy::manual_range_contains)]
#![allow(clippy::new_without_default)]
#![deny(clippy::alloc_instead_of_core)]
#![deny(clippy::std_instead_of_alloc)]
#![deny(clippy::std_instead_of_core)]
#![no_std]

#[macro_use]
extern crate alloc;

mod common;
mod reader;
mod wrapper;

pub use wrapper::*;

pub use crate::common::{
    AnyExtension, Block, DisposalMethod, Extension, GifFrame, GifRead, Repeat,
};

pub use crate::reader::{ColorOutput, MemoryLimit};
pub use crate::reader::{DecodeOptions, Decoder, DecoderIter};
pub use crate::reader::{
    Decoded, FrameDataType, FrameDecoder, OutputBuffer, StreamingDecoder, Version,
};
pub use crate::reader::{DecodingError, DecodingFormatError, EmbeddedIoError};
