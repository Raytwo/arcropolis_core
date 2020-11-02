#![feature(str_strip)]


use std::collections::HashMap;

use crate::core::FileCtx;
use crate::core::callbacks::{ ArcCallback, CallbackType };

lazy_static::lazy_static! {
    pub(crate) static ref ARC_FILES: parking_lot::RwLock<HashMap<u64, FileCtx>> = parking_lot::RwLock::new(HashMap::new());
    pub(crate) static ref CALLBACKS: parking_lot::RwLock<HashMap<CallbackType, Vec<ArcCallback>>> = parking_lot::RwLock::new(HashMap::new());
}

// TODO: Move this to ArcFiles impl?
pub(crate) fn add_entries(entries: Vec<FileCtx>)  {
    let arc_file = &mut ARC_FILES.write();

    for entry in entries {
        arc_file.insert(entry.hash, entry);
    }
}

mod files;
pub mod api;
pub mod arc;
pub mod core;