use std::slice;
use std::path::PathBuf;

#[repr(C)]
//#[derive(Debug, Clone)]
pub struct ArcPath {
    pub path: *const u8,
    pub path_len: u8,
}

impl ArcPath {
    pub fn new<T: AsRef<[u8]>>(filepath: T) -> Self {
        ArcPath {
            path: filepath.as_ref().as_ptr(),
            path_len: filepath.as_ref().len() as _,
        }
    }

    pub fn path(&self) -> Option<PathBuf> {
        unsafe {
            if self.path.is_null() {
                return None
            }

            let path = match std::str::from_utf8(slice::from_raw_parts(self.path, self.path_len as _)) {
                Ok(x) => x,
                Err(err) => panic!(err),
            };
            
            Some(PathBuf::from(path))
        }
    }
}