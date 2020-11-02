use crate::arc::arc_path::ArcPath;

#[repr(C)]
struct ArcEntry {
    pub path: ArcPath,
    pub filesize: u32,
}

#[repr(C)]
struct ArcEntries {
    pub entries: *const ArcEntry,
    pub count: u16,
}