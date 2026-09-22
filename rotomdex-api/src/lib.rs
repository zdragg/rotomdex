#![no_std]
#![recursion_limit = "256"]

extern crate alloc;

pub mod berries;
pub mod contests;
pub mod encounters;
pub mod evolution;
pub mod games;
pub mod items;
pub mod locations;
pub mod machines;
pub mod moves;
pub mod pokemon;
pub mod utility;

mod client;
pub use client::*;
mod error;
pub use error::*;

mod endpoint;
pub(crate) use endpoint::endpoint;

mod follow;
pub use follow::Follow;

pub mod model;
