#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ArcInfo {
    pub hash: u64,
    pub filesize: u32,
    pub extension: u64,
    pub orig_decomp_size: u32,
}