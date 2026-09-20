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
mod gif_decoder;
mod reader;

pub use gif_decoder::{GifDecoder, GifFrameIterator};

pub use crate::common::{AnyExtension, DisposalMethod, Extension, Frame, Repeat};

pub use crate::reader::{ColorOutput, MemoryLimit};
pub use crate::reader::{DecodeOptions, Decoder, Version};
pub use crate::reader::{DecodingError, DecodingFormatError, EmbeddedIoError};

/// Low-level, advanced decoder. Prefer [`Decoder`] instead, which can stream frames too.
pub mod streaming_decoder {
    pub use crate::common::Block;
    pub use crate::reader::{Decoded, FrameDataType, FrameDecoder, OutputBuffer, StreamingDecoder};
}
