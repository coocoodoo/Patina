//! Patina core: rendering, layout and windowing for the Patina UI toolkit.
//!
//! The crate is consumed through its C ABI (see `include/patina.h` and [`ffi`]); the Rust
//! modules are public so the offscreen renderer can be used directly from Rust as well.

#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::field_reassign_with_default
)]

pub mod anim;
pub mod app;
pub mod color;
pub mod ffi;
pub mod geom;
pub mod input;
pub mod layout;
pub mod node;
pub mod paint;
pub mod palettes;
pub mod props;
pub mod render;
pub mod smil;
pub mod state;
pub mod svg;
pub mod text;
pub mod theme;

pub use node::{Id, Kind};
pub use state::{State, lock};
