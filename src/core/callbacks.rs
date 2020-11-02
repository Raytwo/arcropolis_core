use crate::core::{ ArcInfo, ArcFile };

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum CallbackType {
    File(u64),
    /// Unused
    Directory(u64),
    Extension(u64),
}

#[repr(C, u8)]
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum ArcCallback {
    /// Does not work on Extension callbacks for the time being
    Listener(ListenerCallback),
    /// Editors are meant to edit the content of a file, either the game's or the user's. They can provide their own filesize ahead of time if they plan on replacing with a larger file. Size goes unused for now
    Editor(u32, EditorCallback),
    /// Replacers are meant to provide a file, no matter the size and have no intention of replacing one. You probably only want this for stream stuff, as other type don't play too well with that for the time being.
    Replacer(ReplacerCallback),
}

// Hash
pub type ListenerCallback = extern "C" fn(u64);
/// ArcInfo, out_ArcFile
pub type EditorCallback = extern "C" fn(*const ArcInfo, *mut ArcFile) -> bool;
/// Hash, out_ArcFile
pub type ReplacerCallback = extern "C" fn(u64, *mut ArcFile) -> bool;