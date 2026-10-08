//! The openOMSI plugin compiler, as a library. The `oopc` binary is a thin CLI over this; the
//! devtools app reuses the same building, packing, signing and inspection logic.

pub mod build;
pub mod commands;
pub mod manifest;
pub mod templates;
pub mod util;
