//! What goes over the serial console: commands typed in, and thermal images
//! printed out as text.

#![cfg_attr(not(test), no_std)]

pub mod command;
pub mod image;

pub use command::{Command, ParseError};
