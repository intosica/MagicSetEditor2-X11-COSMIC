//! `mse_core` – UI-independent engine for a Magic Set Editor 2 compatible tool.
//!
//! * [`mse_format`] – parser for MSE's tab-indented `key: value` text format
//! * [`set`] / [`card`] – `.mse-set` (zip) loading
//! * [`template`] – game + stylesheet definitions
//! * [`symbol`] – set-symbol rendering (with a *correct* border pass)
//! * [`render`] – card composition to RGBA images

pub mod card;
pub mod error;
pub mod mse_format;
pub mod render;
pub mod set;
pub mod symbol;
pub mod template;

pub use card::Card;
pub use error::{Error, Result};
pub use set::Set;
pub use template::Template;
