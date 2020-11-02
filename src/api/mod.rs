use crate::core::{ ArcInfo, ArcFile };
use crate::core::callbacks::{ ArcCallback, CallbackType };
use crate::arc::ArcPath;
use crate::files::visit;

use crate::{ ARC_FILES, CALLBACKS };

#[no_mangle]
extern "C" fn has_hash(hash: u64) -> bool {
    if ARC_FILES.read().contains_key(&hash) {
        let callbacks = CALLBACKS.read();

        // If we have a file or extension callback
        if let Some(callbacks_vec) = callbacks.get(&CallbackType::File(hash)) {
            // Check every callback for that hash
            for callback in callbacks_vec {
                match callback {
                    ArcCallback::Listener(listener) => listener(hash),
                    _ => continue,
                }
            }
        }

        return true
    }

    false
}

#[no_mangle]
extern "C" fn get_arc_file(hash: u64) -> *mut ArcInfo  {
    match ARC_FILES.read().get(&hash) {
        // TODO: Change that garbage. Ew.
        Some(file_ctx) => {
            let inst = ArcInfo {
                hash: file_ctx.hash,
                filesize: file_ctx.filesize,
                extension: file_ctx.extension,
                orig_decomp_size: file_ctx.orig_subfile.decompressed_size,
            };
            Box::leak(Box::new(inst))
        },
        None => 0 as _,
    }
}

#[no_mangle]
extern "C" fn get_arc_file_path(hash: u64) -> *mut ArcPath  {
    match ARC_FILES.read().get(&hash) {
        Some(file_ctx) => {
            Box::leak(Box::new(ArcPath::new(file_ctx.path.to_str().unwrap())))
        },
        None => 0 as _,
    }
}

#[no_mangle]
extern "C" fn get_file(hash: u64) -> *mut ArcFile  {
    match ARC_FILES.read().get(&hash) {
        Some(file_ctx) => {
            let info = unsafe { *get_arc_file(hash) }; 

            // TODO: Implement `new` for ArcFile
            let mut arc_file = ArcFile {
                file: 0 as _,
                len: 0,
            };

            let callbacks = CALLBACKS.read();

            // Check if a file callback exists first, as these should be affected by extension changes right after
            if let Some(callbacks) = callbacks.get(&CallbackType::File(hash)) {
                // Check every callback available
                for callback in callbacks {
                    match callback {
                        // Replacers should provide the buffer and size themselves
                        ArcCallback::Replacer(replacer) => {
                            let result = replacer(hash, &mut arc_file);

                            // If the replacer did not create the file
                            if !result {
                                // Load the user's file instead
                                arc_file.file = Box::leak(file_ctx.get_file_content().into_boxed_slice()).as_mut_ptr();
                                arc_file.len = file_ctx.filesize;
                            }
                        },
                        // Filesize is ignored for Editors, as the size is already patched by now. (Hopefully)
                        ArcCallback::Editor(_, editor) => {
                            // If a Replacer hasn't created the file already
                            if arc_file.file.is_null() {
                                // Load the user's file instead
                                arc_file.file = Box::leak(file_ctx.get_file_content().into_boxed_slice()).as_mut_ptr();
                                arc_file.len = file_ctx.filesize;
                            }
                            
                            // Provide the file to the Editor
                            editor(&info, &mut arc_file);
                        }
                        // We don't care about Listeners here
                        _ => {},
                    }
                }
            }

            // Check if a extension callback exists
            if let Some(callbacks) = callbacks.get(&CallbackType::Extension(file_ctx.extension)) {
                // Check every callback available
                for callback in callbacks {
                    match callback {
                        // Replacers should provide the buffer and size themselves
                        ArcCallback::Replacer(replacer) => {
                            let result = replacer(hash, &mut arc_file);

                            // If the replacer did not create the file
                            if !result {
                                // Load the user's file instead
                                arc_file.file = Box::leak(file_ctx.get_file_content().into_boxed_slice()).as_mut_ptr();
                                arc_file.len = file_ctx.filesize;
                            }
                        },
                        // Filesize is ignored for Editors, as the size is already patched by now. (Hopefully)
                        ArcCallback::Editor(_, editor) => {
                            // If a Replacer hasn't created the file already
                            if arc_file.file.is_null() {
                                // Load the user's file instead
                                arc_file.file = Box::leak(file_ctx.get_file_content().into_boxed_slice()).as_mut_ptr();
                                arc_file.len = file_ctx.filesize;
                            }
                            
                            // Provide the file to the Editor
                            editor(&info, &mut arc_file);
                        }
                        // We don't care about Listeners here
                        _ => {},
                    }
                }
            }

            // Make sure arc_file contains the user's file if it hasn't been used by now.
            if arc_file.file.is_null() {
                arc_file.file = Box::leak(file_ctx.get_file_content().into_boxed_slice()).as_mut_ptr();
                arc_file.len = file_ctx.filesize;
            }

            // Return a pointer to the ArcResult
            Box::leak(Box::new(arc_file))
        },
        None => 0 as _,
    }
}

/// Calling this does not necessarily mean every path is going to be read.
#[no_mangle]
extern "C"  fn discover(path: &ArcPath, umm: bool) {
        let path = path.path().unwrap();

        if !path.exists() {
            return;
        }

        let files;

        if umm {
            files = visit::umm_directories(&path);
        } else {
            files = visit::directory(&path);
        }

        crate::add_entries(files)
}

#[no_mangle]
extern "C" fn install_callback(cb_type: CallbackType, arc_cb: ArcCallback) {
    let mut callbacks = CALLBACKS.write();

    if let Some(cb_vec) = callbacks.get_mut(&cb_type) {
        cb_vec.push(arc_cb);
        // Sort the vector so Replacers are before Editors
        cb_vec.sort();
        cb_vec.reverse();
    } else {
        callbacks.insert(cb_type, vec![arc_cb]);
    }
}

#[no_mangle]
extern "C" fn uninstall_callback(_handle: u32) {
    unimplemented!();
}

#[no_mangle]
extern "C" fn get_core_version() {
    unimplemented!();
}