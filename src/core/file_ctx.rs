use std::{fs, slice};
use std::path::{ Path, PathBuf };
use smash::resource::{LoadedTables, ResServiceState, SubFile};

#[repr(C)]
//#[derive(Debug, Clone)]
pub struct FileCtx {
    pub path: PathBuf,
    pub hash: u64,
    pub extension: u64,
    pub filesize: u32,
    pub virtual_file: bool,
    pub orig_subfile: SubFile,
}

impl FileCtx {
    pub fn new() -> Self {
        FileCtx {
            path: PathBuf::new(),
            hash: 0,
            extension: 0,
            filesize: 0,
            virtual_file: false,
            orig_subfile: SubFile {
                offset: 0,
                compressed_size: 0,
                decompressed_size: 0,
                flags: 0,
            },
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn arc_path(&self) -> PathBuf {
        let mut arc_path = self.path().to_str().unwrap()[69 + 1..].replace(";", ":");

        if let Some(regional_marker) = arc_path.find("+") {
            arc_path.replace_range(regional_marker..arc_path.find(".").unwrap(), "");
        }

        match arc_path.strip_suffix("mp4") {
            Some(x) => arc_path = format!("{}{}", x, "webm"),
            None => (),
        }

        PathBuf::from(&arc_path)
    }

    pub fn get_region(&self) -> u32 {
        // Default to the player's region index
        let mut region_index = ResServiceState::get_instance().regular_region_idx;

        // Make sure the file has an extension
        if let Some(_) = self.path.extension() {
            // Split the region identifier from the filepath
            let region = self.path.file_name().unwrap().to_str().unwrap().to_string();
            // Check if the filepath it contains a + symbol
            if let Some(region_marker) = region.find('+') {
                match &region[region_marker + 1..region_marker + 6] {
                    "jp_ja" => region_index = 0,
                    "us_en" => region_index = 1,
                    "us_fr" => region_index = 2,
                    "us_es" => region_index = 3,
                    "eu_en" => region_index = 4,
                    "eu_fr" => region_index = 5,
                    "eu_es" => region_index = 6,
                    "eu_de" => region_index = 7,
                    "eu_nl" => region_index = 8,
                    "eu_it" => region_index = 9,
                    "eu_ru" => region_index = 10,
                    "kr_ko" => region_index = 11,
                    "zh_cn" => region_index = 12,
                    "zh_tw" => region_index = 13,
                    _ => region_index = 1,
                }
            }
        }

        region_index
    }

    pub fn get_subfile(&self, t1_index: u32) -> &mut SubFile {
        let loaded_arc = LoadedTables::get_instance().get_arc();

        let file_info = loaded_arc.lookup_file_information_by_t1_index(t1_index);
        //let file_index = loaded_arc.lookup_fileinfoindex_by_t1_index(t1_index);

        // TODO: Make a constant for Redirect
        // if (file_info.flags & 0x00000010) == 0x10 {
        //     file_info = loaded_arc.lookup_file_information_by_t1_index(file_index.file_info_index);
        // }

        let mut sub_index = loaded_arc.lookup_fileinfosubindex_by_index(file_info.sub_index_index);

        // TODO: Make a constant for Regional
        if (file_info.flags & 0x00008000) == 0x8000 {
            sub_index = loaded_arc.lookup_fileinfosubindex_by_index(file_info.sub_index_index + 1 + self.get_region());
        }

        unsafe {
            let sub_file = loaded_arc.sub_files.offset(sub_index.sub_file_index as isize) as *mut SubFile;
            &mut *sub_file
        }
    }

    pub fn get_file_content(&self) -> Vec<u8> {
        // TODO: Add error handling in case the user deleted the file while running
        // TODO: Callbacks
        fs::read(&self.path).unwrap()
    }

    pub fn filesize_replacement(&mut self) {
        let loaded_tables = LoadedTables::get_instance();

        unsafe {
            let hashindexgroup_slice = slice::from_raw_parts(loaded_tables.get_arc().file_info_path,(*loaded_tables).table1_len as usize);

            // TODO: Figure out why bsearch does not work
            let t1_index = match hashindexgroup_slice.iter().position(|x| x.path.hash40.as_u64() == self.hash)
            {
                Some(index) => index as u32,
                None => {
                    println!("[ARC::Patching] File '{}' does not have a hash found in table1, skipping",self.path.display());
                    return;
                }
            };

            // Backup the Subfile for when file watching is added
            self.orig_subfile = self.get_subfile(t1_index).clone();

            let mut subfile = self.get_subfile(t1_index);

            //println!("[ARC::Patching] File '{}', decomp size: {:x}",self.path.display(),subfile.decompressed_size);

            if subfile.decompressed_size < self.filesize {
                 subfile.decompressed_size = self.filesize;
                println!("[ARC::Patching] File '{}' has a new patched decompressed size: {:#x}",self.path.display(),subfile.decompressed_size);
            }
        }
    }
}