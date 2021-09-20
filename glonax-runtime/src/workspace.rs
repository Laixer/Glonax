use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub struct Workspace {
    #[allow(dead_code)]
    db: sled::Db,
    #[allow(dead_code)]
    lock: File,
}

impl Workspace {
    /// Construct new workspace.
    ///
    /// The workspace is both a directory and a key value store.
    /// If the provided directory does not exist it will be created.
    pub fn new(path: &PathBuf) -> super::runtime::Result<Self> {
        Self::setup_if_not_exists(&path);

        debug!("Using workspace directory {}", path.to_str().unwrap());

        let lock = Self::lock(path)?;

        let db = sled::Config::default()
            .path(path)
            .flush_every_ms(Some(200))
            .open()
            .unwrap();

        Ok(Self { db, lock })
    }

    fn lock(path: &Path) -> super::runtime::Result<File> {
        use fs2::FileExt;

        let file = File::create(path.join("lock")).unwrap();

        // FUTURE: Check that err is indeed a lock.
        match file.try_lock_exclusive() {
            Ok(_) => Ok(file),
            Err(_) => Err(super::runtime::Error::WorkspaceInUse),
        }
    }

    /// Setup workspace directories if not exist.
    ///
    /// Thid method will create the absolte path.
    fn setup_if_not_exists(path: &PathBuf) {
        if !path.exists() {
            trace!("Workspace does not exit, creating one..");

            std::fs::create_dir_all(path).unwrap();
        }
    }
}
