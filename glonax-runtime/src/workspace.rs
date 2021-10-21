use std::{
    fs::{create_dir_all, File},
    path::Path,
};

// TODO: return IO result
/// Setup workspace directories if not exist.
///
/// Thid method will create the absolte path.
pub fn setup_if_not_exists(path: &Path) {
    if !path.exists() {
        trace!("Workspace does not exit, creating one..");

        create_dir_all(path).unwrap();
    }

    debug!("Using workspace directory {}", path.to_str().unwrap());
}

/// Create a new directory in the workspace.
pub fn create_directory<T: ToString>(path: &Path, name: &T) -> std::path::PathBuf {
    let path = path.join(name.to_string());

    create_dir_all(&path).unwrap();

    path
}

// TODO: return IO result
/// Lock the workspace.
pub fn lock(path: &Path) -> super::runtime::Result<File> {
    use fs2::FileExt;

    let file = File::create(path.join("lock")).unwrap();

    // FUTURE: Check that err is indeed a lock.
    match file.try_lock_exclusive() {
        Ok(_) => Ok(file),
        Err(_) => Err(super::runtime::Error::WorkspaceInUse),
    }
}
