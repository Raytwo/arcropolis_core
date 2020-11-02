#[repr(C)]
pub struct ArcFile {
    pub file: *mut u8,
    pub len: u32,
}