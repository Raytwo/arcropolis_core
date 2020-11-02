use std::fs;
use std::path::{ Path, PathBuf };

use crate::core::FileCtx;

use smash::hash40;
use smash::resource::ResServiceState;

use log::warn;
use jwalk::WalkDir;

/// Visit Ultimate Mod Manager directories for backwards compatibility
pub(crate) fn umm_directories<P: AsRef<Path>>(path: &P) -> Vec<FileCtx> {
    let mut vec = Vec::new();

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();

        // Skip any directory starting with a period
        if entry.file_name().to_str().unwrap().starts_with(".") {
            continue;
        }

        let new_path = PathBuf::from(&format!("{}/{}", path.as_ref().display(), entry.path().display()));
        
        vec.append(&mut directory(&new_path));
    }

    vec
}

pub(crate) fn directory<P: AsRef<Path>>(path: &P) -> Vec<FileCtx> {
    WalkDir::new(path).parallelism(jwalk::Parallelism::Serial).into_iter().filter_map(|entry| {
        let entry = entry.unwrap();

        if entry.path().is_dir() {
            return None;
        }

        // Skip any file starting with a period, to avoid any error related to path.extension()
        if entry.file_name().to_str().unwrap().starts_with(".") {
            warn!("[ARC::Discovery] File '{}' starts with a period, skipping", entry.path().display());
            return None;
        }

        // Make sure the file has an extension to not cause issues with the code that follows
        if entry.path().extension() == None {
            warn!("[ARC::Discovery] File '{}' does not have an extension, skipping", entry.path().display());
            return None;
        }

        let mut arc_path = match entry.path().strip_prefix(path) {
            Ok(stripped_path) => String::from(stripped_path.to_str().unwrap()),
            Err(_) => return None,
        };

        if let Some(_) = arc_path.find(";") {
            arc_path = arc_path.replace(";", ":");
        }

        if let Some(regional_marker) = arc_path.find("+") {
            // TODO: Return here if the region doesn't match the game's
            arc_path.replace_range(regional_marker..arc_path.find(".").unwrap(), "");
        }

        // TODO: Move that stuff in a separate function that can handle more than one format
        // TODO: Have it just replace the extension to hash in FileCtx
        if let Some(ext) = arc_path.strip_suffix("mp4") {
            arc_path = format!("{}{}", ext, "webm");
        }

        // TODO: Rework the following atrocity

        let mut file_ctx = FileCtx::new();

        file_ctx.path = entry.path().to_path_buf();
        file_ctx.hash = hash40(&arc_path);
        let ext = Path::new(&arc_path).extension().unwrap().to_str().unwrap();
        file_ctx.extension = hash40(ext);

        file_ctx.filesize = match entry.metadata() {
            Ok(meta) => meta.len() as u32,
            Err(err) => panic!(err),
        };

        // TODO: Move this to the regional marker check
        if file_ctx.get_region() != ResServiceState::get_instance().regular_region_idx {
            warn!("[ARC::Discovery] File '{}' does not have a matching region, skipping", file_ctx.path.display());
            return None;
        }

        file_ctx.filesize_replacement();
        Some(file_ctx)
    }).collect()
}